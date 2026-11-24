using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.Linq;
using LumenicMenu.Api;
using LumenicMenu.Native;
using MelonLoader;
using Newtonsoft.Json;

namespace LumenicMenu.Bridge;

/// <summary>
/// Manages all registered LumenicMenu.Api extensions, builds only visible declarative node trees
/// when dirty, pushes them over the dedicated `PushExtensionsJson` FFI channel, and dispatches
/// Rust -> C# extension events back to mod callbacks with exception isolation.
/// </summary>
internal sealed class ExtensionHost
{
    private sealed class TabEntry
    {
        public string ModId;
        public string TabId;
        public ExtTabOptions Options;
        public Action<UiBuilder> RootBuild;
    }

    private sealed class SubPageEntry
    {
        public string PageId;
        public string Title;
        public string Subtitle;
        public bool HideHeader;
        public Action<UiBuilder> HeaderActions;
        public Action<UiBuilder> Build;
    }

    private sealed class ModalEntry
    {
        public string ModId;
        public ExtModalOptions Options;
        public Action<UiBuilder> Build;
    }

    private sealed class SlotEntry
    {
        public string ModId;
        public string SectionId;
        public string SlotId;
        public ExtSlotPosition Position;
        public int Priority;
        public Action<UiBuilder> Build;
    }

    private sealed class SectionStyleEntry
    {
        public string ModId;
        public string SectionId;
        public int Priority;
        public ExtSectionStyle Style;
    }

    private sealed class ThemeEntry
    {
        public string ModId;
        public int Priority;
        public ExtThemePatch Patch;
    }

    private sealed class HubEntry
    {
        public string HubId;
        public string ModId;
        public ExtHubEntryOptions Options;
        public Action<UiBuilder> Build;
    }

    private readonly object _lock = new object();
    private readonly List<TabEntry> _qmTabs = new List<TabEntry>();
    private readonly List<TabEntry> _mmTabs = new List<TabEntry>();
    private readonly List<HubEntry> _qmHubEntries = new List<HubEntry>();
    private readonly List<SlotEntry> _slots = new List<SlotEntry>();
    private readonly List<SectionStyleEntry> _sectionStyles = new List<SectionStyleEntry>();
    private readonly List<ThemeEntry> _themeEntries = new List<ThemeEntry>();
    private readonly List<(string ModId, string TileId)> _hiddenTiles = new List<(string, string)>();

    private string _activeQmTabKey;
    private string _activeMmTabKey;
    private readonly List<SubPageEntry> _qmSubPages = new List<SubPageEntry>();
    private readonly List<SubPageEntry> _mmSubPages = new List<SubPageEntry>();
    private ModalEntry _qmModal;
    private ModalEntry _mmModal;

    // Keyed by "{modId}:{targetId}" -> callback table for the currently rendered node tree.
    private readonly Dictionary<string, ExtCallbackTable> _callbacks = new Dictionary<string, ExtCallbackTable>(StringComparer.Ordinal);

    // Pending main-thread native uploads and UI actions (thread-safe for background tasks).
    private readonly ConcurrentQueue<(ExtSurface Surface, string Key, byte[] Encoded)> _pendingEncodedImages = new();
    private readonly ConcurrentQueue<(ExtSurface Surface, string Key, string Url)> _pendingImageUrls = new();
    private readonly ConcurrentQueue<(ExtSurface Surface, string Key, uint W, uint H, byte[] Rgba)> _pendingRgba = new();
    private readonly ConcurrentQueue<(string Family, byte[] Bytes)> _pendingFonts = new();
    private readonly ConcurrentQueue<(string Text, float Duration)> _pendingToasts = new();
    private readonly ConcurrentQueue<Action> _pendingMainThreadActions = new();
    private string _pendingDefaultFont;

    private volatile bool _dirty = true;
    private string _lastPushedJson = string.Empty;
    private int _slotSeq;

    public bool IsQmExtensionActive => !string.IsNullOrEmpty(_activeQmTabKey);
    public bool IsMmExtensionActive => !string.IsNullOrEmpty(_activeMmTabKey);

    public void MarkDirty() => _dirty = true;

    public void ClearActiveTab(ExtSurface surface)
    {
        lock (_lock)
        {
            if (surface == ExtSurface.QuickMenu)
            {
                if (_activeQmTabKey == null && _qmSubPages.Count == 0) return;
                _activeQmTabKey = null;
                _qmSubPages.Clear();
            }
            else if (surface == ExtSurface.MainMenu)
            {
                if (_activeMmTabKey == null && _mmSubPages.Count == 0) return;
                _activeMmTabKey = null;
                _mmSubPages.Clear();
            }
        }
        _dirty = true;
    }

