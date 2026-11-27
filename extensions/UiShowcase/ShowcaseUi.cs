using System;
using System.Collections.Generic;
using LumenicMenu.Api;

namespace LumenicUiShowcase;

// Coordinates tab/hub registrations, shared demo state, live section/theme handles, and refresh.
internal sealed partial class ShowcaseUi : IDisposable
{
    public const string ModId = "com.lumenic.uishowcase";

    private static readonly string[] SectionTabLabels =
    {
        "Controls",
        "Layout",
        "Media",
        "Overlays",
        "Sections & Theme"
    };

    private static readonly string[] DropdownFruits =
    {
        "Obsidian", "Cyan Pulse", "Emerald Mint", "Amber Glow", "Crimson Edge"
    };

    private static readonly string[] SearchableIcons =
    {
        "bolt", "palette", "widgets", "smart_display", "tune",
        "layers", "rocket_launch", "auto_awesome", "shield", "terminal"
    };

    private static readonly string[] ThemePresetLabels =
    {
        "None (User Default)",
        "Large Accessible Text (135%)",
        "Cyan Accent + Sharp Bevel",
        "Compact High Density"
    };

    private ExtTabHandle _mmTab;
    private ExtTabHandle _qmTab;
    private ExtTabHandle _hubEntry;

    // Active section/theme handles (all opt-in, off by default so user menus stay untouched)
    private ExtRegistrationHandle _qmBannerHandle;
    private ExtRegistrationHandle _qmActionTileHandle;
    private ExtRegistrationHandle _qmActionStyleHandle;
    private ExtRegistrationHandle _qmPlayerActionHandle;
    private ExtRegistrationHandle _qmNotifReplaceHandle;
    private ExtRegistrationHandle _qmHideChatTileHandle;
    private ExtRegistrationHandle _mmBannerHandle;
    private ExtRegistrationHandle _mmQuickToolHandle;
    private ExtRegistrationHandle _mmSettingsHandle;
    private IDisposable _themeOverrideHandle;

    // Interactive demo state
    private int _activeSection;
    private int _clickCount;
    private bool _accentBtnActive = true;
    private bool _toggleA = true;
    private bool _toggleB;
    private float _continuousSlider = 0.65f;
    private float _steppedSlider = 50f;
    private string _plainInput = "Hello Lumenic";
    private string _secretInput = "super-secret-token";
    private string _lastSubmitted = "(none)";
    private int _standardDropdownIdx = 1;
    private int _searchableDropdownIdx;
    private int _gridColumns = 3;
    private bool _showScrollbars = true;
    private int _textureSeed;
    private float _progressValue = 0.72f;
    private bool _modalCloseOnBackdrop = true;
    private int _themePresetIdx;

    private readonly List<string> _eventLog = new List<string>();

    public void Register()
    {
        ShowcaseTextures.RegisterAll(_textureSeed);

        LumenicApi.OnReady += HandleApiReady;
        LumenicApi.OnQuickMenuToggled += HandleQmToggled;
        LumenicApi.OnMainMenuToggled += HandleMmToggled;

        _mmTab = LumenicApi.RegisterMainMenuTab(ModId, new ExtTabOptions
        {
            Id = "showcase_mm",
            Title = "UI Showcase",
            Subtitle = "Interactive LumenicMenu.Api reference",
            Icon = "widgets",
            BadgeText = () => _clickCount > 0 ? _clickCount.ToString() : null,
            Priority = 5,
            HeaderActions = ui => BuildHeaderActions(ui, ExtSurface.MainMenu, _mmTab)
        }, ui => BuildRootPage(ui, ExtSurface.MainMenu, _mmTab));

        _qmTab = LumenicApi.RegisterQuickMenuTab(ModId, new ExtTabOptions
        {
            Id = "showcase_qm",
            Title = "Showcase",
            Subtitle = "All UiBuilder & API demos",
            Icon = "widgets",
            BadgeText = () => _clickCount > 0 ? _clickCount.ToString() : null,
            Priority = 5,
            HeaderActions = ui => BuildHeaderActions(ui, ExtSurface.QuickMenu, _qmTab)
        }, ui => BuildRootPage(ui, ExtSurface.QuickMenu, _qmTab));

        // Demonstrates grouping a utility mod inside the shared "Extensions" Quick Menu hub
        _hubEntry = LumenicApi.RegisterQuickMenuHubEntry(
            hubId: LumenicHubs.Mods,
            hubOptions: null,
            modId: ModId,
            entry: new ExtHubEntryOptions
            {
                Id = "hub_entry",
                Title = "UI Showcase",
                Subtitle = "Shared Hub entry example",
                Icon = "auto_awesome",
                BadgeText = () => ActiveOverrideCount() > 0 ? $"{ActiveOverrideCount()} ON" : "REF",
                Priority = 5
            },
            build: BuildHubEntryPage);

        LogEvent($"Registered (ApiVersion={LumenicApi.ApiVersion}, Active={LumenicApi.IsActive})");
    }

