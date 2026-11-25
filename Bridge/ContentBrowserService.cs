using System;
using System.Collections.Generic;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using ABI_RC.Core.EventSystem;
using ABI_RC.Core.InteractionSystem;
using ABI_RC.Core.Networking;
using ABI_RC.Core.Networking.API;
using ABI_RC.Core.Networking.API.Responses;
using ABI_RC.Core.Networking.API.Responses.CategoriesV2;
using ABI_RC.Core.Networking.API.Responses.DetailsV2;
using ABI_RC.Core.Networking.API.Responses.Lists.ContentLists;
using ABI_RC.Core.Networking.IO.Global;
using ABI_RC.Core.Networking.IO.Instancing;
using ABI_RC.Core.Player;
using ABI_RC.Core.Savior;
using ABI_RC.Core.UI;
using HarmonyLib;
using LumenicMenu.Native;
using MelonLoader;

namespace LumenicMenu.Bridge;

/// <summary>
/// Asynchronous ChilloutVR content browser service for Phase 3B (Worlds, Avatars, Props, Global Search).
/// Uses CVR's official ApiConnection.MakeRequest<T> pipeline off the main thread and marshals
/// read-only snapshots back to Rust on the Unity main thread.
/// </summary>
internal sealed class ContentBrowserService
{
    private static readonly FieldInfo ViewManagerCategoriesField =
        AccessTools.Field(typeof(ViewManager), "_categories");
    private sealed class CategoryDto
    {
        public string Key;
        public string Name;
        public bool IsSystem;
        public bool IsRecent;
    }

    private sealed class CardDto
    {
        public string Id;
        public string Name;
        public string ImageUrl;
        public int PlayerCount;
    }

    private sealed class SearchCardDto
    {
        public string Id;
        public string Name;
        public string Kind;
        public string ImageUrl;
    }

    private sealed class InstanceDto
    {
        public string Id;
        public string Name;
        public int PlayerCount;
        public int MaxPlayers;
        public string Region;
        public List<InstanceMemberDto> Members = new List<InstanceMemberDto>();
    }

    private sealed class ListState
    {
        public string CategoryKey = string.Empty;
        public bool IsSystem = true;
        public int Page;
        public int TotalPages = 1;
        public int SortOrder;
        public bool Ascending;
        public bool Loading;
        public bool LoadingMore;
        public bool LoadedOnce;
        public int ListRevision;
        public string Error = string.Empty;
        public List<CardDto> Items = new List<CardDto>();
        public int RequestSeq;
        public DateTime LastFetchedUtc = DateTime.MinValue;
    }

    private sealed class DetailState
    {
        public string Kind = string.Empty;
        public string Id = string.Empty;
        public string Name = string.Empty;
        public string Description = string.Empty;
        public string AuthorId = string.Empty;
        public string AuthorName = string.Empty;
        public string AuthorImageUrl = string.Empty;
        public string ImageUrl = string.Empty;
        public string PanoUrl = string.Empty;
        public string UploadedAt = string.Empty;
        public string UpdatedAt = string.Empty;
        public string FileSizeText = string.Empty;
        public bool IsPublic;
        public bool IsShared;
        public bool CanUse = true;
        public bool Loading;
        public string Error = string.Empty;
        public List<string> Platforms = new List<string>();
        public List<string> Categories = new List<string>();
        public List<InstanceDto> Instances = new List<InstanceDto>();
    }

    private readonly Action _onDirty;
    private readonly RecentHistoryService _recentHistory;
    private readonly PinnedContentService _pinnedContent;
    private readonly object _lock = new object();

    private bool _categoriesLoading;
    private bool _categoriesLoaded;
    private List<CategoryDto> _worldCategories = new List<CategoryDto>();
    private List<CategoryDto> _avatarCategories = new List<CategoryDto>();
    private List<CategoryDto> _propCategories = new List<CategoryDto>();

    private readonly ListState _worlds = new ListState();
    private readonly ListState _avatars = new ListState();
    private readonly ListState _props = new ListState();

    private string _searchQuery = string.Empty;
    private bool _searchLoading;
    private string _searchError = string.Empty;
    private int _searchSeq;
    private List<SearchCardDto> _searchResults = new List<SearchCardDto>();

    private sealed class InstanceMemberDto
    {
        public string Id;
        public string Name;
        public string ImageUrl;
    }

    private sealed class InstanceDetailState
    {
        public string Id = string.Empty;
        public string Name = string.Empty;
        public string WorldId = string.Empty;
        public string WorldName = string.Empty;
        public string WorldImageUrl = string.Empty;
        public string WorldAuthorId = string.Empty;
        public string WorldAuthorName = string.Empty;
        public string Privacy = string.Empty;
        public string Region = string.Empty;
        public string GameModeName = string.Empty;
        public string OwnerId = string.Empty;
        public string OwnerName = string.Empty;
        public string OwnerRank = string.Empty;
        public string OwnerImageUrl = string.Empty;
        public string GroupId = string.Empty;
        public string GroupName = string.Empty;
        public string GroupImageUrl = string.Empty;
        public bool GroupVerified;
        public bool CanJoin = true;
        public string CannotJoinReason = string.Empty;
        public int PlayerCount;
        public int MaxPlayers;
        public bool Loading;
        public string Error = string.Empty;
        public List<InstanceMemberDto> Members = new List<InstanceMemberDto>();
    }

    private DetailState _detail;
    private int _detailSeq;

    private InstanceDetailState _instanceDetail;
    private int _instanceDetailSeq;

    private bool _creatingInstance;
    private string _createInstanceError = string.Empty;
    private GroupModPromptState _groupModPrompt;
    private TaskCompletionSource<bool> _groupModPromptTcs;
    private volatile bool _qmDirty = true;
    private int _lastPinnedRev = -1;

    public bool ConsumeQmDirty()
    {
        int pinnedRev = _pinnedContent != null ? _pinnedContent.Revision : 0;
        bool dirty = _qmDirty || pinnedRev != _lastPinnedRev;
        _qmDirty = false;
        _lastPinnedRev = pinnedRev;
        return dirty;
    }

    private bool _discoverLoading;
    private bool _discoverLoaded;
    private string _discoverError = string.Empty;
    private List<CardDto> _discoverWorlds = new List<CardDto>();
    private List<CardDto> _discoverAvatars = new List<CardDto>();
    private List<CardDto> _discoverProps = new List<CardDto>();
    private string _pendingJoinInstanceId;

    public ContentBrowserService(
        Action onDirty,
        RecentHistoryService recentHistory = null,
        PinnedContentService pinnedContent = null)
    {
        _onDirty = onDirty;
        _recentHistory = recentHistory;
        _pinnedContent = pinnedContent;
    }

    private bool TryHydrateCategoriesFromViewManager()
    {
        try
        {
            if (ViewManager.Instance == null || ViewManagerCategoriesField == null)
                return false;
            if (ViewManagerCategoriesField.GetValue(ViewManager.Instance) is not Categories_t cats)
                return false;
            if (cats.Worlds.Count == 0 && cats.Avatars.Count == 0 && cats.Spawnables.Count == 0)
                return false;

            lock (_lock)
            {
                _worldCategories = cats.Worlds
                    .Select(c => new CategoryDto { Key = c.CategoryKey ?? string.Empty, Name = c.CategoryClearTextName ?? c.CategoryKey ?? string.Empty, IsSystem = c.IsSystemCategory })
                    .Where(c => !string.IsNullOrEmpty(c.Key))
                    .ToList();
                _avatarCategories = cats.Avatars
                    .Select(c => new CategoryDto { Key = c.CategoryKey ?? string.Empty, Name = c.CategoryClearTextName ?? c.CategoryKey ?? string.Empty, IsSystem = c.IsSystemCategory })
                    .Where(c => !string.IsNullOrEmpty(c.Key))
                    .ToList();
                _propCategories = cats.Spawnables
                    .Select(c => new CategoryDto { Key = c.CategoryKey ?? string.Empty, Name = c.CategoryClearTextName ?? c.CategoryKey ?? string.Empty, IsSystem = c.IsSystemCategory })
                    .Where(c => !string.IsNullOrEmpty(c.Key))
                    .ToList();
                _categoriesLoaded = _worldCategories.Count > 0 || _avatarCategories.Count > 0 || _propCategories.Count > 0;
                return _categoriesLoaded;
            }
        }
        catch
        {
            return false;
        }
    }