    public ExtTabHandle RegisterTab(ExtSurface surface, string modId, ExtTabOptions options, Action<UiBuilder> build)
    {
        if (string.IsNullOrWhiteSpace(modId)) throw new ArgumentException("modId is required", nameof(modId));
        if (options == null || string.IsNullOrWhiteSpace(options.Id)) throw new ArgumentException("options.Id is required", nameof(options));
        if (build == null) throw new ArgumentNullException(nameof(build));

        lock (_lock)
        {
            var list = surface == ExtSurface.MainMenu ? _mmTabs : _qmTabs;
            list.RemoveAll(t => t.ModId == modId && t.TabId == options.Id);
            list.Add(new TabEntry
            {
                ModId = modId,
                TabId = options.Id,
                Options = options,
                RootBuild = build
            });
            list.Sort((a, b) => b.Options.Priority.CompareTo(a.Options.Priority));
        }
        _dirty = true;
        return new ExtTabHandle(this, modId, options.Id, surface);
    }

    public ExtTabHandle RegisterQuickMenuHubEntry(
        string hubId,
        ExtTabOptions hubOptions,
        string modId,
        ExtHubEntryOptions entry,
        Action<UiBuilder> build)
    {
        if (string.IsNullOrWhiteSpace(hubId)) hubId = LumenicHubs.Mods;
        if (string.IsNullOrWhiteSpace(modId)) throw new ArgumentException("modId is required", nameof(modId));
        if (entry == null || string.IsNullOrWhiteSpace(entry.Id)) throw new ArgumentException("entry.Id is required", nameof(entry));
        if (build == null) throw new ArgumentNullException(nameof(build));

        string hubModKey = $"hub:{hubId}";
        string hubTabId = "index";

        lock (_lock)
        {
            _qmHubEntries.RemoveAll(e => e.HubId == hubId && e.ModId == modId && e.Options.Id == entry.Id);
            _qmHubEntries.Add(new HubEntry
            {
                HubId = hubId,
                ModId = modId,
                Options = entry,
                Build = build
            });
            _qmHubEntries.Sort((a, b) => b.Options.Priority.CompareTo(a.Options.Priority));

            if (!_qmTabs.Any(t => t.ModId == hubModKey && t.TabId == hubTabId))
            {
                var opts = hubOptions ?? new ExtTabOptions
                {
                    Id = hubTabId,
                    Title = hubId == LumenicHubs.Mods ? "Extensions" : hubId,
                    Subtitle = "Installed Lumenic Extensions",
                    Icon = "extension",
                    Priority = -5
                };
                opts.Id = hubTabId;
                _qmTabs.Add(new TabEntry
                {
                    ModId = hubModKey,
                    TabId = hubTabId,
                    Options = opts,
                    RootBuild = ui => BuildHubIndexPage(hubId, hubModKey, hubTabId, ui)
                });
                _qmTabs.Sort((a, b) => b.Options.Priority.CompareTo(a.Options.Priority));
            }
        }
        _dirty = true;
        return new ExtTabHandle(this, hubModKey, hubTabId, ExtSurface.QuickMenu);
    }

    private void BuildHubIndexPage(string hubId, string hubModKey, string hubTabId, UiBuilder ui)
    {
        List<HubEntry> entries;
        lock (_lock)
        {
            entries = _qmHubEntries.Where(e => e.HubId == hubId).ToList();
        }

        if (entries.Count == 0)
        {
            ui.Card(c => c.Text("No mods registered in this hub.", fg: "muted"));
            return;
        }

        ui.Scroll($"hub_scroll_{hubId}", scroll =>
        {
            scroll.Grid(2, grid =>
            {
                foreach (HubEntry item in entries)
                {
                    string btnId = $"hub_open_{item.ModId}_{item.Options.Id}";
                    string badge = SafeEval(item.ModId, item.Options.BadgeText);
                    grid.CustomButton(btnId, card =>
                    {
                        card.Row(r =>
                        {
                            r.Icon(string.IsNullOrEmpty(item.Options.Icon) ? "extension" : item.Options.Icon, size: 18f, fg: "accent");
                            r.Col(col =>
                            {
                                col.Row(top =>
                                {
                                    top.Text(item.Options.Title ?? item.Options.Id, size: 12f, weight: "semibold", ellipsis: true,
                                        style: new ExtStyle { Grow = 1f });
                                    if (!string.IsNullOrEmpty(badge))
                                        top.Badge(badge, size: 9.5f);
                                }, new ExtStyle { Align = "center", Gap = 5f });
                                if (!string.IsNullOrEmpty(item.Options.Subtitle))
                                    col.Text(item.Options.Subtitle, size: 10f, fg: "muted", ellipsis: true);
                            }, new ExtStyle { Grow = 1f, Gap = 2f });
                            r.Icon("chevron_right", size: 15f, fg: "dim");
                        }, new ExtStyle { Align = "center", Gap = 8f, WFrac = 1f });
                    }, () =>
                    {
                        PushSubPage(ExtSurface.QuickMenu, hubModKey, hubTabId, new ExtSubPageOptions
                        {
                            Id = $"{item.ModId}_{item.Options.Id}",
                            Title = item.Options.Title ?? item.Options.Id,
                            Subtitle = item.Options.Subtitle ?? string.Empty,
                            HideHeader = item.Options.HideHeader,
                            HeaderActions = item.Options.HeaderActions
                        }, item.Build);
                    }, style: new ExtStyle { Grow = 1f, MinH = 54f, Px = 10f, Py = 8f });
                }
            }, new ExtStyle { Gap = 7f });
        }, new ExtStyle { Grow = 1f, Gap = 6f });
    }

