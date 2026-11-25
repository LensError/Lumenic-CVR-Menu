using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using LumenicMenu.Native;
using MelonLoader;
using MelonLoader.Utils;
using Newtonsoft.Json;

namespace LumenicMenu.Bridge;

/// <summary>
/// Persists user-pinned Avatars, Worlds, and Props in pin order for Quick Menu quick access.
/// </summary>
internal sealed class PinnedContentService
{
    public const int MaxPinned = 60;

    public sealed class PinnedEntry
    {
        public string Kind = string.Empty; // "avatar" | "world" | "prop"
        public string Id = string.Empty;
        public string Name = string.Empty;
        public string ImageUrl = string.Empty;
    }

    private sealed class PersistedPins
    {
        public int Version = 1;
        public List<PinnedEntry> Items = new List<PinnedEntry>();
    }

    private readonly Action _onDirty;
    private readonly object _lock = new object();
    private readonly object _fileLock = new object();
    private readonly string _filePath;

    private PersistedPins _data = new PersistedPins();
    private bool _saveScheduled;
    private volatile bool _dirtyDisk;
    private volatile int _revision;

    public int Revision => _revision;

    public PinnedContentService(Action onDirty)
    {
        _onDirty = onDirty;
        string baseDir = string.IsNullOrEmpty(MelonEnvironment.UserDataDirectory)
            ? "UserData"
            : MelonEnvironment.UserDataDirectory;
        _filePath = Path.Combine(baseDir, "LumenicMenu", "pinned-items.json");
        LoadFromDisk();
    }

    public bool TogglePin(string kind, string id, string name, string imageUrl)
    {
        if (string.IsNullOrWhiteSpace(kind) || string.IsNullOrWhiteSpace(id))
            return false;

        string normKind = NormalizeKind(kind);
        string trimId = id.Trim();
        bool pinnedNow;

        lock (_lock)
        {
            int idx = _data.Items.FindIndex(x =>
                string.Equals(x.Kind, normKind, StringComparison.OrdinalIgnoreCase)
                && string.Equals(x.Id, trimId, StringComparison.OrdinalIgnoreCase));
            if (idx >= 0)
            {
                _data.Items.RemoveAt(idx);
                pinnedNow = false;
            }
            else
            {
                if (_data.Items.Count >= MaxPinned)
                    _data.Items.RemoveAt(0);
                _data.Items.Add(new PinnedEntry
                {
                    Kind = normKind,
                    Id = trimId,
                    Name = string.IsNullOrWhiteSpace(name) ? trimId : name.Trim(),
                    ImageUrl = imageUrl?.Trim() ?? string.Empty
                });
                pinnedNow = true;
            }
            _revision++;
        }

        ScheduleSave();
        _onDirty?.Invoke();
        return pinnedNow;
    }

    public void ClearAll()
    {
        lock (_lock)
        {
            _data = new PersistedPins();
            _dirtyDisk = false;
            _revision++;
        }
        try
        {
            lock (_fileLock)
            {
                if (File.Exists(_filePath))
                    File.Delete(_filePath);
            }
        }
        catch (Exception ex)
        {
            MelonLogger.Warning($"[LumenicMenu] Failed to delete pinned-items.json: {ex.Message}");
        }
        _onDirty?.Invoke();
    }

    public void Unpin(string kind, string id)
    {
        if (string.IsNullOrWhiteSpace(kind) || string.IsNullOrWhiteSpace(id))
            return;

        string normKind = NormalizeKind(kind);
        string trimId = id.Trim();
        bool removed;
        lock (_lock)
        {
            int idx = _data.Items.FindIndex(x =>
                string.Equals(x.Kind, normKind, StringComparison.OrdinalIgnoreCase)
                && string.Equals(x.Id, trimId, StringComparison.OrdinalIgnoreCase));
            removed = idx >= 0;
            if (removed)
            {
                _data.Items.RemoveAt(idx);
                _revision++;
            }
        }

        if (removed)
        {
            ScheduleSave();
            _onDirty?.Invoke();
        }
    }