    public void EnsureCategoriesAndLoad(string kind)
    {
        bool needCats;
        lock (_lock)
        {
            needCats = !_categoriesLoaded && !_categoriesLoading;
        }

        if (needCats && TryHydrateCategoriesFromViewManager())
        {
            needCats = false;
            _onDirty?.Invoke();
        }

        if (needCats)
        {
            lock (_lock)
            {
                if (_categoriesLoading) return;
                _categoriesLoading = true;
            }
            Task.Run(() => FetchCategoriesAsync(kind));
            return;
        }

        ListState st = GetListState(kind);
        if (st == null)
            return;

        string catKey;
        bool isSys;
        int page;
        int sort;
        bool asc;
        lock (_lock)
        {
            if (st.Loading)
                return;
            if (string.IsNullOrEmpty(st.CategoryKey))
            {
                CategoryDto defCat = PickDefaultCategory(kind, GetCategoriesFor(kind));
                if (defCat != null)
                {
                    st.CategoryKey = defCat.Key;
                    st.IsSystem = defCat.IsSystem;
                }
            }
            catKey = st.CategoryKey;
            isSys = st.IsSystem;
            page = st.Page;
            sort = st.SortOrder;
            asc = st.Ascending;
        }

        if (!string.IsNullOrEmpty(catKey))
        {
            LoadList(kind, catKey, isSys, page, sort, asc);
        }
    }

    public void LoadList(string kind, string categoryKey, bool isSystem, int page, int sortOrder, bool ascending)
    {
        ListState st = GetListState(kind);
        if (st == null)
            return;

        if (string.IsNullOrWhiteSpace(categoryKey))
        {
            EnsureCategoriesAndLoad(kind);
            return;
        }

        int clampedSort = Math.Max(0, Math.Min(3, sortOrder));
        if (RecentHistoryService.IsLocalRecentCategory(categoryKey))
        {
            List<CardDto> localItems = BuildLocalRecentCards(categoryKey, clampedSort, ascending);
            lock (_lock)
            {
                st.CategoryKey = categoryKey;
                st.IsSystem = true;
                st.Page = 0;
                st.TotalPages = 1;
                st.SortOrder = clampedSort;
                st.Ascending = ascending;
                st.Loading = false;
                st.LoadingMore = false;
                st.LoadedOnce = true;
                st.Error = string.Empty;
                st.Items = localItems;
                st.ListRevision++;
                st.RequestSeq++;
            }
            _onDirty?.Invoke();
            _recentHistory?.RequestHydrateCategory(categoryKey, () => RefreshLocalRecentIfActive(kind));
            return;
        }

        if (!AuthManager.IsAuthenticated)
            return;

        int seq;
        int clampedPage = Math.Max(0, page);
        lock (_lock)
        {
            st.CategoryKey = categoryKey;
            st.IsSystem = isSystem;
            st.Page = clampedPage;
            st.SortOrder = clampedSort;
            st.Ascending = ascending;
            st.Loading = true;
            st.LoadingMore = false;
            st.Error = string.Empty;
            st.ListRevision++;
            seq = ++st.RequestSeq;
        }
        _onDirty?.Invoke();

        Task.Run(() => FetchListAsync(kind, categoryKey, isSystem, clampedPage, (ContentOrder)clampedSort, ascending, seq, append: false));
    }

    public void RefreshIfStale(string kind)
    {
        ListState st = GetListState(kind);
        if (st == null)
            return;

        string categoryKey;
        bool isSystem;
        int sortOrder;
        bool ascending;
        int seq;
        lock (_lock)
        {
            if (RecentHistoryService.IsLocalRecentCategory(st.CategoryKey))
            {
                categoryKey = st.CategoryKey;
                isSystem = true;
                sortOrder = st.SortOrder;
                ascending = st.Ascending;
                seq = -1;
            }
            else
            {
                if (!AuthManager.IsAuthenticated || st.Loading || st.LoadingMore || !st.LoadedOnce || st.Page > 0)
                    return;
                if (string.IsNullOrWhiteSpace(st.CategoryKey))
                    return;

                double cooldownSecs = string.Equals(kind, "worlds", StringComparison.OrdinalIgnoreCase) ? 45.0 : 120.0;
                if ((DateTime.UtcNow - st.LastFetchedUtc).TotalSeconds < cooldownSecs)
                    return;

                categoryKey = st.CategoryKey;
                isSystem = st.IsSystem;
                sortOrder = st.SortOrder;
                ascending = st.Ascending;
                st.Loading = true;
                st.Error = string.Empty;
                seq = ++st.RequestSeq;
            }
        }

        if (seq < 0)
        {
            RefreshLocalRecentIfActive(kind);
            return;
        }

        _onDirty?.Invoke();
        Task.Run(() => FetchListAsync(kind, categoryKey, isSystem, 0, (ContentOrder)sortOrder, ascending, seq, append: false));
    }

    public void LoadMore(string kind)
    {
        if (!AuthManager.IsAuthenticated)
            return;

        ListState st = GetListState(kind);
        if (st == null)
            return;

        string categoryKey;
        bool isSystem;
        int nextPage;
        int sortOrder;
        bool ascending;
        int seq;
        lock (_lock)
        {
            if (st.Loading || st.LoadingMore)
                return;
            if (string.IsNullOrWhiteSpace(st.CategoryKey) || RecentHistoryService.IsLocalRecentCategory(st.CategoryKey))
                return;
            if (st.Page + 1 >= Math.Max(1, st.TotalPages))
                return;

            categoryKey = st.CategoryKey;
            isSystem = st.IsSystem;
            nextPage = st.Page + 1;
            sortOrder = st.SortOrder;
            ascending = st.Ascending;
            st.LoadingMore = true;
            st.Error = string.Empty;
            seq = st.RequestSeq;
        }
        _onDirty?.Invoke();

        Task.Run(() => FetchListAsync(kind, categoryKey, isSystem, nextPage, (ContentOrder)sortOrder, ascending, seq, append: true));
    }

    public void RefreshLocalRecentIfActive(string kind)
    {
        ListState st = GetListState(kind);
        if (st == null)
            return;

        string categoryKey;
        int sortOrder;
        bool ascending;
        lock (_lock)
        {
            if (!RecentHistoryService.IsLocalRecentCategory(st.CategoryKey))
                return;
            categoryKey = st.CategoryKey;
            sortOrder = st.SortOrder;
            ascending = st.Ascending;
        }

        List<CardDto> localItems = BuildLocalRecentCards(categoryKey, sortOrder, ascending);
        lock (_lock)
        {
            if (st.CategoryKey != categoryKey)
                return;
            st.Items = localItems;
            st.Page = 0;
            st.TotalPages = 1;
            st.Loading = false;
            st.LoadingMore = false;
            st.LoadedOnce = true;
            st.Error = string.Empty;
        }
        _onDirty?.Invoke();
    }

    private List<CardDto> BuildLocalRecentCards(string categoryKey, int sortOrder, bool ascending)
    {
        if (_recentHistory == null)
            return new List<CardDto>();

        List<RecentHistoryService.RecentContentEntry> entries = _recentHistory.GetContentEntries(categoryKey);
        if (sortOrder == 2)
        {
            entries.Sort((a, b) => ascending
                ? string.Compare(a.Name, b.Name, StringComparison.OrdinalIgnoreCase)
                : string.Compare(b.Name, a.Name, StringComparison.OrdinalIgnoreCase));
        }
        else if (ascending)
        {
            entries.Reverse();
        }

        return entries.Select(e => new CardDto
        {
            Id = e.Id,
            Name = e.Name,
            ImageUrl = e.ImageUrl,
            PlayerCount = 0
        }).ToList();
    }

    public void LoadDetail(string kind, string id)
    {
        if (!AuthManager.IsAuthenticated || string.IsNullOrWhiteSpace(kind) || string.IsNullOrWhiteSpace(id))
            return;

        int seq;
        lock (_lock)
        {
            seq = ++_detailSeq;
            _detail = new DetailState
            {
                Kind = kind,
                Id = id,
                Loading = true
            };
        }
        _onDirty?.Invoke();

        Task.Run(() => FetchDetailAsync(kind, id, seq));
    }

    public void CloseDetail()
    {
        lock (_lock)
        {
            _detailSeq++;
            _detail = null;
            _instanceDetailSeq++;
            _instanceDetail = null;
            _createInstanceError = string.Empty;
        }
        _onDirty?.Invoke();
    }

    public void LoadInstanceDetail(string instanceId)
    {
        if (!AuthManager.IsAuthenticated || string.IsNullOrWhiteSpace(instanceId))
            return;

        int seq;
        lock (_lock)
        {
            seq = ++_instanceDetailSeq;
            _instanceDetail = new InstanceDetailState
            {
                Id = instanceId,
                Loading = true
            };
        }
        _onDirty?.Invoke();

        Task.Run(() => FetchInstanceDetailAsync(instanceId, seq));
    }

    public void CloseInstanceDetail()
    {
        lock (_lock)
        {
            _instanceDetailSeq++;
            _instanceDetail = null;
        }
        _onDirty?.Invoke();
    }