    public void UnregisterTab(ExtSurface surface, string modId, string tabId)
    {
        lock (_lock)
        {
            var list = surface == ExtSurface.MainMenu ? _mmTabs : _qmTabs;
            list.RemoveAll(t => t.ModId == modId && t.TabId == tabId);
            string key = $"{modId}:{tabId}";
            if (surface == ExtSurface.MainMenu && _activeMmTabKey == key)
            {
                _activeMmTabKey = null;
                _mmSubPages.Clear();
            }
            else if (surface == ExtSurface.QuickMenu && _activeQmTabKey == key)
            {
                _activeQmTabKey = null;
                _qmSubPages.Clear();
            }
        }
        _dirty = true;
    }

    public void SelectTab(ExtSurface surface, string compositeKey)
    {
        lock (_lock)
        {
            if (surface == ExtSurface.MainMenu)
            {
                _activeMmTabKey = string.IsNullOrEmpty(compositeKey) ? null : compositeKey;
                _mmSubPages.Clear();
            }
            else
            {
                _activeQmTabKey = string.IsNullOrEmpty(compositeKey) ? null : compositeKey;
                _qmSubPages.Clear();
            }
        }
        _dirty = true;
    }

    public void PushSubPage(ExtSurface surface, string modId, string tabId, ExtSubPageOptions options, Action<UiBuilder> build)
    {
        if (options == null || build == null) return;
        lock (_lock)
        {
            string key = $"{modId}:{tabId}";
            var stack = surface == ExtSurface.MainMenu ? _mmSubPages : _qmSubPages;
            if (surface == ExtSurface.MainMenu) _activeMmTabKey = key;
            else _activeQmTabKey = key;

            stack.Add(new SubPageEntry
            {
                PageId = string.IsNullOrWhiteSpace(options.Id) ? $"sub_{stack.Count + 1}" : options.Id,
                Title = options.Title ?? string.Empty,
                Subtitle = options.Subtitle ?? string.Empty,
                HideHeader = options.HideHeader,
                HeaderActions = options.HeaderActions,
                Build = build
            });
        }
        _dirty = true;
    }

    public void PopSubPage(ExtSurface surface)
    {
        lock (_lock)
        {
            var stack = surface == ExtSurface.MainMenu ? _mmSubPages : _qmSubPages;
            if (stack.Count > 0)
            {
                stack.RemoveAt(stack.Count - 1);
            }
        }
        _dirty = true;
    }

    public ExtRegistrationHandle OpenModal(ExtSurface surface, string modId, ExtModalOptions options, Action<UiBuilder> build)
    {
        var entry = new ModalEntry
        {
            ModId = modId ?? "ext",
            Options = options ?? new ExtModalOptions(),
            Build = build
        };
        lock (_lock)
        {
            if (surface == ExtSurface.MainMenu) _mmModal = entry;
            else _qmModal = entry;
        }
        _dirty = true;
        return new ExtRegistrationHandle(this, () =>
        {
            lock (_lock)
            {
                if (surface == ExtSurface.MainMenu && ReferenceEquals(_mmModal, entry)) _mmModal = null;
                else if (surface != ExtSurface.MainMenu && ReferenceEquals(_qmModal, entry)) _qmModal = null;
            }
        });
    }

    public void CloseModal(ExtSurface surface)
    {
        lock (_lock)
        {
            if (surface == ExtSurface.MainMenu) _mmModal = null;
            else if (surface == ExtSurface.QuickMenu) _qmModal = null;
            else { _mmModal = null; _qmModal = null; }
        }
        _dirty = true;
    }

    public void ShowToast(string text, float durationSeconds)
    {
        if (string.IsNullOrWhiteSpace(text)) return;
        _pendingToasts.Enqueue((text, durationSeconds));
    }

    public void OpenKeyboard(
        string initialText,
        Action<string> onSubmit,
        string title,
        string placeholder,
        int maxLen,
        bool masked,
        bool multiLine,
        ExtSurface surface)
    {
        _pendingMainThreadActions.Enqueue(() =>
        {
            try
            {
                var km = ABI_RC.Core.UI.UIRework.Managers.KeyboardManager.Instance;
                if (km == null || !km.IsReady)
                    return;

                var source = surface == ExtSurface.MainMenu
                    ? ABI_RC.Core.UI.UIRework.Managers.KeyboardManager.OpenSource.MainMenu
                    : ABI_RC.Core.UI.UIRework.Managers.KeyboardManager.OpenSource.QuickMenu;

                km.ShowKeyboard(
                    initialText ?? string.Empty,
                    submitted =>
                    {
                        try { onSubmit?.Invoke(submitted ?? string.Empty); }
                        catch (Exception ex) { MelonLogger.Error($"[LumenicApi] OpenKeyboard callback failed: {ex}"); }
                    },
                    placeholder ?? string.Empty,
                    "Confirm",
                    Math.Max(0, maxLen),
                    masked,
                    multiLine: multiLine,
                    title ?? "Input",
                    source);
            }
            catch (Exception ex)
            {
                MelonLogger.Warning($"[LumenicApi] OpenKeyboard failed: {ex.Message}");
            }
        });
    }

