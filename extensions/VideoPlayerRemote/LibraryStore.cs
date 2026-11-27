using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using MelonLoader.Utils;
using Newtonsoft.Json;

namespace VideoPlayerRemote
{
    [Serializable]
    internal class SavedVideoItem
    {
        public string Title = "";
        public string Url = "";

        public string DisplayTitle =>
            !string.IsNullOrWhiteSpace(Title) ? Title : ShortenUrl(Url);

        private static string ShortenUrl(string url)
        {
            if (string.IsNullOrEmpty(url))
                return "Untitled Video";
            return url.Length > 48 ? url.Substring(0, 45) + "..." : url;
        }
    }

    [Serializable]
    internal class SavedPlaylist
    {
        public string Id = Guid.NewGuid().ToString("N");
        public string Name = "New Playlist";
        public List<SavedVideoItem> Items = new List<SavedVideoItem>();
    }

    [Serializable]
    internal class VideoLibraryData
    {
        public List<SavedVideoItem> Favorites = new List<SavedVideoItem>();
        public List<SavedPlaylist> Playlists = new List<SavedPlaylist>();
    }

    internal static class LibraryStore
    {
        private static string _filePath;
        public static VideoLibraryData Data { get; private set; } = new VideoLibraryData();

        public static event Action OnLibraryChanged;

        public static void Load()
        {
            try
            {
                string dir = Path.Combine(MelonEnvironment.UserDataDirectory, "VideoPlayerRemote");
                Directory.CreateDirectory(dir);
                _filePath = Path.Combine(dir, "library.json");

                if (File.Exists(_filePath))
                {
                    string json = File.ReadAllText(_filePath);
                    var loaded = JsonConvert.DeserializeObject<VideoLibraryData>(json);
                    if (loaded != null)
                    {
                        loaded.Favorites ??= new List<SavedVideoItem>();
                        loaded.Playlists ??= new List<SavedPlaylist>();
                        foreach (var pl in loaded.Playlists)
                        {
                            pl.Items ??= new List<SavedVideoItem>();
                            if (string.IsNullOrEmpty(pl.Id))
                                pl.Id = Guid.NewGuid().ToString("N");
                        }
                        Data = loaded;
                    }
                }
            }
            catch (Exception ex)
            {
                VideoPlayerRemoteMod.Log?.Error($"Failed to load library.json: {ex.Message}");
                Data = new VideoLibraryData();
            }
        }

        public static void Save()
        {
            try
            {
                if (string.IsNullOrEmpty(_filePath))
                    return;

                string dir = Path.GetDirectoryName(_filePath);
                if (!string.IsNullOrEmpty(dir))
                    Directory.CreateDirectory(dir);

                string json = JsonConvert.SerializeObject(Data, Formatting.Indented);
                string tempPath = _filePath + ".tmp";
                File.WriteAllText(tempPath, json);
                if (File.Exists(_filePath))
                    File.Delete(_filePath);
                File.Move(tempPath, _filePath);
            }
            catch (Exception ex)
            {
                VideoPlayerRemoteMod.Log?.Error($"Failed to save library.json: {ex.Message}");
            }

            OnLibraryChanged?.Invoke();
        }

        public static bool IsFavorite(string url)
        {
            if (string.IsNullOrWhiteSpace(url))
                return false;
            return Data.Favorites.Any(f => string.Equals(f.Url, url.Trim(), StringComparison.OrdinalIgnoreCase));
        }

        public static bool AddFavorite(string title, string url)
        {
            if (string.IsNullOrWhiteSpace(url))
                return false;

            string trimmedUrl = url.Trim();
            var existing = Data.Favorites.FirstOrDefault(f =>
                string.Equals(f.Url, trimmedUrl, StringComparison.OrdinalIgnoreCase));

            if (existing != null)
            {
                if (!string.IsNullOrWhiteSpace(title) && string.IsNullOrWhiteSpace(existing.Title))
                {
                    existing.Title = title.Trim();
                    Save();
                }
                return false;
            }

            Data.Favorites.Insert(0, new SavedVideoItem
            {
                Title = string.IsNullOrWhiteSpace(title) ? trimmedUrl : title.Trim(),
                Url = trimmedUrl
            });
            Save();
            return true;
        }

        public static void RemoveFavorite(string url)
        {
            if (string.IsNullOrWhiteSpace(url))
                return;

            int removed = Data.Favorites.RemoveAll(f =>
                string.Equals(f.Url, url.Trim(), StringComparison.OrdinalIgnoreCase));
            if (removed > 0)
                Save();
        }

        public static SavedPlaylist CreatePlaylist(string name)
        {
            string cleanName = string.IsNullOrWhiteSpace(name) ? $"Playlist {Data.Playlists.Count + 1}" : name.Trim();
            var playlist = new SavedPlaylist
            {
                Id = Guid.NewGuid().ToString("N"),
                Name = cleanName
            };
            Data.Playlists.Add(playlist);
            Save();
            return playlist;
        }

        public static void RenamePlaylist(SavedPlaylist playlist, string newName)
        {
            if (playlist == null || string.IsNullOrWhiteSpace(newName))
                return;
            playlist.Name = newName.Trim();
            Save();
        }

        public static void Shutdown()
        {
            OnLibraryChanged = null;
        }

        public static void DeletePlaylist(SavedPlaylist playlist)
        {
            if (playlist == null)
                return;
            if (Data.Playlists.Remove(playlist))
            {
                VideoPlayerTracker.HandlePlaylistDeleted(playlist);
                Save();
            }
        }

        public static void AddToPlaylist(SavedPlaylist playlist, string title, string url)
        {
            if (playlist == null || string.IsNullOrWhiteSpace(url))
                return;
            playlist.Items.Add(new SavedVideoItem
            {
                Title = string.IsNullOrWhiteSpace(title) ? url.Trim() : title.Trim(),
                Url = url.Trim()
            });
            Save();
        }

        public static void RemoveFromPlaylist(SavedPlaylist playlist, int index)
        {
            if (playlist == null || index < 0 || index >= playlist.Items.Count)
                return;
            playlist.Items.RemoveAt(index);
            VideoPlayerTracker.HandlePlaylistItemRemoved(playlist, index);
            Save();
        }

        public static void MoveInPlaylist(SavedPlaylist playlist, int index, int delta)
        {
            if (playlist == null || index < 0 || index >= playlist.Items.Count)
                return;
            int target = index + delta;
            if (target < 0 || target >= playlist.Items.Count)
                return;

            var item = playlist.Items[index];
            playlist.Items.RemoveAt(index);
            playlist.Items.Insert(target, item);
            VideoPlayerTracker.HandlePlaylistItemMoved(playlist, index, target);
            Save();
        }
    }
}