    private async Task FetchInstanceDetailAsync(string instanceId, int seq)
    {
        try
        {
            BaseResponse<InstanceDetailsResponse> resp =
                await ApiConnection.MakeRequest<InstanceDetailsResponse>(
                    ApiConnection.ApiOperation.InstanceDetail,
                    new { instanceID = instanceId });

            var next = new InstanceDetailState
            {
                Id = instanceId,
                Loading = false
            };

            if (resp?.Data != null)
            {
                InstanceDetailsResponse d = resp.Data;
                next.Name = d.Name ?? "Instance";
                next.WorldId = d.World?.Id ?? string.Empty;
                next.WorldName = d.World?.Name ?? string.Empty;
                next.WorldImageUrl = d.World?.ImageUrl ?? string.Empty;
                next.WorldAuthorId = d.Author?.Id ?? string.Empty;
                next.WorldAuthorName = d.Author?.Name ?? string.Empty;

                string prettyPrivacy = string.Empty;
                try
                {
                    if (!string.IsNullOrWhiteSpace(d.InstanceSettingPrivacy))
                    {
                        prettyPrivacy = Instances.GetPrettyInstanceName(Instances.GetPrivacy(d.InstanceSettingPrivacy));
                    }
                }
                catch
                {
                    // Fallback to raw privacy string below.
                }
                next.Privacy = !string.IsNullOrWhiteSpace(prettyPrivacy) ? prettyPrivacy : (d.InstanceSettingPrivacy ?? string.Empty);
                next.Region = d.Region ?? string.Empty;
                next.GameModeName = d.GameModeName ?? string.Empty;

                next.OwnerId = d.Owner?.Id ?? d.Author?.Id ?? string.Empty;
                next.OwnerName = d.Owner?.Name ?? d.Author?.Name ?? string.Empty;
                next.OwnerRank = d.Owner?.Rank ?? string.Empty;
                next.OwnerImageUrl = d.Owner?.ImageUrl ?? string.Empty;

                if (d.Group != null)
                {
                    next.GroupId = d.Group.Id.ToString();
                    next.GroupName = d.Group.Name ?? string.Empty;
                    next.GroupImageUrl = d.Group.Image?.ToString() ?? string.Empty;
                    next.GroupVerified = d.Group.IsVerified;
                }

                try
                {
                    if (d.Requirements != null && GameServerTags.IsJoinBlocked(d.Requirements, out string reason))
                    {
                        next.CanJoin = false;
                        next.CannotJoinReason = reason ?? "Cannot join this instance.";
                    }
                }
                catch
                {
                    next.CanJoin = true;
                }

                next.PlayerCount = d.CurrentPlayerCount;
                next.MaxPlayers = d.MaxPlayer;
                if (d.Members != null)
                {
                    foreach (UserDetails m in d.Members)
                    {
                        if (m == null || string.IsNullOrEmpty(m.Id)) continue;
                        next.Members.Add(new InstanceMemberDto
                        {
                            Id = m.Id,
                            Name = m.Name ?? "Player",
                            ImageUrl = m.ImageUrl ?? string.Empty
                        });
                    }
                }
            }
            else
            {
                next.Error = resp?.Message ?? "Failed to load instance details.";
            }

            lock (_lock)
            {
                if (_instanceDetailSeq == seq)
                    _instanceDetail = next;
            }
            _onDirty?.Invoke();
        }
        catch (Exception ex)
        {
            lock (_lock)
            {
                if (_instanceDetailSeq == seq && _instanceDetail != null)
                {
                    _instanceDetail.Loading = false;
                    _instanceDetail.Error = ex.Message;
                }
            }
            _onDirty?.Invoke();
        }
    }

    public void RunGlobalSearch(string query)
    {
        if (!AuthManager.IsAuthenticated)
            return;

        string trimmed = (query ?? string.Empty).Trim();
        int seq;
        bool cleared = false;
        lock (_lock)
        {
            _searchQuery = trimmed;
            _searchError = string.Empty;
            if (string.IsNullOrEmpty(trimmed))
            {
                _searchResults.Clear();
                _searchLoading = false;
                cleared = true;
                seq = _searchSeq;
            }
            else
            {
                _searchLoading = true;
                seq = ++_searchSeq;
            }
        }
        _onDirty?.Invoke();
        if (cleared)
            return;

        bool mature = false;
        try
        {
            mature = MetaPort.Instance?.settings?.GetSettingsBool("IncludeNSFWInSearch") ?? false;
        }
        catch
        {
            // Ignore settings read failure.
        }

        Task.Run(() => FetchSearchAsync(trimmed, mature, seq));
    }

    public void WearAvatar(string avatarId)
    {
        if (!string.IsNullOrWhiteSpace(avatarId))
        {
            AssetManagement.Instance?.LoadLocalAvatar(avatarId);
        }
    }

    public void SpawnProp(string propId)
    {
        if (string.IsNullOrWhiteSpace(propId))
            return;

        string propName = "Prop";
        string propImage = string.Empty;
        lock (_lock)
        {
            if (_detail != null && _detail.Kind == "prop" && string.Equals(_detail.Id, propId, StringComparison.OrdinalIgnoreCase))
            {
                if (!string.IsNullOrWhiteSpace(_detail.Name)) propName = _detail.Name;
                if (!string.IsNullOrWhiteSpace(_detail.ImageUrl)) propImage = _detail.ImageUrl;
            }
            else
            {
                CardDto card = _props.Items.FirstOrDefault(x => string.Equals(x.Id, propId, StringComparison.OrdinalIgnoreCase))
                    ?? _discoverProps.FirstOrDefault(x => string.Equals(x.Id, propId, StringComparison.OrdinalIgnoreCase));
                if (card != null)
                {
                    if (!string.IsNullOrWhiteSpace(card.Name)) propName = card.Name;
                    if (!string.IsNullOrWhiteSpace(card.ImageUrl)) propImage = card.ImageUrl;
                }
            }
        }

        if (PlayerSetup.Instance != null)
        {
            PlayerSetup.Instance.SelectPropToSpawn(propId, propImage, propName);
            CohtmlHud.Instance?.SelectPropToSpawn(
                propImage,
                propName,
                "Place to spawn · Right-Click (Desktop) or Grip (VR) to cancel");
        }

        try
        {
            if (CVR_MenuManager.Instance != null && CVR_MenuManager.Instance.IsViewShown)
                CVR_MenuManager.Instance.ToggleQuickMenu(show: false);
            if (ViewManager.Instance != null && ViewManager.Instance.IsViewShown)
                ViewManager.Instance.UiStateToggle(show: false);
        }
        catch
        {
            // Ignore if ViewManager is not ready.
        }
    }

    public void TogglePin(string kind, string id)
    {
        if (_pinnedContent == null || string.IsNullOrWhiteSpace(kind) || string.IsNullOrWhiteSpace(id))
            return;

        string name = string.Empty;
        string imageUrl = string.Empty;
        lock (_lock)
        {
            if (_detail != null && string.Equals(_detail.Id, id, StringComparison.OrdinalIgnoreCase))
            {
                name = _detail.Name;
                imageUrl = _detail.ImageUrl;
            }
            if (string.IsNullOrWhiteSpace(name) || string.IsNullOrWhiteSpace(imageUrl))
            {
                ListState st = GetListState(kind == "world" ? "worlds" : kind == "avatar" ? "avatars" : kind == "prop" ? "props" : kind);
                CardDto c = st?.Items.FirstOrDefault(x => string.Equals(x.Id, id, StringComparison.OrdinalIgnoreCase));
                if (c != null)
                {
                    if (string.IsNullOrWhiteSpace(name)) name = c.Name;
                    if (string.IsNullOrWhiteSpace(imageUrl)) imageUrl = c.ImageUrl;
                }
            }
        }

        _pinnedContent.TogglePin(kind, id, name, imageUrl);
    }

    public void Unpin(string kind, string id)
    {
        _pinnedContent?.Unpin(kind, id);
    }

    public void JoinInstance(string instanceId)
    {
        if (!string.IsNullOrWhiteSpace(instanceId))
        {
            Instances.TryJoinInstance(instanceId, Instances.JoinInstanceSource.Mod);
        }
    }

    public void DropPortal(string instanceId)
    {
        if (!string.IsNullOrWhiteSpace(instanceId))
        {
            PlayerSetup.Instance?.DropPortal(instanceId);
        }
    }

    public void ToggleContentCategory(string kind, string id, string categoryKey)
    {
        if (!AuthManager.IsAuthenticated
            || string.IsNullOrWhiteSpace(kind)
            || string.IsNullOrWhiteSpace(id)
            || string.IsNullOrWhiteSpace(categoryKey))
            return;

        string[] nextCats;
        lock (_lock)
        {
            if (_detail == null || !string.Equals(_detail.Id, id, StringComparison.OrdinalIgnoreCase))
                return;

            var list = new List<string>(_detail.Categories);
            int idx = list.FindIndex(c => string.Equals(c, categoryKey, StringComparison.OrdinalIgnoreCase));
            if (idx >= 0)
                list.RemoveAt(idx);
            else
                list.Add(categoryKey);

            _detail.Categories = list;
            nextCats = list.ToArray();
        }
        _onDirty?.Invoke();

        ApiConnection.ApiOperation op = kind switch
        {
            "avatar" => ApiConnection.ApiOperation.AvatarCategorySet,
            "prop" => ApiConnection.ApiOperation.PropCategorySet,
            _ => ApiConnection.ApiOperation.WorldCategorySet
        };

        Task.Run(async () =>
        {
            try
            {
                await ApiConnection.MakeRequest<BaseResponse<object>>(op, new { Id = id, Categories = nextCats });
            }
            catch (Exception ex)
            {
                MelonLogger.Warning($"[LumenicMenu] ToggleContentCategory({kind}, {id}, {categoryKey}) failed: {ex.Message}");
            }
        });
    }