    public ExtRegistrationHandle RegisterSlot(
        string modId,
        string sectionId,
        ExtSlotPosition position,
        Action<UiBuilder> build,
        int priority,
        string slotId)
    {
        if (string.IsNullOrWhiteSpace(modId) || string.IsNullOrWhiteSpace(sectionId) || build == null)
            return new ExtRegistrationHandle(this, null);

        string resolvedId = string.IsNullOrWhiteSpace(slotId)
            ? $"slot_{System.Threading.Interlocked.Increment(ref _slotSeq)}"
            : slotId;

        var entry = new SlotEntry
        {
            ModId = modId,
            SectionId = sectionId,
            SlotId = resolvedId,
            Position = position,
            Priority = priority,
            Build = build
        };

        lock (_lock)
        {
            _slots.Add(entry);
        }
        _dirty = true;
        return new ExtRegistrationHandle(this, () =>
        {
            lock (_lock) { _slots.Remove(entry); }
        });
    }

    public ExtRegistrationHandle SetSectionStyle(string modId, string sectionId, ExtSectionStyle style, int priority)
    {
        if (string.IsNullOrWhiteSpace(modId) || string.IsNullOrWhiteSpace(sectionId) || style == null)
            return new ExtRegistrationHandle(this, null);

        var entry = new SectionStyleEntry
        {
            ModId = modId,
            SectionId = sectionId,
            Priority = priority,
            Style = style
        };
        lock (_lock)
        {
            _sectionStyles.Add(entry);
        }
        _dirty = true;
        return new ExtRegistrationHandle(this, () =>
        {
            lock (_lock) { _sectionStyles.Remove(entry); }
        });
    }

    public ExtRegistrationHandle HideBuiltInTile(string modId, string tileId)
    {
        if (string.IsNullOrWhiteSpace(tileId))
            return new ExtRegistrationHandle(this, null);

        var pair = (modId ?? "ext", tileId);
        lock (_lock)
        {
            _hiddenTiles.Add(pair);
        }
        _dirty = true;
        return new ExtRegistrationHandle(this, () =>
        {
            lock (_lock) { _hiddenTiles.Remove(pair); }
        });
    }

    public IDisposable SetThemeOverride(string modId, ExtThemePatch patch, int priority)
    {
        var entry = new ThemeEntry
        {
            ModId = modId ?? "ext",
            Priority = priority,
            Patch = patch ?? new ExtThemePatch()
        };
        lock (_lock)
        {
            _themeEntries.Add(entry);
        }
        _dirty = true;
        return new ExtRegistrationHandle(this, () =>
        {
            lock (_lock) { _themeEntries.Remove(entry); }
        });
    }

    public bool RegisterFont(string familyName, byte[] fontBytes)
    {
        if (fontBytes == null || fontBytes.Length == 0) return false;
        _pendingFonts.Enqueue((familyName ?? string.Empty, fontBytes));
        _dirty = true;
        return true;
    }

    public bool SetDefaultFontFamily(string familyName)
    {
        if (string.IsNullOrWhiteSpace(familyName)) return false;
        _pendingDefaultFont = familyName;
        _dirty = true;
        return true;
    }

    public void RegisterImageBytes(string key, byte[] bytes, ExtSurface surface)
    {
        if (string.IsNullOrWhiteSpace(key) || bytes == null || bytes.Length == 0) return;
        string normalized = NormalizeImageKey(key);
        _pendingEncodedImages.Enqueue((surface, normalized, bytes));
    }

    public void RegisterImageUrl(string key, string url, ExtSurface surface)
    {
        if (string.IsNullOrWhiteSpace(key) || string.IsNullOrWhiteSpace(url)) return;
        string normalized = NormalizeImageKey(key);
        _pendingImageUrls.Enqueue((surface, normalized, url));
    }

    public void RegisterRawRgba(string key, uint width, uint height, byte[] rgba, ExtSurface surface)
    {
        if (string.IsNullOrWhiteSpace(key) || rgba == null || width == 0 || height == 0) return;
        string normalized = NormalizeImageKey(key);
        _pendingRgba.Enqueue((surface, normalized, width, height, rgba));
    }

    private static string NormalizeImageKey(string key)
        => key.StartsWith("ext:", StringComparison.Ordinal) ? key : key;