    public void Dispose()
    {
        LumenicApi.OnReady -= HandleApiReady;
        LumenicApi.OnQuickMenuToggled -= HandleQmToggled;
        LumenicApi.OnMainMenuToggled -= HandleMmToggled;

        ResetAllOverrides(markDirty: false);

        _hubEntry?.Dispose();
        _qmTab?.Dispose();
        _mmTab?.Dispose();
        _hubEntry = null;
        _qmTab = null;
        _mmTab = null;
    }

    private void RefreshAll()
    {
        _mmTab?.MarkDirty();
        _qmTab?.MarkDirty();
        _hubEntry?.MarkDirty();
        _qmBannerHandle?.MarkDirty();
        _qmActionTileHandle?.MarkDirty();
        _mmBannerHandle?.MarkDirty();
        _mmQuickToolHandle?.MarkDirty();
        _mmSettingsHandle?.MarkDirty();
    }

    private void LogEvent(string message)
    {
        string stamp = DateTime.Now.ToString("HH:mm:ss");
        _eventLog.Insert(0, $"[{stamp}] {message}");
        if (_eventLog.Count > 24)
            _eventLog.RemoveAt(_eventLog.Count - 1);
    }

    private void HandleApiReady()
    {
        LogEvent("Event: OnReady fired");
        RefreshAll();
    }

    private void HandleQmToggled(bool open)
    {
        LogEvent($"Event: OnQuickMenuToggled({open})");
        RefreshAll();
    }

    private void HandleMmToggled(bool open)
    {
        LogEvent($"Event: OnMainMenuToggled({open})");
        RefreshAll();
    }

    private int ActiveOverrideCount()
    {
        int count = 0;
        if (_qmBannerHandle != null) count++;
        if (_qmActionTileHandle != null) count++;
        if (_qmActionStyleHandle != null) count++;
        if (_qmPlayerActionHandle != null) count++;
        if (_qmNotifReplaceHandle != null) count++;
        if (_qmHideChatTileHandle != null) count++;
        if (_mmBannerHandle != null) count++;
        if (_mmQuickToolHandle != null) count++;
        if (_mmSettingsHandle != null) count++;
        if (_themeOverrideHandle != null) count++;
        return count;
    }

    private void BuildHeaderActions(UiBuilder ui, ExtSurface surface, ExtTabHandle tab)
    {
        ui.Row(r =>
        {
            r.Button("hdr_subpage", "Sub-Page", () => PushDemoSubPage(tab, 1),
                icon: "layers", tooltip: "Push a nested sub-page onto this tab's stack");
            r.IconButton("hdr_toast", "notifications_active", () =>
            {
                LumenicApi.ShowToast($"Toast from {surface} header!", 2.5f);
                LogEvent($"ShowToast from {surface}");
                RefreshAll();
            }, variant: "accent", tooltip: "Trigger a toast notification");
        }, new ExtStyle { Gap = 6f, Align = "center" });
    }