    public void FlushMainThreadActions()
    {
        string joinId;
        lock (_lock)
        {
            joinId = _pendingJoinInstanceId;
            _pendingJoinInstanceId = null;
        }
        if (!string.IsNullOrEmpty(joinId))
        {
            Instances.TryJoinInstance(joinId, Instances.JoinInstanceSource.Mod);
        }
    }

    private static int ParseRegionCode(string region)
    {
        if (string.IsNullOrWhiteSpace(region))
            return 0;
        if (int.TryParse(region, out int numeric) && numeric >= 0 && numeric <= 2)
            return numeric;
        if (string.Equals(region, "Us", StringComparison.OrdinalIgnoreCase))
            return 1;
        if (string.Equals(region, "As", StringComparison.OrdinalIgnoreCase))
            return 2;
        return 0;
    }

    public void CreateInstance(string worldId, string privacy, string region, string groupId = null)
    {
        if (!AuthManager.IsAuthenticated || string.IsNullOrWhiteSpace(worldId))
            return;

        string safePrivacy = string.IsNullOrWhiteSpace(privacy) ? "Friends" : privacy;
        bool isGroupPrivacy =
            string.Equals(safePrivacy, "Group", StringComparison.OrdinalIgnoreCase)
            || string.Equals(safePrivacy, "GroupPlus", StringComparison.OrdinalIgnoreCase)
            || string.Equals(safePrivacy, "GroupPublic", StringComparison.OrdinalIgnoreCase);

        if (isGroupPrivacy && string.IsNullOrWhiteSpace(groupId))
        {
            lock (_lock)
            {
                _creatingInstance = false;
                _createInstanceError = "Select a group first.";
                _qmDirty = true;
            }
            _onDirty?.Invoke();
            return;
        }

        lock (_lock)
        {
            if (_creatingInstance)
                return;
            _creatingInstance = true;
            _createInstanceError = string.Empty;
            _qmDirty = true;
        }
        _onDirty?.Invoke();

        int regionInt = ParseRegionCode(region);
        string safeGroupId = groupId ?? string.Empty;
        Task.Run(async () =>
        {
            try
            {
                object payload = isGroupPrivacy
                    ? (object)new { worldId, privacy = safePrivacy, region = regionInt, Group = safeGroupId, engine = "Citadel" }
                    : new { worldId, privacy = safePrivacy, region = regionInt, engine = "Citadel" };

                BaseResponse<InstanceCreateResponse> resp =
                    await ApiConnection.MakeRequest<InstanceCreateResponse>(
                        ApiConnection.ApiOperation.InstanceCreate,
                        payload);
                if (resp?.Data == null || string.IsNullOrEmpty(resp.Data.Id))
                {
                    string err = resp?.Message ?? "Failed to create instance.";
                    MelonLogger.Warning($"[LumenicMenu] CreateInstance failed (status={resp?.HttpStatusCode}, msg={err}).");
                    lock (_lock)
                    {
                        _creatingInstance = false;
                        _createInstanceError = err;
                        _qmDirty = true;
                    }
                    _onDirty?.Invoke();
                    return;
                }

                string newId = resp.Data.Id;
                string newName = resp.Data.Name ?? "New Instance";
                string newRegion = resp.Data.Region ?? region;

                // Poll InstanceDetail until CVR's game server finishes spinning up (mirrors ViewManager.CreateInstanceCoroutine).
                bool ready = false;
                for (int attempt = 0; attempt < 40; attempt++)
                {
                    await Task.Delay(500).ConfigureAwait(false);
                    try
                    {
                        BaseResponse<InstanceDetailsResponse> check =
                            await ApiConnection.MakeRequest<InstanceDetailsResponse>(
                                ApiConnection.ApiOperation.InstanceDetail,
                                new { instanceID = newId });
                        if (check?.Data != null)
                        {
                            ready = true;
                            break;
                        }
                    }
                    catch
                    {
                        // Keep polling until ready or timeout.
                    }
                }

                lock (_lock)
                {
                    _creatingInstance = false;
                    _qmDirty = true;
                    if (ready)
                    {
                        _createInstanceError = string.Empty;
                        _pendingJoinInstanceId = newId;
                        if (_detail != null && _detail.Kind == "world" && string.Equals(_detail.Id, worldId, StringComparison.OrdinalIgnoreCase))
                        {
                            if (!_detail.Instances.Any(i => string.Equals(i.Id, newId, StringComparison.OrdinalIgnoreCase)))
                            {
                                _detail.Instances.Insert(0, new InstanceDto
                                {
                                    Id = newId,
                                    Name = newName,
                                    PlayerCount = 0,
                                    MaxPlayers = 100,
                                    Region = newRegion
                                });
                            }
                        }
                    }
                    else
                    {
                        _createInstanceError = "Instance creation timed out waiting for server.";
                    }
                }
                _onDirty?.Invoke();
            }
            catch (Exception ex)
            {
                MelonLogger.Warning($"[LumenicMenu] CreateInstance error: {ex.Message}");
                lock (_lock)
                {
                    _creatingInstance = false;
                    _qmDirty = true;
                    _createInstanceError = ex.Message;
                }
                _onDirty?.Invoke();
            }
        });
    }

    public Task<bool> ShowGroupModPrompt(InstanceDetailsResponse details)
    {
        if (details == null)
            return Task.FromResult(false);

        TaskCompletionSource<bool> prevTcs;
        TaskCompletionSource<bool> nextTcs = new TaskCompletionSource<bool>();
        lock (_lock)
        {
            prevTcs = _groupModPromptTcs;
            _groupModPromptTcs = nextTcs;
            _qmDirty = true;
            _groupModPrompt = new GroupModPromptState
            {
                InstanceId = details.Id ?? string.Empty,
                InstanceName = details.Name ?? string.Empty,
                WorldName = details.World?.Name ?? string.Empty,
                GroupId = details.Group?.Id.ToString() ?? string.Empty,
                GroupName = details.Group?.Name ?? "Group Instance",
                Privacy = details.InstanceSettingPrivacy ?? string.Empty,
                IsVerified = details.Group?.IsVerified ?? false,
                ModeratorCount = details.Group?.Moderators?.Count ?? 0
            };
        }
        prevTcs?.TrySetResult(false);
        _onDirty?.Invoke();
        return nextTcs.Task;
    }

    public void ResolveGroupModPrompt(bool accept, bool neverShow = false)
    {
        if (neverShow && MetaPort.Instance?.settings != null)
        {
            MetaPort.Instance.settings.SetSettingsBool("GeneralUIGroupModerationNeverShow", false);
            MetaPort.Instance.settings.settingsHaveChanged = true;
        }

        TaskCompletionSource<bool> tcs;
        lock (_lock)
        {
            tcs = _groupModPromptTcs;
            _groupModPromptTcs = null;
            _groupModPrompt = null;
            _qmDirty = true;
        }
        tcs?.TrySetResult(accept);
        _onDirty?.Invoke();
    }

    public void CancelGroupModPrompt()
    {
        ResolveGroupModPrompt(false, false);
    }

    public void SetHomeWorld(string worldId)
    {
        if (!string.IsNullOrWhiteSpace(worldId))
        {
            ViewManager.Instance?.SetHomeWorld(worldId);
        }
    }

    public void LoadDiscover()
    {
        if (!AuthManager.IsAuthenticated)
        {
            lock (_lock)
            {
                _discoverError = "Sign in to ChilloutVR to load Discover.";
            }
            _onDirty?.Invoke();
            return;
        }

        lock (_lock)
        {
            if (_discoverLoading)
                return;
            _discoverLoading = true;
            _discoverError = string.Empty;
        }
        _onDirty?.Invoke();

        bool mature = false;
        try
        {
            mature = MetaPort.Instance?.settings?.GetSettingsBool("IncludeNSFWInDiscover") ?? false;
        }
        catch
        {
            // Ignore if settings not ready.
        }

        Task.Run(() => FetchDiscoverAsync(mature));
    }