    public void UnregisterMod(string modId)
    {
        if (string.IsNullOrWhiteSpace(modId)) return;
        lock (_lock)
        {
            _qmTabs.RemoveAll(t => t.ModId == modId);
            _mmTabs.RemoveAll(t => t.ModId == modId);
            _qmHubEntries.RemoveAll(e => e.ModId == modId);
            _qmTabs.RemoveAll(t => t.ModId.StartsWith("hub:", StringComparison.Ordinal)
                && !_qmHubEntries.Any(e => $"hub:{e.HubId}" == t.ModId));
            _slots.RemoveAll(s => s.ModId == modId);
            _sectionStyles.RemoveAll(s => s.ModId == modId);
            _themeEntries.RemoveAll(t => t.ModId == modId);
            _hiddenTiles.RemoveAll(h => h.ModId == modId);
            if (_activeQmTabKey != null && _activeQmTabKey.StartsWith(modId + ":", StringComparison.Ordinal))
            {
                _activeQmTabKey = null;
                _qmSubPages.Clear();
            }
            if (_activeMmTabKey != null && _activeMmTabKey.StartsWith(modId + ":", StringComparison.Ordinal))
            {
                _activeMmTabKey = null;
                _mmSubPages.Clear();
            }
            if (_qmModal?.ModId == modId) _qmModal = null;
            if (_mmModal?.ModId == modId) _mmModal = null;
        }
        _dirty = true;
    }

    public void InvalidateNativeCache()
    {
        _lastPushedJson = string.Empty;
        _dirty = true;
    }

    /// <summary>
    /// Called on Unity's main thread from LumenicMenuMod.OnLateUpdate to drain pending
    /// texture/font uploads and push extension JSON only when dirty.
    /// </summary>
    public void Tick(NativeBridge bridge, ImageLoaderService images, bool mainVisible, bool quickVisible)
    {
        if (bridge == null || !bridge.IsHealthy)
            return;

        while (_pendingToasts.TryDequeue(out var toast))
        {
            LumenicMenuMod.Instance?.UiLib?.HandleShowAlertToast(toast.Text, Math.Max(1, (int)Math.Round(toast.Duration)));
            LumenicMenuMod.Instance?.MarkStateDirty();
        }

        while (_pendingMainThreadActions.TryDequeue(out var action))
        {
            try { action(); }
            catch (Exception ex) { MelonLogger.Error($"[LumenicApi] Main-thread action failed: {ex}"); }
        }

        while (_pendingFonts.TryDequeue(out var f))
        {
            bridge.RegisterFontBytes(f.Family, f.Bytes);
        }
        if (!string.IsNullOrEmpty(_pendingDefaultFont))
        {
            string fam = _pendingDefaultFont;
            _pendingDefaultFont = null;
            bridge.SetDefaultFontFamily(fam);
        }

        while (_pendingEncodedImages.TryDequeue(out var img))
        {
            if (img.Surface == ExtSurface.MainMenu || img.Surface == ExtSurface.Both)
                bridge.UploadImageBytes(NativeBridge.SurfaceMain, img.Key, img.Encoded);
            if (img.Surface == ExtSurface.QuickMenu || img.Surface == ExtSurface.Both)
                bridge.UploadImageBytes(NativeBridge.SurfaceQuick, img.Key, img.Encoded);
        }

        while (_pendingImageUrls.TryDequeue(out var u))
        {
            if (images == null) continue;
            if (u.Surface == ExtSurface.MainMenu || u.Surface == ExtSurface.Both)
                images.RequestUrl(NativeBridge.SurfaceMain, u.Key, u.Url);
            if (u.Surface == ExtSurface.QuickMenu || u.Surface == ExtSurface.Both)
                images.RequestUrl(NativeBridge.SurfaceQuick, u.Key, u.Url);
        }

        while (_pendingRgba.TryDequeue(out var r))
        {
            if (r.Surface == ExtSurface.MainMenu || r.Surface == ExtSurface.Both)
                bridge.UploadRgbaTexture(NativeBridge.SurfaceMain, r.Key, r.W, r.H, r.Rgba);
            if (r.Surface == ExtSurface.QuickMenu || r.Surface == ExtSurface.Both)
                bridge.UploadRgbaTexture(NativeBridge.SurfaceQuick, r.Key, r.W, r.H, r.Rgba);
        }

        if (!_dirty)
            return;

        _dirty = false;
        string json = BuildSnapshotJson(mainVisible, quickVisible);
        if (!string.Equals(json, _lastPushedJson, StringComparison.Ordinal))
        {
            if (bridge.PushExtensionsJson(json))
            {
                _lastPushedJson = json;
            }
        }
    }