    public void PatchMetadata(string kind, string id, string name, string imageUrl)
    {
        if (string.IsNullOrWhiteSpace(kind) || string.IsNullOrWhiteSpace(id))
            return;

        string normKind = NormalizeKind(kind);
        string trimId = id.Trim();
        bool changed = false;
        lock (_lock)
        {
            for (int i = 0; i < _data.Items.Count; i++)
            {
                PinnedEntry e = _data.Items[i];
                if (!string.Equals(e.Kind, normKind, StringComparison.OrdinalIgnoreCase)
                    || !string.Equals(e.Id, trimId, StringComparison.OrdinalIgnoreCase))
                    continue;
                if (!string.IsNullOrWhiteSpace(name) && e.Name != name.Trim())
                {
                    e.Name = name.Trim();
                    changed = true;
                }
                if (!string.IsNullOrWhiteSpace(imageUrl) && e.ImageUrl != imageUrl.Trim())
                {
                    e.ImageUrl = imageUrl.Trim();
                    changed = true;
                }
            }
        }

        if (changed)
            ScheduleSave();
    }

    public List<object> BuildSnapshot(ImageLoaderService images)
    {
        List<PinnedEntry> copy;
        lock (_lock)
        {
            copy = _data.Items.Select(e => new PinnedEntry
            {
                Kind = e.Kind,
                Id = e.Id,
                Name = e.Name,
                ImageUrl = e.ImageUrl
            }).ToList();
        }

        var list = new List<object>(copy.Count);
        foreach (PinnedEntry e in copy)
        {
            list.Add(new
            {
                kind = e.Kind,
                id = e.Id,
                name = e.Name,
                image_key = images != null ? images.RequestThumbnail(NativeBridge.SurfaceQuick, e.ImageUrl) : string.Empty
            });
        }
        return list;
    }

    public void FlushSave()
    {
        if (_dirtyDisk)
            SaveToDiskSync();
    }

    private static string NormalizeKind(string kind)
    {
        string k = (kind ?? string.Empty).Trim().ToLowerInvariant();
        if (k.StartsWith("world")) return "world";
        if (k.StartsWith("avatar")) return "avatar";
        if (k.StartsWith("prop") || k.StartsWith("spawnable")) return "prop";
        return k;
    }

    private void LoadFromDisk()
    {
        try
        {
            if (!File.Exists(_filePath))
                return;
            string json = File.ReadAllText(_filePath);
            PersistedPins loaded = JsonConvert.DeserializeObject<PersistedPins>(json);
            if (loaded?.Items != null)
            {
                lock (_lock)
                {
                    _data = loaded;
                    if (_data.Items.Count > MaxPinned)
                        _data.Items.RemoveRange(MaxPinned, _data.Items.Count - MaxPinned);
                }
            }
        }
        catch (Exception ex)
        {
            MelonLogger.Warning($"[LumenicMenu] Failed to load pinned items: {ex.Message}");
        }
    }

    private void ScheduleSave()
    {
        _dirtyDisk = true;
        lock (_lock)
        {
            if (_saveScheduled)
                return;
            _saveScheduled = true;
        }

        Task.Run(async () =>
        {
            try
            {
                await Task.Delay(1200).ConfigureAwait(false);
                SaveToDiskSync();
            }
            finally
            {
                lock (_lock)
                {
                    _saveScheduled = false;
                }
                if (_dirtyDisk)
                    ScheduleSave();
            }
        });
    }

    private void SaveToDiskSync()
    {
        string json;
        lock (_lock)
        {
            _dirtyDisk = false;
            json = JsonConvert.SerializeObject(_data, Formatting.Indented);
        }

        lock (_fileLock)
        {
            try
            {
                string dir = Path.GetDirectoryName(_filePath);
                if (!string.IsNullOrEmpty(dir) && !Directory.Exists(dir))
                    Directory.CreateDirectory(dir);

                string tmp = _filePath + ".tmp";
                File.WriteAllText(tmp, json);
                if (File.Exists(_filePath))
                    File.Delete(_filePath);
                File.Move(tmp, _filePath);
            }
            catch (Exception ex)
            {
                MelonLogger.Warning($"[LumenicMenu] Failed to save pinned items: {ex.Message}");
            }
        }
    }
}