    private async Task FetchDiscoverAsync(bool mature)
    {
        try
        {
            var payload = new
            {
                count = 16,
                matureContent = mature
            };

            Task<BaseResponse<IEnumerable<UgcResponsev2>>> tWorlds =
                ApiConnection.MakeRequest<IEnumerable<UgcResponsev2>>(
                    ApiConnection.ApiOperation.WorldsRandom,
                    payload,
                    "2");
            Task<BaseResponse<IEnumerable<UgcResponsev2>>> tAvatars =
                ApiConnection.MakeRequest<IEnumerable<UgcResponsev2>>(
                    ApiConnection.ApiOperation.AvatarsRandom,
                    payload,
                    "2");
            Task<BaseResponse<IEnumerable<UgcResponsev2>>> tProps =
                ApiConnection.MakeRequest<IEnumerable<UgcResponsev2>>(
                    ApiConnection.ApiOperation.PropsRandom,
                    payload,
                    "2");

            await Task.WhenAll(tWorlds, tAvatars, tProps);

            var worlds = new List<CardDto>();
            if (tWorlds.Result?.Data != null)
            {
                foreach (UgcResponsev2 e in tWorlds.Result.Data)
                {
                    if (e != null && !string.IsNullOrEmpty(e.id) && !string.IsNullOrEmpty(e.name))
                        worlds.Add(new CardDto { Id = e.id, Name = e.name, ImageUrl = e.image ?? string.Empty, PlayerCount = e.PlayerCount });
                }
            }

            var avatars = new List<CardDto>();
            if (tAvatars.Result?.Data != null)
            {
                foreach (UgcResponsev2 e in tAvatars.Result.Data)
                {
                    if (e != null && !string.IsNullOrEmpty(e.id) && !string.IsNullOrEmpty(e.name))
                        avatars.Add(new CardDto { Id = e.id, Name = e.name, ImageUrl = e.image ?? string.Empty });
                }
            }

            var props = new List<CardDto>();
            if (tProps.Result?.Data != null)
            {
                foreach (UgcResponsev2 e in tProps.Result.Data)
                {
                    if (e != null && !string.IsNullOrEmpty(e.id) && !string.IsNullOrEmpty(e.name))
                        props.Add(new CardDto { Id = e.id, Name = e.name, ImageUrl = e.image ?? string.Empty });
                }
            }

            lock (_lock)
            {
                _discoverWorlds = worlds;
                _discoverAvatars = avatars;
                _discoverProps = props;
                _discoverLoaded = true;
                _discoverLoading = false;
                _discoverError = string.Empty;
            }
        }
        catch (Exception ex)
        {
            MelonLogger.Warning($"[LumenicMenu] Failed to load Discover feed: {ex.Message}");
            lock (_lock)
            {
                _discoverLoading = false;
                _discoverError = $"Failed to load Discover: {ex.Message}";
            }
        }
        _onDirty?.Invoke();
    }

    public object BuildSnapshot(ImageLoaderService images, ImageLoaderService quickImages = null)
    {
        bool catsLoaded;
        lock (_lock) { catsLoaded = _categoriesLoaded; }
        if (!catsLoaded)
        {
            TryHydrateCategoriesFromViewManager();
        }

        List<CategoryDto> worldCats;
        List<CategoryDto> avatarCats;
        List<CategoryDto> propCats;
        ListSnapshotCopy worldsCopy;
        ListSnapshotCopy avatarsCopy;
        ListSnapshotCopy propsCopy;
        string searchQuery;
        bool searchLoading;
        string searchError;
        List<SearchCardDto> searchResults;
        List<CardDto> discoverWorlds;
        List<CardDto> discoverAvatars;
        List<CardDto> discoverProps;
        bool discoverLoading;
        bool discoverLoaded;
        string discoverError;
        DetailState detailCopy;
        List<InstanceDto> detailInstances = null;
        List<string> detailCategories = null;
        InstanceDetailState instDetailCopy;
        List<InstanceMemberDto> instMembers = null;
        bool creatingInstance;
        string createInstanceError;
        GroupModPromptState groupModPromptCopy;

        lock (_lock)
        {
            catsLoaded = _categoriesLoaded;
            worldCats = new List<CategoryDto>(_worldCategories);
            avatarCats = new List<CategoryDto>(_avatarCategories);
            propCats = new List<CategoryDto>(_propCategories);
            worldsCopy = CopyListState(_worlds);
            avatarsCopy = CopyListState(_avatars);
            propsCopy = CopyListState(_props);
            searchQuery = _searchQuery;
            searchLoading = _searchLoading;
            searchError = _searchError;
            searchResults = new List<SearchCardDto>(_searchResults);
            discoverWorlds = new List<CardDto>(_discoverWorlds);
            discoverAvatars = new List<CardDto>(_discoverAvatars);
            discoverProps = new List<CardDto>(_discoverProps);
            discoverLoading = _discoverLoading;
            discoverLoaded = _discoverLoaded;
            discoverError = _discoverError;
            creatingInstance = _creatingInstance;
            createInstanceError = _createInstanceError;
            groupModPromptCopy = _groupModPrompt;
            detailCopy = _detail;
            if (detailCopy != null)
            {
                detailInstances = detailCopy.Instances.Select(i => new InstanceDto
                {
                    Id = i.Id,
                    Name = i.Name,
                    PlayerCount = i.PlayerCount,
                    MaxPlayers = i.MaxPlayers,
                    Region = i.Region,
                    Members = i.Members != null ? new List<InstanceMemberDto>(i.Members) : new List<InstanceMemberDto>()
                }).ToList();
                detailCategories = new List<string>(detailCopy.Categories);
            }
            instDetailCopy = _instanceDetail;
            if (instDetailCopy != null)
            {
                instMembers = new List<InstanceMemberDto>(instDetailCopy.Members);
            }
        }

        var allWorldCats = new List<CategoryDto>();
        if (Settings.EnableRecentHistory)
        {
            allWorldCats.Add(new CategoryDto { Key = RecentHistoryService.KeyWorldsVisited, Name = "Recently Visited", IsSystem = true, IsRecent = true });
        }
        allWorldCats.AddRange(worldCats);

        var allAvatarCats = new List<CategoryDto>();
        if (Settings.EnableRecentHistory)
        {
            allAvatarCats.Add(new CategoryDto { Key = RecentHistoryService.KeyAvatarsUsed, Name = "Recently Used", IsSystem = true, IsRecent = true });
            allAvatarCats.Add(new CategoryDto { Key = RecentHistoryService.KeyAvatarsSeen, Name = "Recently Seen", IsSystem = true, IsRecent = true });
        }
        allAvatarCats.AddRange(avatarCats);

        var allPropCats = new List<CategoryDto>();
        if (Settings.EnableRecentHistory)
        {
            allPropCats.Add(new CategoryDto { Key = RecentHistoryService.KeyPropsSpawned, Name = "Recently Spawned", IsSystem = true, IsRecent = true });
            allPropCats.Add(new CategoryDto { Key = RecentHistoryService.KeyPropsSeen, Name = "Recently Seen", IsSystem = true, IsRecent = true });
        }
        allPropCats.AddRange(propCats);