    public void DispatchEvent(string extId, string targetId, string nodeId, string kind, string strVal, float floatVal, bool boolVal)
    {
        string cbKey = $"{extId}:{targetId}";
        ExtCallbackTable table;
        lock (_lock)
        {
            if (!_callbacks.TryGetValue(cbKey, out table))
                return;
        }

        try
        {
            switch (kind)
            {
                case "click":
                    if (table.Clicks.TryGetValue(nodeId, out var onClick))
                        onClick?.Invoke();
                    break;
                case "change":
                    if (table.BoolChanges.TryGetValue(nodeId, out var onBool))
                        onBool?.Invoke(boolVal);
                    else if (table.FloatChanges.TryGetValue(nodeId, out var onFloat))
                        onFloat?.Invoke(floatVal);
                    else if (table.IntChanges.TryGetValue(nodeId, out var onInt))
                        onInt?.Invoke((int)Math.Round(floatVal));
                    else if (table.StringChanges.TryGetValue(nodeId, out var onStr))
                        onStr?.Invoke(strVal ?? string.Empty);
                    break;
                case "submit":
                    if (table.StringSubmits.TryGetValue(nodeId, out var onSub))
                        onSub?.Invoke(strVal ?? string.Empty);
                    else if (table.StringChanges.TryGetValue(nodeId, out var onStrFallback))
                        onStrFallback?.Invoke(strVal ?? string.Empty);
                    break;
            }
        }
        catch (Exception ex)
        {
            MelonLogger.Error($"[LumenicApi] Extension '{extId}' callback failed on '{targetId}/{nodeId}' ({kind}): {ex}");
        }
    }

    private string BuildSnapshotJson(bool mainVisible, bool quickVisible)
    {
        var newCallbacks = new Dictionary<string, ExtCallbackTable>(StringComparer.Ordinal);
        object payload;

        lock (_lock)
        {
            var theme = MergeThemeLocked();

            var qmTabsDto = _qmTabs.Select(t => new
            {
                ext_id = t.ModId,
                id = t.TabId,
                label = t.Options.Title ?? t.TabId,
                icon = t.Options.Icon ?? "extension",
                icon_key = t.Options.IconKey ?? string.Empty,
                badge = SafeEval(t.ModId, t.Options.BadgeText),
                priority = t.Options.Priority
            }).ToList();

            var mmTabsDto = _mmTabs.Select(t => new
            {
                ext_id = t.ModId,
                id = t.TabId,
                label = t.Options.Title ?? t.TabId,
                icon = t.Options.Icon ?? "extension",
                icon_key = t.Options.IconKey ?? string.Empty,
                badge = SafeEval(t.ModId, t.Options.BadgeText),
                priority = t.Options.Priority
            }).ToList();

            object qmPageDto = quickVisible ? BuildActivePageLocked(_qmTabs, _activeQmTabKey, _qmSubPages, newCallbacks) : null;
            object mmPageDto = mainVisible ? BuildActivePageLocked(_mmTabs, _activeMmTabKey, _mmSubPages, newCallbacks) : null;
            object qmModalDto = quickVisible ? BuildModalLocked(_qmModal, newCallbacks) : null;
            object mmModalDto = mainVisible ? BuildModalLocked(_mmModal, newCallbacks) : null;

            var sectionsDto = BuildSectionsLocked(mainVisible, quickVisible, newCallbacks);
            var hiddenTiles = _hiddenTiles.Select(h => h.TileId).Distinct(StringComparer.Ordinal).ToList();

            // Preserve callbacks for the surface that is currently hidden so reopening doesn't lose handlers.
            foreach (var kv in newCallbacks)
            {
                _callbacks[kv.Key] = kv.Value;
            }

            payload = new
            {
                theme,
                qm_tabs = qmTabsDto,
                mm_tabs = mmTabsDto,
                qm_active_page = qmPageDto,
                mm_active_page = mmPageDto,
                qm_modal = qmModalDto,
                mm_modal = mmModalDto,
                hidden_tiles = hiddenTiles,
                sections = sectionsDto
            };
        }

        return JsonConvert.SerializeObject(payload);
    }

    private object MergeThemeLocked()
    {
        float? fontScaleOverride = null;
        float fontScaleMul = 1f, titleScale = 1f, bodyScale = 1f, captionScale = 1f, iconScale = 1f;
        float spacingScale = 1f, paddingScale = 1f, radiusScale = 1f, bevelScale = 1f;
        float? tileMinHeight = null;
        int? qmActionCols = null;
        float glowIntensity = 1f, shadowStrength = 1f, panelOpacity = 1f;
        string cardBg = null, cardInnerBg = null, tileIdleBg = null, tileIdleHover = null;
        string borderSubtle = null, textPrimary = null, textMuted = null, textDim = null, accentOverride = null;