    private void BuildRootPage(UiBuilder ui, ExtSurface surface, ExtTabHandle tab)
    {
        bool compact = surface == ExtSurface.QuickMenu;

        ui.Tabs("sec_tabs", SectionTabLabels, _activeSection, idx =>
        {
            _activeSection = idx;
            RefreshAll();
        }, size: compact ? 10.5f : 11.5f);

        ui.Scroll("sec_scroll", content =>
        {
            switch (_activeSection)
            {
                case 0: BuildControlsSection(content, compact); break;
                case 1: BuildLayoutSection(content, compact); break;
                case 2: BuildMediaSection(content, compact); break;
                case 3: BuildOverlaysSection(content, surface, tab, compact); break;
                case 4: BuildSectionsAndThemeLab(content, compact); break;
            }
        }, new ExtStyle { Grow = 1f, Gap = 8f, Pr = 2f }, hideScrollbar: !_showScrollbars);
    }

    private void BuildHubEntryPage(UiBuilder ui)
    {
        ui.Card(c =>
        {
            c.Text("Shared Quick Menu Hub Entry", size: 13f, weight: "bold");
            c.Text("Registered via LumenicApi.RegisterQuickMenuHubEntry(LumenicHubs.Mods, ...). Use this for utility mods so you do not crowd the top-level tab bar.",
                size: 11f, fg: "muted", wrap: true);

            c.Row(r =>
            {
                r.Button("hub_open_qm", "Open Full QM Tab", () => _qmTab?.Open(),
                    icon: "open_in_new", variant: "accent");
                r.Button("hub_toast", "Test Toast", () => LumenicApi.ShowToast("Hello from Extensions Hub!", 2.5f),
                    icon: "campaign");
            }, new ExtStyle { Gap = 8f, Pt = 4f });
        }, new ExtStyle { Gap = 6f });

        ui.Card(c =>
        {
            c.Row(r =>
            {
                r.Text("Active Live Overrides", size: 11.5f, weight: "semibold", style: new ExtStyle { Grow = 1f });
                r.Badge(ActiveOverrideCount().ToString(), fg: ActiveOverrideCount() > 0 ? "emerald" : "muted");
            }, new ExtStyle { Align = "center" });

            if (ActiveOverrideCount() > 0)
            {
                c.Button("hub_reset", "Reset All Overrides", () => ResetAllOverrides(markDirty: true),
                    icon: "restart_alt", variant: "danger");
            }
        }, new ExtStyle { Gap = 6f });
    }

    private void PushDemoSubPage(ExtTabHandle tab, int depth)
    {
        if (tab == null)
            return;

        tab.PushSubPage(new ExtSubPageOptions
        {
            Id = $"subpage_{depth}",
            Title = $"Sub-Page Level {depth}",
            Subtitle = "Back button is rendered automatically by LumenicMenu",
            HeaderActions = h => h.Badge($"Depth {depth}", icon: "layers", fg: "accent")
        }, ui =>
        {
            ui.Card(c =>
            {
                c.Text($"Navigation Stack Level {depth}", size: 13.5f, weight: "bold");
                c.Text("Sub-pages share the tab's header and automatically restore the previous page when PopSubPage() or the built-in Back button is pressed.",
                    size: 11f, fg: "muted", wrap: true);

                c.Row(r =>
                {
                    r.Button($"push_{depth}", $"Push Level {depth + 1}", () => PushDemoSubPage(tab, depth + 1),
                        icon: "arrow_forward", variant: "accent");
                    r.Button($"pop_{depth}", "Pop Sub-Page", () => tab.PopSubPage(),
                        icon: "arrow_back");
                }, new ExtStyle { Gap = 8f, Pt = 4f });
            }, new ExtStyle { Gap = 6f });
        });
    }
}