        ImageLoaderService qmImg = quickImages ?? images;
        return new
        {
            categories_loaded = catsLoaded,
            pinned_items = _pinnedContent != null ? _pinnedContent.BuildSnapshot(qmImg) : new List<object>(),
            world_categories = allWorldCats.Select(c => new { key = c.Key, name = c.Name, is_system = c.IsSystem, is_recent = c.IsRecent }).ToList(),
            avatar_categories = allAvatarCats.Select(c => new { key = c.Key, name = c.Name, is_system = c.IsSystem, is_recent = c.IsRecent }).ToList(),
            prop_categories = allPropCats.Select(c => new { key = c.Key, name = c.Name, is_system = c.IsSystem, is_recent = c.IsRecent }).ToList(),
            worlds = BuildListDto(worldsCopy, images),
            avatars = BuildListDto(avatarsCopy, images),
            props = BuildListDto(propsCopy, images),
            search_query = searchQuery,
            search_loading = searchLoading,
            search_error = searchError,
            search_results = searchResults.Select(r => new
            {
                id = r.Id,
                name = r.Name,
                kind = r.Kind,
                image_key = images != null ? images.RequestThumbnail(NativeBridge.SurfaceMain, r.ImageUrl) : string.Empty
            }).ToList(),
            discover_worlds = discoverWorlds.Select(item => new
            {
                id = item.Id,
                name = item.Name,
                image_key = images != null ? images.RequestThumbnail(NativeBridge.SurfaceMain, item.ImageUrl) : string.Empty,
                player_count = item.PlayerCount
            }).ToList(),
            discover_avatars = discoverAvatars.Select(item => new
            {
                id = item.Id,
                name = item.Name,
                image_key = images != null ? images.RequestThumbnail(NativeBridge.SurfaceMain, item.ImageUrl) : string.Empty,
                player_count = 0
            }).ToList(),
            discover_props = discoverProps.Select(item => new
            {
                id = item.Id,
                name = item.Name,
                image_key = images != null ? images.RequestThumbnail(NativeBridge.SurfaceMain, item.ImageUrl) : string.Empty,
                player_count = 0
            }).ToList(),
            discover_loading = discoverLoading,
            discover_loaded = discoverLoaded,
            discover_error = discoverError,
            creating_instance = creatingInstance,
            create_instance_error = createInstanceError,
            group_mod_prompt = groupModPromptCopy == null
                ? null
                : new
                {
                    instance_id = groupModPromptCopy.InstanceId,
                    instance_name = groupModPromptCopy.InstanceName,
                    world_name = groupModPromptCopy.WorldName,
                    group_id = groupModPromptCopy.GroupId,
                    group_name = groupModPromptCopy.GroupName,
                    privacy = groupModPromptCopy.Privacy,
                    is_verified = groupModPromptCopy.IsVerified,
                    moderator_count = groupModPromptCopy.ModeratorCount
                },
            instance_detail = instDetailCopy == null
                ? null
                : new
                {
                    id = instDetailCopy.Id,
                    name = instDetailCopy.Name,
                    world_id = instDetailCopy.WorldId,
                    world_name = instDetailCopy.WorldName,
                    world_image_key = images != null ? images.RequestThumbnail(NativeBridge.SurfaceMain, instDetailCopy.WorldImageUrl) : string.Empty,
                    world_author_id = instDetailCopy.WorldAuthorId,
                    world_author_name = instDetailCopy.WorldAuthorName,
                    privacy = instDetailCopy.Privacy,
                    region = instDetailCopy.Region,
                    game_mode_name = instDetailCopy.GameModeName,
                    owner_id = instDetailCopy.OwnerId,
                    owner_name = instDetailCopy.OwnerName,
                    owner_rank = instDetailCopy.OwnerRank,
                    owner_image_key = images != null ? images.RequestThumbnail(NativeBridge.SurfaceMain, instDetailCopy.OwnerImageUrl) : string.Empty,
                    group_id = instDetailCopy.GroupId,
                    group_name = instDetailCopy.GroupName,
                    group_image_key = images != null ? images.RequestThumbnail(NativeBridge.SurfaceMain, instDetailCopy.GroupImageUrl) : string.Empty,
                    group_verified = instDetailCopy.GroupVerified,
                    can_join = instDetailCopy.CanJoin,
                    cannot_join_reason = instDetailCopy.CannotJoinReason,
                    player_count = instDetailCopy.PlayerCount,
                    max_players = instDetailCopy.MaxPlayers,
                    loading = instDetailCopy.Loading,
                    error = instDetailCopy.Error,
                    members = instMembers.Select(m => new
                    {
                        id = m.Id,
                        name = m.Name,
                        image_key = images != null ? images.RequestThumbnail(NativeBridge.SurfaceMain, m.ImageUrl) : string.Empty
                    }).ToList()
                },
            detail = detailCopy == null
                ? null
                : new
                {
                    kind = detailCopy.Kind,
                    id = detailCopy.Id,
                    name = detailCopy.Name,
                    description = detailCopy.Description,
                    author_id = detailCopy.AuthorId,
                    author_name = detailCopy.AuthorName,
                    author_image_key = images != null ? images.RequestThumbnail(NativeBridge.SurfaceMain, detailCopy.AuthorImageUrl) : string.Empty,
                    image_key = images != null ? images.RequestThumbnail(NativeBridge.SurfaceMain, detailCopy.ImageUrl) : string.Empty,
                    pano_url = detailCopy.PanoUrl,
                    uploaded_at = detailCopy.UploadedAt,
                    updated_at = detailCopy.UpdatedAt,
                    file_size_text = detailCopy.FileSizeText,
                    platforms = detailCopy.Platforms ?? new List<string>(),
                    has_3d_preview = detailCopy.Kind == "avatar" && PlayerPreviewRig.CanPreviewAvatar(detailCopy.Id),
                    is_current_avatar = detailCopy.Kind == "avatar" && string.Equals(detailCopy.Id, MetaPort.Instance?.currentAvatarGuid, StringComparison.OrdinalIgnoreCase),
                    is_public = detailCopy.IsPublic,
                    is_shared = detailCopy.IsShared,
                    can_use = detailCopy.CanUse,
                    loading = detailCopy.Loading,
                    error = detailCopy.Error,
                    categories = detailCategories,
                    instances = detailInstances.Select(i => new
                    {
                        id = i.Id,
                        name = i.Name,
                        player_count = i.PlayerCount,
                        max_players = i.MaxPlayers,
                        region = i.Region,
                        members = (i.Members ?? new List<InstanceMemberDto>()).Select(m => new
                        {
                            id = m.Id,
                            name = m.Name,
                            image_key = images != null ? images.RequestThumbnail(NativeBridge.SurfaceMain, m.ImageUrl) : string.Empty
                        }).ToList()
                    }).ToList()
                }
        };
    }

    private sealed class GroupModPromptState
    {
        public string InstanceId;
        public string InstanceName;
        public string WorldName;
        public string GroupId;
        public string GroupName;
        public string Privacy;
        public bool IsVerified;
        public int ModeratorCount;
    }

    private sealed class ListSnapshotCopy
    {
        public string CategoryKey;
        public bool IsSystem;
        public int Page;
        public int TotalPages;
        public int SortOrder;
        public bool Ascending;
        public bool Loading;
        public bool LoadingMore;
        public bool LoadedOnce;
        public int ListRevision;
        public string Error;
        public List<CardDto> Items;
    }

    private static ListSnapshotCopy CopyListState(ListState st)
    {
        return new ListSnapshotCopy
        {
            CategoryKey = st.CategoryKey,
            IsSystem = st.IsSystem,
            Page = st.Page,
            TotalPages = st.TotalPages,
            SortOrder = st.SortOrder,
            Ascending = st.Ascending,
            Loading = st.Loading,
            LoadingMore = st.LoadingMore,
            LoadedOnce = st.LoadedOnce,
            ListRevision = st.ListRevision,
            Error = st.Error,
            Items = new List<CardDto>(st.Items)
        };
    }

    private static object BuildListDto(ListSnapshotCopy st, ImageLoaderService images)
    {
        int totalPages = Math.Max(1, st.TotalPages);
        bool hasMore = !RecentHistoryService.IsLocalRecentCategory(st.CategoryKey) && (st.Page + 1 < totalPages);
        return new
        {
            category_key = st.CategoryKey,
            is_system = st.IsSystem,
            page = st.Page,
            total_pages = totalPages,
            sort_order = st.SortOrder,
            ascending = st.Ascending,
            loading = st.Loading,
            loading_more = st.LoadingMore,
            has_more = hasMore,
            list_revision = st.ListRevision,
            loaded_once = st.LoadedOnce,
            error = st.Error,
            items = st.Items.Select(item => new
            {
                id = item.Id,
                name = item.Name,
                image_key = images != null ? images.RequestThumbnail(NativeBridge.SurfaceMain, item.ImageUrl) : string.Empty,
                player_count = item.PlayerCount
            }).ToList()
        };
    }

    private async Task FetchCategoriesAsync(string initialKind)
    {
        try
        {
            BaseResponse<CombinedCategoriesResponse> resp =
                await ApiConnection.MakeRequest<CombinedCategoriesResponse>(ApiConnection.ApiOperation.Categories, null, "2");
            if (resp?.Data == null)
            {
                MelonLogger.Warning($"[LumenicMenu] Categories API returned empty (status={resp?.HttpStatusCode}, msg={resp?.Message}).");
                lock (_lock) { _categoriesLoading = false; }
                return;
            }

            var cats = new Categories_t();
            resp.Data.UpdateCategoryList(in cats);

            lock (_lock)
            {
                _worldCategories = cats.Worlds
                    .Select(c => new CategoryDto { Key = c.CategoryKey ?? string.Empty, Name = c.CategoryClearTextName ?? c.CategoryKey ?? string.Empty, IsSystem = c.IsSystemCategory })
                    .Where(c => !string.IsNullOrEmpty(c.Key))
                    .ToList();

                _avatarCategories = cats.Avatars
                    .Select(c => new CategoryDto { Key = c.CategoryKey ?? string.Empty, Name = c.CategoryClearTextName ?? c.CategoryKey ?? string.Empty, IsSystem = c.IsSystemCategory })
                    .Where(c => !string.IsNullOrEmpty(c.Key))
                    .ToList();

                _propCategories = cats.Spawnables
                    .Select(c => new CategoryDto { Key = c.CategoryKey ?? string.Empty, Name = c.CategoryClearTextName ?? c.CategoryKey ?? string.Empty, IsSystem = c.IsSystemCategory })
                    .Where(c => !string.IsNullOrEmpty(c.Key))
                    .ToList();

                _categoriesLoaded = true;
                _categoriesLoading = false;
            }

            _onDirty?.Invoke();

            if (!string.IsNullOrEmpty(initialKind))
            {
                ListState st = GetListState(initialKind);
                if (st != null)
                {
                    string catKey;
                    bool isSys;
                    int page;
                    int sort;
                    bool asc;
                    lock (_lock)
                    {
                        if (st.Loading) return;
                        if (string.IsNullOrEmpty(st.CategoryKey))
                        {
                            CategoryDto defCat = PickDefaultCategory(initialKind, GetCategoriesFor(initialKind));
                            if (defCat != null)
                            {
                                st.CategoryKey = defCat.Key;
                                st.IsSystem = defCat.IsSystem;
                            }
                        }
                        catKey = st.CategoryKey;
                        isSys = st.IsSystem;
                        page = st.Page;
                        sort = st.SortOrder;
                        asc = st.Ascending;
                    }
                    if (!string.IsNullOrEmpty(catKey))
                    {
                        LoadList(initialKind, catKey, isSys, page, sort, asc);
                    }
                }
            }
        }
        catch (Exception ex)
        {
            MelonLogger.Warning($"[LumenicMenu] Failed to fetch CVR categories: {ex.Message}");
            lock (_lock) { _categoriesLoading = false; }
        }
    }