        // Apply ascending priority so higher priority overrides lower.
        foreach (var entry in _themeEntries.OrderBy(e => e.Priority))
        {
            var p = entry.Patch;
            if (p.FontScaleOverride.HasValue) fontScaleOverride = p.FontScaleOverride;
            if (p.FontScaleMultiplier.HasValue) fontScaleMul = p.FontScaleMultiplier.Value;
            if (p.TitleScale.HasValue) titleScale = p.TitleScale.Value;
            if (p.BodyScale.HasValue) bodyScale = p.BodyScale.Value;
            if (p.CaptionScale.HasValue) captionScale = p.CaptionScale.Value;
            if (p.IconScale.HasValue) iconScale = p.IconScale.Value;
            if (p.SpacingScale.HasValue) spacingScale = p.SpacingScale.Value;
            if (p.PaddingScale.HasValue) paddingScale = p.PaddingScale.Value;
            if (p.RadiusScale.HasValue) radiusScale = p.RadiusScale.Value;
            if (p.BevelScale.HasValue) bevelScale = p.BevelScale.Value;
            if (p.TileMinHeight.HasValue) tileMinHeight = p.TileMinHeight;
            if (p.QmActionColumns.HasValue) qmActionCols = p.QmActionColumns;
            if (p.GlowIntensity.HasValue) glowIntensity = p.GlowIntensity.Value;
            if (p.ShadowStrength.HasValue) shadowStrength = p.ShadowStrength.Value;
            if (p.PanelOpacity.HasValue) panelOpacity = p.PanelOpacity.Value;

            if (!string.IsNullOrEmpty(p.CardBg)) cardBg = p.CardBg;
            if (!string.IsNullOrEmpty(p.CardInnerBg)) cardInnerBg = p.CardInnerBg;
            if (!string.IsNullOrEmpty(p.TileIdleBg)) tileIdleBg = p.TileIdleBg;
            if (!string.IsNullOrEmpty(p.TileIdleHover)) tileIdleHover = p.TileIdleHover;
            if (!string.IsNullOrEmpty(p.BorderSubtle)) borderSubtle = p.BorderSubtle;
            if (!string.IsNullOrEmpty(p.TextPrimary)) textPrimary = p.TextPrimary;
            if (!string.IsNullOrEmpty(p.TextMuted)) textMuted = p.TextMuted;
            if (!string.IsNullOrEmpty(p.TextDim)) textDim = p.TextDim;
            if (!string.IsNullOrEmpty(p.AccentOverride)) accentOverride = p.AccentOverride;
        }

        return new
        {
            font_scale_override = fontScaleOverride,
            font_scale_mul = fontScaleMul,
            title_scale = titleScale,
            body_scale = bodyScale,
            caption_scale = captionScale,
            icon_scale = iconScale,
            spacing_scale = spacingScale,
            padding_scale = paddingScale,
            radius_scale = radiusScale,
            bevel_scale = bevelScale,
            tile_min_height = tileMinHeight,
            qm_action_columns = qmActionCols,
            glow_intensity = glowIntensity,
            shadow_strength = shadowStrength,
            panel_opacity = panelOpacity,
            card_bg = cardBg,
            card_inner_bg = cardInnerBg,
            tile_idle_bg = tileIdleBg,
            tile_idle_hover = tileIdleHover,
            border_subtle = borderSubtle,
            text_primary = textPrimary,
            text_muted = textMuted,
            text_dim = textDim,
            accent_override = accentOverride
        };
    }

    private object BuildActivePageLocked(
        List<TabEntry> tabs,
        string activeKey,
        List<SubPageEntry> subPages,
        Dictionary<string, ExtCallbackTable> callbacks)
    {
        if (string.IsNullOrEmpty(activeKey)) return null;
        TabEntry tab = tabs.FirstOrDefault(t => $"{t.ModId}:{t.TabId}" == activeKey);
        if (tab == null) return null;

        bool hasSub = subPages.Count > 0;
        SubPageEntry topSub = hasSub ? subPages[subPages.Count - 1] : null;
        string pageId = hasSub ? topSub.PageId : tab.TabId;
        string title = hasSub ? topSub.Title : (tab.Options.Title ?? tab.TabId);
        string subtitle = hasSub ? topSub.Subtitle : (tab.Options.Subtitle ?? string.Empty);
        bool hideHeader = hasSub ? topSub.HideHeader : tab.Options.HideHeader;
        Action<UiBuilder> headerActionsBuild = hasSub ? topSub.HeaderActions : tab.Options.HeaderActions;
        Action<UiBuilder> rootBuild = hasSub ? topSub.Build : tab.RootBuild;

        var cb = new ExtCallbackTable();
        callbacks[$"{tab.ModId}:{pageId}"] = cb;

        var headerNodes = new List<ExtNodeDto>();
        if (headerActionsBuild != null)
        {
            SafeBuild(tab.ModId, () => headerActionsBuild(new UiBuilder(headerNodes, cb)));
        }

        var rootChildren = new List<ExtNodeDto>();
        SafeBuild(tab.ModId, () => rootBuild(new UiBuilder(rootChildren, cb)));
        ExtNodeDto root = WrapRoot(rootChildren);

        return new
        {
            ext_id = tab.ModId,
            tab_id = tab.TabId,
            page_id = pageId,
            title,
            subtitle,
            can_go_back = hasSub,
            hide_header = hideHeader,
            header_actions = headerNodes,
            root
        };
    }

    private object BuildModalLocked(ModalEntry modal, Dictionary<string, ExtCallbackTable> callbacks)
    {
        if (modal == null) return null;
        string modalId = string.IsNullOrWhiteSpace(modal.Options.Id) ? "modal" : modal.Options.Id;
        var cb = new ExtCallbackTable();
        callbacks[$"{modal.ModId}:{modalId}"] = cb;