    private async Task FetchListAsync(string kind, string categoryKey, bool isSystem, int page, ContentOrder order, bool ascending, int seq, bool append)
    {
        try
        {
            var payload = new
            {
                category = categoryKey,
                page,
                order,
                sortAscending = ascending
            };

            var items = new List<CardDto>();
            int totalPages = 1;
            string error = string.Empty;

            if (kind == "worlds")
            {
                var op = isSystem ? ApiConnection.ApiOperation.WorldsSystemCategory : ApiConnection.ApiOperation.WorldsUserCategory;
                BaseResponse<PaginatedList<ContentWorldListEntry>> resp =
                    await ApiConnection.MakeRequest<PaginatedList<ContentWorldListEntry>>(op, payload, "2");
                if (resp?.Data != null)
                {
                    totalPages = Math.Max(1, (int)resp.Data.TotalPages);
                    foreach (ContentWorldListEntry e in resp.Data.Entries)
                    {
                        if (string.IsNullOrEmpty(e.Id) || string.IsNullOrEmpty(e.Name)) continue;
                        string imgUrl = e.Image?.ToString() ?? string.Empty;
                        items.Add(new CardDto
                        {
                            Id = e.Id,
                            Name = e.Name,
                            ImageUrl = imgUrl,
                            PlayerCount = e.PlayerCount
                        });
                        _recentHistory?.UpdateKnownMetadata("world", e.Id, e.Name, imgUrl);
                    }
                }
                else
                {
                    error = resp?.Message ?? "Failed to load worlds.";
                    MelonLogger.Warning($"[LumenicMenu] Worlds load failed for category '{categoryKey}' (sys={isSystem}, status={resp?.HttpStatusCode}, msg={resp?.Message}).");
                }
            }
            else if (kind == "avatars")
            {
                var op = isSystem ? ApiConnection.ApiOperation.AvatarsSystemCategory : ApiConnection.ApiOperation.AvatarsUserCategory;
                BaseResponse<PaginatedList<ContentBaseListEntry>> resp =
                    await ApiConnection.MakeRequest<PaginatedList<ContentBaseListEntry>>(op, payload, "2");
                if (resp?.Data != null)
                {
                    totalPages = Math.Max(1, (int)resp.Data.TotalPages);
                    foreach (ContentBaseListEntry e in resp.Data.Entries)
                    {
                        if (string.IsNullOrEmpty(e.Id) || string.IsNullOrEmpty(e.Name)) continue;
                        string imgUrl = e.Image?.ToString() ?? string.Empty;
                        items.Add(new CardDto
                        {
                            Id = e.Id,
                            Name = e.Name,
                            ImageUrl = imgUrl
                        });
                        _recentHistory?.UpdateKnownMetadata("avatar", e.Id, e.Name, imgUrl);
                    }
                }
                else
                {
                    error = resp?.Message ?? "Failed to load avatars.";
                }
            }
            else if (kind == "props")
            {
                var op = isSystem ? ApiConnection.ApiOperation.PropsSystemCategory : ApiConnection.ApiOperation.PropsUserCategory;
                BaseResponse<PaginatedList<ContentBaseListEntry>> resp =
                    await ApiConnection.MakeRequest<PaginatedList<ContentBaseListEntry>>(op, payload, "2");
                if (resp?.Data != null)
                {
                    totalPages = Math.Max(1, (int)resp.Data.TotalPages);
                    foreach (ContentBaseListEntry e in resp.Data.Entries)
                    {
                        if (string.IsNullOrEmpty(e.Id) || string.IsNullOrEmpty(e.Name)) continue;
                        string imgUrl = e.Image?.ToString() ?? string.Empty;
                        items.Add(new CardDto
                        {
                            Id = e.Id,
                            Name = e.Name,
                            ImageUrl = imgUrl
                        });
                        _recentHistory?.UpdateKnownMetadata("prop", e.Id, e.Name, imgUrl);
                    }
                }
                else
                {
                    error = resp?.Message ?? "Failed to load props.";
                }
            }

            ListState st = GetListState(kind);
            if (st != null)
            {
                lock (_lock)
                {
                    if (st.RequestSeq == seq)
                    {
                        if (append)
                        {
                            if (string.IsNullOrEmpty(error))
                            {
                                var existing = new HashSet<string>(st.Items.Select(x => x.Id), StringComparer.OrdinalIgnoreCase);
                                foreach (CardDto item in items)
                                {
                                    if (existing.Add(item.Id))
                                        st.Items.Add(item);
                                }
                                st.Page = page;
                                st.TotalPages = Math.Max(st.TotalPages, totalPages);
                            }
                            st.Error = error;
                            st.LoadingMore = false;
                        }
                        else
                        {
                            st.Items = items;
                            st.Page = page;
                            st.TotalPages = totalPages;
                            st.Error = error;
                            st.Loading = false;
                            st.LoadingMore = false;
                            st.LoadedOnce = true;
                            st.LastFetchedUtc = DateTime.UtcNow;
                        }
                    }
                }
            }
            _onDirty?.Invoke();
        }
        catch (Exception ex)
        {
            ListState st = GetListState(kind);
            if (st != null)
            {
                lock (_lock)
                {
                    if (st.RequestSeq == seq)
                    {
                        st.Error = ex.Message;
                        st.Loading = false;
                        st.LoadingMore = false;
                    }
                }
            }
            _onDirty?.Invoke();
        }
    }

    private static void PopulateContentBaseMeta(DetailState next, ContentBase d)
    {
        next.Name = d.Name ?? string.Empty;
        next.Description = d.Description ?? string.Empty;
        next.AuthorId = d.Author?.Id ?? string.Empty;
        next.AuthorName = d.Author?.Name ?? string.Empty;
        next.AuthorImageUrl = d.Author?.Image?.ToString() ?? string.Empty;
        next.ImageUrl = d.Image?.ToString() ?? string.Empty;
        next.IsPublic = d.Public;
        next.IsShared = d.IsSharedWithMe;
        string myId = MetaPort.Instance?.ownerId ?? AuthManager.UserId;
        bool isMine = !string.IsNullOrEmpty(myId)
            && !string.IsNullOrEmpty(next.AuthorId)
            && string.Equals(next.AuthorId, myId, StringComparison.OrdinalIgnoreCase);
        next.CanUse = d.Permitted || d.Public || d.IsSharedWithMe || isMine;
        if (d.UploadedAt > DateTime.MinValue.AddYears(1))
            next.UploadedAt = d.UploadedAt.ToLocalTime().ToString("MMM d, yyyy");
        if (d.Categories != null)
            next.Categories = d.Categories.Where(c => !string.IsNullOrEmpty(c)).ToList();

        if (d.Platforms != null && d.Platforms.Count > 0)
        {
            ulong bestSize = 0;
            DateTime latestUpdate = DateTime.MinValue;
            foreach (var kv in d.Platforms)
            {
                string platLabel = kv.Key.ToString().Replace("_", " ");
                if (!next.Platforms.Contains(platLabel))
                    next.Platforms.Add(platLabel);
                if (kv.Value != null)
                {
                    if (kv.Value.FileSize > bestSize)
                        bestSize = kv.Value.FileSize;
                    if (kv.Value.UpdatedAt > latestUpdate)
                        latestUpdate = kv.Value.UpdatedAt;
                }
            }
            if (bestSize > 0)
                next.FileSizeText = FormatFileSize(bestSize);
            if (latestUpdate > DateTime.MinValue.AddYears(1))
                next.UpdatedAt = latestUpdate.ToLocalTime().ToString("MMM d, yyyy");
        }
    }

    private static string FormatFileSize(ulong bytes)
    {
        if (bytes >= 1024UL * 1024UL * 1024UL)
            return $"{bytes / (1024.0 * 1024.0 * 1024.0):F2} GB";
        if (bytes >= 1024UL * 1024UL)
            return $"{bytes / (1024.0 * 1024.0):F1} MB";
        if (bytes >= 1024UL)
            return $"{bytes / 1024.0:F0} KB";
        return $"{bytes} B";
    }

    private async Task FetchDetailAsync(string kind, string id, int seq)
    {
        try
        {
            var next = new DetailState
            {
                Kind = kind,
                Id = id,
                Loading = false
            };

            if (kind == "world")
            {
                BaseResponse<ContentWorldResponse> resp =
                    await ApiConnection.MakeRequest<ContentWorldResponse>(ApiConnection.ApiOperation.WorldDetail, new { worldID = id }, "2");
                if (resp?.Data != null)
                {
                    ContentWorldResponse d = resp.Data;
                    PopulateContentBaseMeta(next, d);
                    next.PanoUrl = (d.PanoHighResImage ?? d.PanoLowResImage)?.ToString() ?? string.Empty;
                    if (d.Instances != null)
                    {
                        foreach (WorldDetailsInstance inst in d.Instances)
                        {
                            if (inst == null || string.IsNullOrEmpty(inst.Id)) continue;
                            next.Instances.Add(new InstanceDto
                            {
                                Id = inst.Id,
                                Name = inst.Name ?? "Instance",
                                PlayerCount = inst.PlayerCount,
                                MaxPlayers = inst.MaxPlayerCount,
                                Region = inst.Region ?? string.Empty
                            });
                        }
                    }
                }
                else
                {
                    next.Error = resp?.Message ?? "Failed to load world details.";
                }
            }
            else if (kind == "avatar")
            {
                BaseResponse<ContentAvatarResponse> resp =
                    await ApiConnection.MakeRequest<ContentAvatarResponse>(ApiConnection.ApiOperation.AvatarDetail, new { avatarID = id }, "2");
                if (resp?.Data != null)
                {
                    PopulateContentBaseMeta(next, resp.Data);
                }
                else
                {
                    next.Error = resp?.Message ?? "Failed to load avatar details.";
                }
            }
            else if (kind == "prop")
            {
                BaseResponse<ContentSpawnableResponse> resp =
                    await ApiConnection.MakeRequest<ContentSpawnableResponse>(ApiConnection.ApiOperation.PropDetail, new { id }, "2");
                if (resp?.Data != null)
                {
                    PopulateContentBaseMeta(next, resp.Data);
                }
                else
                {
                    next.Error = resp?.Message ?? "Failed to load prop details.";
                }
            }

            if (string.IsNullOrEmpty(next.Error) && !string.IsNullOrEmpty(next.Name))
            {
                _recentHistory?.UpdateKnownMetadata(kind, id, next.Name, next.ImageUrl);
                _pinnedContent?.PatchMetadata(kind, id, next.Name, next.ImageUrl);
            }

            List<string> instancesToEnrich = null;
            lock (_lock)
            {
                if (_detailSeq == seq)
                {
                    _detail = next;
                    if (kind == "world" && next.Instances.Count > 0)
                    {
                        instancesToEnrich = next.Instances
                            .Where(i => i.PlayerCount > 0 && !string.IsNullOrEmpty(i.Id))
                            .Take(12)
                            .Select(i => i.Id)
                            .ToList();
                    }
                }
            }
            _onDirty?.Invoke();

            if (instancesToEnrich != null && instancesToEnrich.Count > 0)
            {
                _ = Task.Run(() => EnrichWorldInstanceMembersAsync(instancesToEnrich, seq));
            }
        }
        catch (Exception ex)
        {
            lock (_lock)
            {
                if (_detailSeq == seq && _detail != null)
                {
                    _detail.Loading = false;
                    _detail.Error = ex.Message;
                }
            }
            _onDirty?.Invoke();
        }
    }

    private async Task EnrichWorldInstanceMembersAsync(List<string> instanceIds, int seq)
    {
        const int batchSize = 4;
        for (int offset = 0; offset < instanceIds.Count; offset += batchSize)
        {
            lock (_lock)
            {
                if (_detailSeq != seq || _detail == null)
                    return;
            }

            var batch = instanceIds.Skip(offset).Take(batchSize).ToList();
            var tasks = batch.Select(async instId =>
            {
                try
                {
                    BaseResponse<InstanceDetailsResponse> resp = await ApiConnection.MakeRequest<InstanceDetailsResponse>(
                        ApiConnection.ApiOperation.InstanceDetail,
                        new { instanceID = instId }).ConfigureAwait(false);
                    if (resp?.Data?.Members == null)
                        return (instId, (List<InstanceMemberDto>)null);

                    var list = new List<InstanceMemberDto>(4);
                    foreach (UserDetails m in resp.Data.Members)
                    {
                        if (m == null || string.IsNullOrEmpty(m.Id))
                            continue;
                        list.Add(new InstanceMemberDto
                        {
                            Id = m.Id,
                            Name = m.Name ?? "Player",
                            ImageUrl = m.ImageUrl ?? string.Empty
                        });
                        if (list.Count >= 4)
                            break;
                    }
                    return (instId, list);
                }
                catch
                {
                    return (instId, (List<InstanceMemberDto>)null);
                }
            }).ToArray();

            var results = await Task.WhenAll(tasks).ConfigureAwait(false);
            bool anyUpdated = false;
            lock (_lock)
            {
                if (_detailSeq != seq || _detail == null)
                    return;

                foreach (var (instId, members) in results)
                {
                    if (members == null || members.Count == 0)
                        continue;
                    InstanceDto target = _detail.Instances.FirstOrDefault(i => i.Id == instId);
                    if (target != null)
                    {
                        target.Members = members;
                        anyUpdated = true;
                    }
                }
            }

            if (anyUpdated)
                _onDirty?.Invoke();
        }
    }

    private async Task FetchSearchAsync(string term, bool matureContent, int seq)
    {
        try
        {
            BaseResponse<IEnumerable<SearchResult>> resp =
                await ApiConnection.MakeRequest<IEnumerable<SearchResult>>(
                    ApiConnection.ApiOperation.GlobalSearch,
                    new { term, matureContent });

            var results = new List<SearchCardDto>();
            string error = string.Empty;

            if (resp?.Data != null)
            {
                foreach (SearchResult r in resp.Data)
                {
                    if (r == null || string.IsNullOrEmpty(r.Id) || string.IsNullOrEmpty(r.Name))
                        continue;

                    string k = (r.Type ?? string.Empty).Trim().ToLowerInvariant();
                    if (k.Contains("world")) k = "world";
                    else if (k.Contains("avatar")) k = "avatar";
                    else if (k.Contains("prop") || k.Contains("spawnable")) k = "prop";
                    else if (k.Contains("user")) k = "user";

                    results.Add(new SearchCardDto
                    {
                        Id = r.Id,
                        Name = r.Name,
                        Kind = k,
                        ImageUrl = r.ImageUrl ?? string.Empty
                    });
                }
            }
            else
            {
                error = resp?.Message ?? "Search failed.";
            }

            lock (_lock)
            {
                if (_searchSeq == seq)
                {
                    _searchResults = results;
                    _searchError = error;
                    _searchLoading = false;
                }
            }
            _onDirty?.Invoke();
        }
        catch (Exception ex)
        {
            lock (_lock)
            {
                if (_searchSeq == seq)
                {
                    _searchError = ex.Message;
                    _searchLoading = false;
                }
            }
            _onDirty?.Invoke();
        }
    }

    private ListState GetListState(string kind)
    {
        return kind switch
        {
            "worlds" => _worlds,
            "avatars" => _avatars,
            "props" => _props,
            _ => null
        };
    }

    private static CategoryDto PickDefaultCategory(string kind, List<CategoryDto> cats)
    {
        if (cats == null || cats.Count == 0)
            return null;

        string[] preferredKeys = kind switch
        {
            "worlds" => new[] { "wrldactive", "wrldpopular", "wrldtrending", "wrldnew" },
            "avatars" => new[] { "avtrmine", "avtrpublic" },
            "props" => new[] { "propmine", "prop_mine", "proppublic", "prop_public" },
            _ => Array.Empty<string>()
        };

        foreach (string key in preferredKeys)
        {
            CategoryDto match = cats.FirstOrDefault(c =>
                string.Equals(c.Key, key, StringComparison.OrdinalIgnoreCase));
            if (match != null)
                return match;
        }

        string[] preferredNames = kind switch
        {
            "worlds" => new[] { "active", "popular", "trending", "new" },
            "avatars" => new[] { "my avatars", "public" },
            "props" => new[] { "my props", "public" },
            _ => Array.Empty<string>()
        };

        foreach (string name in preferredNames)
        {
            CategoryDto match = cats.FirstOrDefault(c =>
                c.IsSystem && !string.IsNullOrEmpty(c.Name)
                && c.Name.IndexOf(name, StringComparison.OrdinalIgnoreCase) >= 0);
            if (match != null)
                return match;
        }

        return cats.FirstOrDefault(c => c.IsSystem) ?? cats[0];
    }

    private List<CategoryDto> GetCategoriesFor(string kind)
    {
        return kind switch
        {
            "worlds" => _worldCategories,
            "avatars" => _avatarCategories,
            "props" => _propCategories,
            _ => new List<CategoryDto>()
        };
    }
}