        var children = new List<ExtNodeDto>();
        SafeBuild(modal.ModId, () => modal.Build?.Invoke(new UiBuilder(children, cb)));

        return new
        {
            ext_id = modal.ModId,
            modal_id = modalId,
            title = modal.Options.Title ?? "Dialog",
            width = modal.Options.Width,
            close_on_backdrop = modal.Options.CloseOnBackdrop,
            root = WrapRoot(children)
        };
    }

    private Dictionary<string, object> BuildSectionsLocked(
        bool mainVisible,
        bool quickVisible,
        Dictionary<string, ExtCallbackTable> callbacks)
    {
        var result = new Dictionary<string, object>(StringComparer.Ordinal);
        var sectionIds = new HashSet<string>(StringComparer.Ordinal);
        foreach (var s in _slots) sectionIds.Add(s.SectionId);
        foreach (var st in _sectionStyles) sectionIds.Add(st.SectionId);

        foreach (string secId in sectionIds)
        {
            bool isQm = secId.StartsWith("qm.", StringComparison.Ordinal);
            bool isMm = secId.StartsWith("mm.", StringComparison.Ordinal);
            if (isQm && !quickVisible) continue;
            if (isMm && !mainVisible) continue;

            ExtSectionStyle mergedStyle = MergeSectionStyleLocked(secId);
            var matchingSlots = _slots.Where(s => s.SectionId == secId).OrderByDescending(s => s.Priority).ToList();

            object BuildSlotDto(SlotEntry s)
            {
                var cb = new ExtCallbackTable();
                callbacks[$"{s.ModId}:{s.SlotId}"] = cb;
                var children = new List<ExtNodeDto>();
                SafeBuild(s.ModId, () => s.Build(new UiBuilder(children, cb)));
                return new
                {
                    ext_id = s.ModId,
                    slot_id = s.SlotId,
                    priority = s.Priority,
                    root = WrapRoot(children)
                };
            }

            var replaceSlot = matchingSlots.FirstOrDefault(s => s.Position == ExtSlotPosition.Replace);
            result[secId] = new
            {
                style = mergedStyle,
                replace = replaceSlot != null ? BuildSlotDto(replaceSlot) : null,
                before = matchingSlots.Where(s => s.Position == ExtSlotPosition.Before).Select(BuildSlotDto).ToList(),
                after = matchingSlots.Where(s => s.Position == ExtSlotPosition.After).Select(BuildSlotDto).ToList(),
                inside_start = matchingSlots.Where(s => s.Position == ExtSlotPosition.InsideStart).Select(BuildSlotDto).ToList(),
                inside_end = matchingSlots.Where(s => s.Position == ExtSlotPosition.InsideEnd).Select(BuildSlotDto).ToList()
            };
        }

        return result;
    }

    private ExtSectionStyle MergeSectionStyleLocked(string sectionId)
    {
        var matching = _sectionStyles.Where(s => s.SectionId == sectionId).OrderBy(s => s.Priority).ToList();
        if (matching.Count == 0) return null;

        var merged = new ExtSectionStyle();
        foreach (var m in matching)
        {
            if (m.Style.Hidden) merged.Hidden = true;
            if (m.Style.FontScale.HasValue) merged.FontScale = m.Style.FontScale;
            if (m.Style.SpacingScale.HasValue) merged.SpacingScale = m.Style.SpacingScale;
            if (m.Style.PaddingScale.HasValue) merged.PaddingScale = m.Style.PaddingScale;
            if (m.Style.Columns.HasValue) merged.Columns = m.Style.Columns;
            if (m.Style.MinHeight.HasValue) merged.MinHeight = m.Style.MinHeight;
            if (!string.IsNullOrEmpty(m.Style.CardBg)) merged.CardBg = m.Style.CardBg;
            if (!string.IsNullOrEmpty(m.Style.Border)) merged.Border = m.Style.Border;
        }
        return merged;
    }

    private static ExtNodeDto WrapRoot(List<ExtNodeDto> children)
    {
        if (children.Count == 1)
            return children[0];
        return new ExtNodeDto
        {
            Kind = "col",
            Style = new ExtStyle { Grow = 1f, WFrac = 1f, Gap = 6f },
            Children = children
        };
    }

    private static string SafeEval(string modId, Func<string> fn)
    {
        if (fn == null) return string.Empty;
        try { return fn() ?? string.Empty; }
        catch (Exception ex)
        {
            MelonLogger.Error($"[LumenicApi] Extension '{modId}' badge callback failed: {ex.Message}");
            return string.Empty;
        }
    }

    private static void SafeBuild(string modId, Action fn)
    {
        try { fn(); }
        catch (Exception ex)
        {
            MelonLogger.Error($"[LumenicApi] Extension '{modId}' UI build failed: {ex}");
        }
    }
}
