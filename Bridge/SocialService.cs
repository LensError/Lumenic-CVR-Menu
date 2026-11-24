using System;
using System.Collections.Generic;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using ABI_RC.Core.InteractionSystem;
using ABI_RC.Core.Networking;
using ABI_RC.Core.Networking.API;
using ABI_RC.Core.Networking.API.Responses;
using ABI_RC.Core.Networking.API.Responses.BadgeResponses;
using ABI_RC.Core.Networking.API.Responses.GroupResponses;
using ABI_RC.Core.Networking.API.Responses.Lists.ContentLists;
using ABI_RC.Core.Networking.API.UserWebsocket;
using ABI_RC.Core.Networking.IO.Global;
using ABI_RC.Core.Networking.IO.Instancing;
using ABI_RC.Core.Networking.IO.Social;
using ABI_RC.Core.Player;
using ABI_RC.Core.Savior;
using HarmonyLib;
using LumenicMenu.Native;
using MelonLoader;

namespace LumenicMenu.Bridge;

/// <summary>
/// Phase 3C Social, Friends, Current Instance, User Details, and Home Live Feeds service.
/// Reads live CVR friend/instance state and fetches UserDetails / Active Worlds asynchronously.
/// </summary>
internal sealed class SocialService
{
    private static readonly FieldInfo FriendsOnlineStateField =
        AccessTools.Field(typeof(ViewManager), "_friendsOnlineState");

    private static readonly FieldInfo ViewManagerCategoriesField =
        AccessTools.Field(typeof(ViewManager), "_categories");

    private static readonly MethodInfo GuardianKickUserMethod =
        AccessTools.Method(
            AccessTools.TypeByName("ABI_RC.Core.Networking.Guardian.GuardianExtendedControls"),
            "KickUser",
            new[] { typeof(string) });

    private sealed class ActiveWorldDto
    {
        public string Id;
        public string Name;
        public int PlayerCount;
        public string Region;
    }

    private sealed class UserBadgeDto
    {
        public string Id = string.Empty;
        public string Name = string.Empty;
        public string Description = string.Empty;
        public string ImageUrl = string.Empty;
        public bool Featured;
        public bool Hidden;
    }

    private sealed class UserGroupDto
    {
        public string Id = string.Empty;
        public string Name = string.Empty;
        public string Tag = string.Empty;
        public string ImageUrl = string.Empty;
        public int MemberCount;
        public bool IsOwner;
        public bool IsFeatured;
    }

    private sealed class UserDetailDto
    {
        public string Id = string.Empty;
        public string Name = string.Empty;
        public string Rank = string.Empty;
        public string Pronouns = string.Empty;
        public string Bio = string.Empty;
        public string Note = string.Empty;
        public bool SavingNote;
        public string NoteStatus = string.Empty;
        public bool SavingProfile;
        public string ProfileStatus = string.Empty;
        public string FeaturedBadge = string.Empty;
        public string FeaturedBadgeImageUrl = string.Empty;
        public string FeaturedGroup = string.Empty;
        public string FeaturedGroupImageUrl = string.Empty;
        public string ImageUrl = string.Empty;
        public string AvatarId = string.Empty;
        public string AvatarName = string.Empty;
        public string AvatarImageUrl = string.Empty;
        public bool OnlineState;
        public bool IsConnected;
        public bool IsFriend;
        public bool IsBlocked;
        public bool IsLocal;
        public string InstanceId = string.Empty;
        public string InstanceName = string.Empty;
        public string WorldName = string.Empty;
        public bool Loading;
        public string Error = string.Empty;
        public List<UserBadgeDto> Badges = new List<UserBadgeDto>();
        public List<UserGroupDto> Groups = new List<UserGroupDto>();
    }

    private sealed class GroupMemberDto
    {
        public string Id = string.Empty;
        public string Name = string.Empty;
        public string Role = string.Empty;
        public string ImageUrl = string.Empty;
    }

    private sealed class GroupDetailDto
    {
        public string Id = string.Empty;
        public string Name = string.Empty;
        public string Tag = string.Empty;
        public string Description = string.Empty;
        public string ImageUrl = string.Empty;
        public int MemberCount;
        public string OwnerId = string.Empty;
        public string OwnerName = string.Empty;
        public bool IsOwner;
        public bool IsMember;
        public bool IsFeatured;
        public bool Loading;
        public bool Saving;
        public string StatusMsg = string.Empty;
        public string Error = string.Empty;
        public List<GroupMemberDto> Members = new List<GroupMemberDto>();
    }

    private readonly Action _onDirty;
    private readonly RecentHistoryService _recentHistory;
    private readonly object _lock = new object();

    private bool _localProfileFetched;
    private string _localFeaturedBadge = string.Empty;
    private string _localFeaturedBadgeImageUrl = string.Empty;
    private string _localFeaturedGroup = string.Empty;
    private string _localFeaturedGroupImageUrl = string.Empty;
    private string _localRank = string.Empty;
    private string _localImageUrl = string.Empty;

    private bool _activeWorldsFetching;
    private DateTime _lastActiveWorldsFetch = DateTime.MinValue;
    private List<ActiveWorldDto> _activeWorlds = new List<ActiveWorldDto>();

    private UserDetailDto _userDetail;
    private int _userDetailSeq;

    private GroupDetailDto _groupDetail;
    private int _groupDetailSeq;

    private List<UserGroupDto> _myGroups = new List<UserGroupDto>();

    public SocialService(Action onDirty, RecentHistoryService recentHistory = null)
    {
        _onDirty = onDirty;
        _recentHistory = recentHistory;
    }

    public string LocalFeaturedBadge
    {
        get { lock (_lock) { return _localFeaturedBadge; } }
    }

    public string LocalFeaturedBadgeImageUrl
    {
        get { lock (_lock) { return _localFeaturedBadgeImageUrl; } }
    }

    public string LocalFeaturedGroup
    {
        get { lock (_lock) { return _localFeaturedGroup; } }
    }

    public string LocalFeaturedGroupImageUrl
    {
        get { lock (_lock) { return _localFeaturedGroupImageUrl; } }
    }

    public string LocalRank
    {
        get { lock (_lock) { return _localRank; } }
    }

    public string LocalImageUrl
    {
        get { lock (_lock) { return _localImageUrl; } }
    }

    public void EnsureLocalProfileLoaded()
    {
        if (!AuthManager.IsAuthenticated)
            return;

        string myId = MetaPort.Instance?.ownerId ?? AuthManager.UserId;
        if (string.IsNullOrEmpty(myId))
            return;

        bool needProfile;
        lock (_lock)
        {
            needProfile = !_localProfileFetched;
            if (needProfile) _localProfileFetched = true;
        }

        if (needProfile)
        {
            Task.Run(() => FetchLocalProfileAsync(myId));
        }
    }

    public void EnsureHomeFeedsLoaded()
    {
        if (!AuthManager.IsAuthenticated)
            return;

        string myId = MetaPort.Instance?.ownerId ?? AuthManager.UserId;
        bool needProfile;
        bool needWorlds;
        lock (_lock)
        {
            needProfile = !_localProfileFetched && !string.IsNullOrEmpty(myId);
            if (needProfile) _localProfileFetched = true;

            needWorlds = !_activeWorldsFetching && (DateTime.UtcNow - _lastActiveWorldsFetch).TotalSeconds > 60.0;
            if (needWorlds)
            {
                _activeWorldsFetching = true;
                _lastActiveWorldsFetch = DateTime.UtcNow;
            }
        }

        if (needProfile)
        {
            Task.Run(() => FetchLocalProfileAsync(myId));
        }

        if (needWorlds)
        {
            string activeKey = ResolveActiveWorldsKeyOnMainThread();
            Task.Run(() => FetchActiveWorldsAsync(activeKey));
        }
    }

    private static string ResolveActiveWorldsKeyOnMainThread()
    {
        try
        {
            if (ViewManager.Instance != null && ViewManagerCategoriesField?.GetValue(ViewManager.Instance) is Categories_t cats && cats.Worlds.Count > 0)
            {
                var match = cats.Worlds.FirstOrDefault(c => c.IsSystemCategory && c.CategoryKey != null && c.CategoryKey.IndexOf("active", StringComparison.OrdinalIgnoreCase) >= 0)
                    ?? cats.Worlds.FirstOrDefault(c => c.IsSystemCategory);
                if (match != null && !string.IsNullOrEmpty(match.CategoryKey))
                    return match.CategoryKey;
            }
        }
        catch
        {
            // Fallback to "active".
        }
        return "active";
    }

    public void RefreshFriends()
    {
        try
        {
            Friends.RequestFriendsListTaskFetch();
        }
        catch
        {
            // Ignore if not authenticated yet.
        }
        _onDirty?.Invoke();
    }

    public void LoadUserDetail(string userId)
    {
        if (!AuthManager.IsAuthenticated || string.IsNullOrWhiteSpace(userId))
            return;

        string localOwnerId = MetaPort.Instance?.ownerId ?? AuthManager.UserId ?? string.Empty;
        bool isLocal = !string.IsNullOrEmpty(localOwnerId)
            && string.Equals(userId, localOwnerId, StringComparison.OrdinalIgnoreCase);

        int seq;
        lock (_lock)
        {
            seq = ++_userDetailSeq;
            _userDetail = new UserDetailDto
            {
                Id = userId,
                IsLocal = isLocal,
                Loading = true
            };
        }
        _onDirty?.Invoke();

        Task.Run(() => FetchUserDetailAsync(userId, isLocal, seq));
    }

    public void CloseUserDetail()
    {
        lock (_lock)
        {
            _userDetailSeq++;
            _userDetail = null;
            _groupDetailSeq++;
            _groupDetail = null;
        }
        _onDirty?.Invoke();
    }

    public void LoadGroupDetail(string groupId)
    {
        if (!AuthManager.IsAuthenticated || string.IsNullOrWhiteSpace(groupId))
            return;

        string localOwnerId = MetaPort.Instance?.ownerId ?? AuthManager.UserId ?? string.Empty;
        int seq;
        lock (_lock)
        {
            seq = ++_groupDetailSeq;
            _groupDetail = new GroupDetailDto
            {
                Id = groupId,
                Loading = true
            };
        }
        _onDirty?.Invoke();

        Task.Run(() => FetchGroupDetailAsync(groupId, localOwnerId, seq));
    }

    public void CloseGroupDetail()
    {
        lock (_lock)
        {
            _groupDetailSeq++;
            _groupDetail = null;
        }
        _onDirty?.Invoke();
    }

    public void UpdateGroupName(string groupId, string newName)
    {
        if (!AuthManager.IsAuthenticated || string.IsNullOrWhiteSpace(groupId) || string.IsNullOrWhiteSpace(newName))
            return;

        string trimmed = newName.Trim();
        lock (_lock)
        {
            if (_groupDetail != null && _groupDetail.Id == groupId)
            {
                _groupDetail.Saving = true;
                _groupDetail.StatusMsg = string.Empty;
                _groupDetail.Error = string.Empty;
            }
        }
        _onDirty?.Invoke();

        Task.Run(async () =>
        {
            try
            {
                var resp = await ApiConnection.MakeRequest<object>(
                    ApiConnection.ApiOperation.GroupNameUpdate,
                    new { groupId, name = trimmed });
                lock (_lock)
                {
                    if (_groupDetail != null && _groupDetail.Id == groupId)
                    {
                        _groupDetail.Saving = false;
                        if (resp != null && resp.IsSuccessStatusCode)
                        {
                            _groupDetail.Name = trimmed;
                            _groupDetail.StatusMsg = "Group name updated.";
                        }
                        else
                        {
                            _groupDetail.Error = resp?.Message ?? "Failed to update group name.";
                        }
                    }
                }
            }
            catch (Exception ex)
            {
                lock (_lock)
                {
                    if (_groupDetail != null && _groupDetail.Id == groupId)
                    {
                        _groupDetail.Saving = false;
                        _groupDetail.Error = ex.Message;
                    }
                }
            }
            _onDirty?.Invoke();
        });
    }

    public void UpdateGroupDescription(string groupId, string newDesc)
    {
        if (!AuthManager.IsAuthenticated || string.IsNullOrWhiteSpace(groupId))
            return;

        string safeDesc = newDesc ?? string.Empty;
        lock (_lock)
        {
            if (_groupDetail != null && _groupDetail.Id == groupId)
            {
                _groupDetail.Saving = true;
                _groupDetail.StatusMsg = string.Empty;
                _groupDetail.Error = string.Empty;
            }
        }
        _onDirty?.Invoke();

        Task.Run(async () =>
        {
            try
            {
                var resp = await ApiConnection.MakeRequest<object>(
                    ApiConnection.ApiOperation.GroupDescUpdate,
                    new { groupId, description = safeDesc });
                lock (_lock)
                {
                    if (_groupDetail != null && _groupDetail.Id == groupId)
                    {
                        _groupDetail.Saving = false;
                        if (resp != null && resp.IsSuccessStatusCode)
                        {
                            _groupDetail.Description = safeDesc;
                            _groupDetail.StatusMsg = "Group description updated.";
                        }
                        else
                        {
                            _groupDetail.Error = resp?.Message ?? "Failed to update group description.";
                        }
                    }
                }
            }
            catch (Exception ex)
            {
                lock (_lock)
                {
                    if (_groupDetail != null && _groupDetail.Id == groupId)
                    {
                        _groupDetail.Saving = false;
                        _groupDetail.Error = ex.Message;
                    }
                }
            }
            _onDirty?.Invoke();
        });
    }

    public void KickGroupMember(string groupId, string targetId)
    {
        if (!AuthManager.IsAuthenticated || string.IsNullOrWhiteSpace(groupId) || string.IsNullOrWhiteSpace(targetId))
            return;

        Task.Run(async () =>
        {
            try
            {
                var resp = await ApiConnection.MakeRequest<object>(
                    ApiConnection.ApiOperation.GroupKickUser,
                    new { groupId, targetId });
                lock (_lock)
                {
                    if (_groupDetail != null && _groupDetail.Id == groupId)
                    {
                        if (resp != null && resp.IsSuccessStatusCode)
                        {
                            _groupDetail.Members.RemoveAll(m => string.Equals(m.Id, targetId, StringComparison.OrdinalIgnoreCase));
                            _groupDetail.MemberCount = Math.Max(1, _groupDetail.MemberCount - 1);
                            _groupDetail.StatusMsg = "Member kicked.";
                        }
                        else
                        {
                            _groupDetail.Error = resp?.Message ?? "Failed to kick member.";
                        }
                    }
                }
            }
            catch (Exception ex)
            {
                lock (_lock)
                {
                    if (_groupDetail != null && _groupDetail.Id == groupId)
                        _groupDetail.Error = ex.Message;
                }
            }
            _onDirty?.Invoke();
        });
    }

    public void LeaveGroup(string groupId)
    {
        if (!AuthManager.IsAuthenticated || string.IsNullOrWhiteSpace(groupId))
            return;

        string myId = MetaPort.Instance?.ownerId ?? AuthManager.UserId ?? string.Empty;
        Task.Run(async () =>
        {
            try
            {
                var resp = await ApiConnection.MakeRequest<object>(
                    ApiConnection.ApiOperation.GroupLeave,
                    new { groupId });
                if (resp != null && resp.IsSuccessStatusCode)
                {
                    CloseGroupDetail();
                    if (!string.IsNullOrEmpty(myId))
                    {
                        int seq = -1;
                        lock (_lock)
                        {
                            if (_userDetail != null && string.Equals(_userDetail.Id, myId, StringComparison.OrdinalIgnoreCase))
                                seq = _userDetailSeq;
                        }
                        if (seq >= 0)
                            await FetchUserDetailAsync(myId, true, seq);
                    }
                }
                else
                {
                    lock (_lock)
                    {
                        if (_groupDetail != null && _groupDetail.Id == groupId)
                            _groupDetail.Error = resp?.Message ?? "Failed to leave group.";
                    }
                    _onDirty?.Invoke();
                }
            }
            catch (Exception ex)
            {
                lock (_lock)
                {
                    if (_groupDetail != null && _groupDetail.Id == groupId)
                        _groupDetail.Error = ex.Message;
                }
                _onDirty?.Invoke();
            }
        });
    }

    public void DeleteGroup(string groupId)
    {
        if (!AuthManager.IsAuthenticated || string.IsNullOrWhiteSpace(groupId))
            return;

        string myId = MetaPort.Instance?.ownerId ?? AuthManager.UserId ?? string.Empty;
        Task.Run(async () =>
        {
            try
            {
                var resp = await ApiConnection.MakeRequest<object>(
                    ApiConnection.ApiOperation.GroupDelete,
                    new { groupId });
                if (resp != null && resp.IsSuccessStatusCode)
                {
                    CloseGroupDetail();
                    if (!string.IsNullOrEmpty(myId))
                    {
                        int seq = -1;
                        lock (_lock)
                        {
                            if (_userDetail != null && string.Equals(_userDetail.Id, myId, StringComparison.OrdinalIgnoreCase))
                                seq = _userDetailSeq;
                        }
                        if (seq >= 0)
                            await FetchUserDetailAsync(myId, true, seq);
                    }
                }
                else
                {
                    lock (_lock)
                    {
                        if (_groupDetail != null && _groupDetail.Id == groupId)
                            _groupDetail.Error = resp?.Message ?? "Failed to delete group.";
                    }
                    _onDirty?.Invoke();
                }
            }
            catch (Exception ex)
            {
                lock (_lock)
                {
                    if (_groupDetail != null && _groupDetail.Id == groupId)
                        _groupDetail.Error = ex.Message;
                }
                _onDirty?.Invoke();
            }
        });
    }

    private async Task FetchGroupDetailAsync(string groupId, string localOwnerId, int seq)
    {
        try
        {
            var detailTask = ApiConnection.MakeRequest<GroupWithRelation>(
                ApiConnection.ApiOperation.GroupDetail,
                new { id = groupId });
            var membersTask = ApiConnection.MakeRequest<GroupMemberList>(
                ApiConnection.ApiOperation.GroupGetMembers,
                new { id = groupId, page = 0, sortOrder = "default", sortAscending = false });

            BaseResponse<GroupWithRelation> resp = await detailTask;
            BaseResponse<GroupMemberList> memResp = null;
            try { memResp = await membersTask; } catch { /* Non-fatal. */ }

            var next = new GroupDetailDto
            {
                Id = groupId,
                Loading = false
            };

            if (resp?.Data != null)
            {
                GroupWithRelation g = resp.Data;
                next.Name = g.Name ?? string.Empty;
                next.Tag = g.Tag ?? string.Empty;
                next.Description = g.Description ?? string.Empty;
                next.ImageUrl = g.Image != null ? g.Image.ToString() : string.Empty;
                next.MemberCount = g.MemberCount;
                next.OwnerId = g.Owner?.Id ?? string.Empty;
                next.OwnerName = g.Owner?.Name ?? string.Empty;
                next.IsOwner = g.LocalUserOwns
                    || (!string.IsNullOrEmpty(localOwnerId) && string.Equals(next.OwnerId, localOwnerId, StringComparison.OrdinalIgnoreCase));
                next.IsMember = next.IsOwner || (g.Relation != null && g.Relation.MemberStatus != ABI_RC.Core.Networking.API.Responses.GroupResponses.GroupEnums.GroupMemberStatus.None);
                lock (_lock)
                {
                    next.IsFeatured = !string.IsNullOrEmpty(_localFeaturedGroup)
                        && string.Equals(_localFeaturedGroup, next.Name, StringComparison.OrdinalIgnoreCase);
                }
            }
            else
            {
                next.Error = resp?.Message ?? "Failed to load group details.";
            }

            if (memResp?.Data?.Entries != null)
            {
                foreach (GroupMember m in memResp.Data.Entries)
                {
                    if (m == null || string.IsNullOrEmpty(m.Id)) continue;
                    next.Members.Add(new GroupMemberDto
                    {
                        Id = m.Id,
                        Name = m.Name ?? "Member",
                        Role = m.Role ?? "Member",
                        ImageUrl = m.Image ?? string.Empty
                    });
                }
            }

            lock (_lock)
            {
                if (_groupDetailSeq == seq)
                    _groupDetail = next;
            }
            _onDirty?.Invoke();
        }
        catch (Exception ex)
        {
            lock (_lock)
            {
                if (_groupDetailSeq == seq && _groupDetail != null)
                {
                    _groupDetail.Loading = false;
                    _groupDetail.Error = ex.Message;
                }
            }
            _onDirty?.Invoke();
        }
    }

    public void InvitePlayer(string userId)
    {
        if (!string.IsNullOrWhiteSpace(userId))
            ViewManager.Instance?.InvitePlayer(userId);
    }

    public void RequestInvite(string userId)
    {
        if (!string.IsNullOrWhiteSpace(userId))
            ViewManager.Instance?.RequestInvite(userId);
    }

    public void SendFriendRequest(string userId)
    {
        if (!string.IsNullOrWhiteSpace(userId))
            ViewManager.Instance?.SendFriendRequest(userId);
    }

    public void Unfriend(string userId)
    {
        if (!string.IsNullOrWhiteSpace(userId))
        {
            string myId = MetaPort.Instance?.ownerId ?? AuthManager.UserId;
            if (!string.IsNullOrEmpty(myId) && string.Equals(userId, myId, StringComparison.OrdinalIgnoreCase))
            {
                ABI_RC.Systems.UI.UILib.QuickMenuAPI.ShowNotice(
                    "Be Kind to Yourself",
                    "You can't unfriend yourself! Someone has to stay on your friends list.",
                    null,
                    "I'll be nice");
                return;
            }

            ViewManager.Instance?.Unfriend(userId);
            lock (_lock)
            {
                if (_userDetail != null && _userDetail.Id == userId)
                    _userDetail.IsFriend = false;
            }
            _onDirty?.Invoke();
        }
    }

    public void SetBlocked(string userId, bool blocked)
    {
        if (string.IsNullOrWhiteSpace(userId))
            return;

        string myId = MetaPort.Instance?.ownerId ?? AuthManager.UserId;
        if (blocked && !string.IsNullOrEmpty(myId) && string.Equals(userId, myId, StringComparison.OrdinalIgnoreCase))
        {
            ABI_RC.Systems.UI.UILib.QuickMenuAPI.ShowNotice(
                "Nice Try",
                "You can't block yourself. You're stuck with you forever.",
                null,
                "Fine...");
            return;
        }

        if (blocked)
            ViewManager.Instance?.BlockUser(userId, contactApi: true);
        else
            ViewManager.Instance?.UnblockUser(userId);

        lock (_lock)
        {
            if (_userDetail != null && _userDetail.Id == userId)
                _userDetail.IsBlocked = blocked;
        }
        _onDirty?.Invoke();
    }

    public void SetPlayerMute(string userId, bool muted)
    {
        if (string.IsNullOrWhiteSpace(userId) || MetaPort.Instance?.SelfModerationManager == null)
            return;
        try
        {
            MetaPort.Instance.SelfModerationManager.currentUserId = userId;
            MetaPort.Instance.settings?.SetSettingsBool("SelfModerationMute", muted);
        }
        catch (Exception ex)
        {
            MelonLogger.Warning($"[LumenicMenu] SetPlayerMute failed: {ex.Message}");
        }
        _onDirty?.Invoke();
    }

    public void SetPlayerVoiceVolume(string userId, int volume)
    {
        if (string.IsNullOrWhiteSpace(userId) || MetaPort.Instance?.SelfModerationManager == null)
            return;
        try
        {
            MetaPort.Instance.SelfModerationManager.currentUserId = userId;
            MetaPort.Instance.settings?.SetSettingsInt("SelfModerationVolume", UnityEngine.Mathf.Clamp(volume, 0, 200));
        }
        catch (Exception ex)
        {
            MelonLogger.Warning($"[LumenicMenu] SetPlayerVoiceVolume failed: {ex.Message}");
        }
        _onDirty?.Invoke();
    }

    public void SetPlayerAvatarVisibility(string userId, int mode)
    {
        if (string.IsNullOrWhiteSpace(userId) || MetaPort.Instance?.SelfModerationManager == null)
            return;
        try
        {
            MetaPort.Instance.SelfModerationManager.currentUserId = userId;
            MetaPort.Instance.settings?.SetSettingsInt("SelfModerationUsersAvatars", UnityEngine.Mathf.Clamp(mode, 0, 2));
        }
        catch (Exception ex)
        {
            MelonLogger.Warning($"[LumenicMenu] SetPlayerAvatarVisibility failed: {ex.Message}");
        }
        _onDirty?.Invoke();
    }

    public void SetPlayerPropVisibility(string userId, int mode)
    {
        if (string.IsNullOrWhiteSpace(userId) || MetaPort.Instance?.SelfModerationManager == null)
            return;
        try
        {
            MetaPort.Instance.SelfModerationManager.currentUserId = userId;
            MetaPort.Instance.settings?.SetSettingsInt("SelfModerationUsersProps", UnityEngine.Mathf.Clamp(mode, 0, 2));
        }
        catch (Exception ex)
        {
            MelonLogger.Warning($"[LumenicMenu] SetPlayerPropVisibility failed: {ex.Message}");
        }
        _onDirty?.Invoke();
    }

    public void ReloadPlayerAvatar(string userId)
    {
        if (string.IsNullOrWhiteSpace(userId))
            return;
        try
        {
            CVRPlayerManager.Instance?.ReloadPlayersAvatar(userId);
        }
        catch (Exception ex)
        {
            MelonLogger.Warning($"[LumenicMenu] ReloadPlayerAvatar failed: {ex.Message}");
        }
    }

    public void SaveUserNote(string userId, string note)
    {
        if (!AuthManager.IsAuthenticated || string.IsNullOrWhiteSpace(userId))
            return;

        string safeNote = note ?? string.Empty;
        if (safeNote.Length > 512)
            safeNote = safeNote.Substring(0, 512);

        lock (_lock)
        {
            if (_userDetail != null && string.Equals(_userDetail.Id, userId, StringComparison.OrdinalIgnoreCase))
            {
                _userDetail.SavingNote = true;
                _userDetail.NoteStatus = string.Empty;
            }
        }
        _onDirty?.Invoke();

        Task.Run(async () =>
        {
            try
            {
                BaseResponse<object> resp = await ApiConnection.MakeRequest<object>(
                    ApiConnection.ApiOperation.UserDetailNote,
                    new { userID = userId, note = safeNote }).ConfigureAwait(false);

                int code = resp != null ? (int)resp.HttpStatusCode : 0;
                lock (_lock)
                {
                    if (_userDetail != null && string.Equals(_userDetail.Id, userId, StringComparison.OrdinalIgnoreCase))
                    {
                        _userDetail.SavingNote = false;
                        if (code >= 200 && code < 300)
                        {
                            _userDetail.Note = safeNote;
                            _userDetail.NoteStatus = "Saved";
                        }
                        else if (code == 412)
                        {
                            _userDetail.NoteStatus = "Must be friends to save note";
                        }
                        else
                        {
                            _userDetail.NoteStatus = resp?.Message ?? $"Save failed ({code})";
                        }
                    }
                }
            }
            catch (Exception ex)
            {
                lock (_lock)
                {
                    if (_userDetail != null && string.Equals(_userDetail.Id, userId, StringComparison.OrdinalIgnoreCase))
                    {
                        _userDetail.SavingNote = false;
                        _userDetail.NoteStatus = ex.Message;
                    }
                }
            }
            _onDirty?.Invoke();
        });
    }

    public void UpdateOwnProfile(string pronouns, string bio, bool updatePronouns, bool updateBio, Action<string> onUpdated = null)
    {
        if (!AuthManager.IsAuthenticated || (!updatePronouns && !updateBio))
            return;

        string safePronouns = string.IsNullOrWhiteSpace(pronouns) ? string.Empty : pronouns.Trim();
        if (safePronouns.Length > 32)
            safePronouns = safePronouns.Substring(0, 32);

        string safeBio = string.IsNullOrWhiteSpace(bio) ? string.Empty : bio.Trim();
        if (safeBio.Length > 512)
            safeBio = safeBio.Substring(0, 512);

        lock (_lock)
        {
            if (_userDetail != null && _userDetail.IsLocal)
            {
                _userDetail.SavingProfile = true;
                _userDetail.ProfileStatus = string.Empty;
            }
        }
        _onDirty?.Invoke();

        Task.Run(async () =>
        {
            try
            {
                var payload = new Dictionary<string, object>();
                if (updatePronouns) payload["Pronouns"] = safePronouns;
                if (updateBio) payload["Bio"] = safeBio;

                BaseResponse<UserDetailsResponse> resp = await ApiConnection.MakeRequest<UserDetailsResponse>(
                    ApiConnection.ApiOperation.UpdateOurUserProfile,
                    payload).ConfigureAwait(false);

                int code = resp != null ? (int)resp.HttpStatusCode : 0;
                lock (_lock)
                {
                    if (_userDetail != null && _userDetail.IsLocal)
                    {
                        _userDetail.SavingProfile = false;
                        if (code >= 200 && code < 300)
                        {
                            if (resp?.Data != null)
                            {
                                _userDetail.Pronouns = resp.Data.ProfilePronouns ?? safePronouns;
                                _userDetail.Bio = resp.Data.ProfileBio ?? safeBio;
                            }
                            else
                            {
                                if (updatePronouns) _userDetail.Pronouns = safePronouns;
                                if (updateBio) _userDetail.Bio = safeBio;
                            }
                            _userDetail.ProfileStatus = "Saved";
                        }
                        else
                        {
                            _userDetail.ProfileStatus = resp?.Message ?? $"Save failed ({code})";
                        }
                    }
                }

                if (code >= 200 && code < 300 && resp?.Data != null)
                {
                    if (ViewManager.Instance != null)
                    {
                        var udField = AccessTools.Field(typeof(ViewManager), "UserDetails");
                        if (udField != null)
                            udField.SetValue(ViewManager.Instance, resp.Data);
                        else
                            AccessTools.Property(typeof(ViewManager), "UserDetails")?.SetValue(ViewManager.Instance, resp.Data);
                    }
                    onUpdated?.Invoke(resp.Data.Id);
                }
            }
            catch (Exception ex)
            {
                lock (_lock)
                {
                    if (_userDetail != null && _userDetail.IsLocal)
                    {
                        _userDetail.SavingProfile = false;
                        _userDetail.ProfileStatus = ex.Message;
                    }
                }
            }
            _onDirty?.Invoke();
        });
    }

    public void KickUser(string userId)
    {
        if (string.IsNullOrWhiteSpace(userId))
            return;
        try
        {
            GuardianKickUserMethod?.Invoke(null, new object[] { userId });
        }
        catch (Exception ex)
        {
            MelonLogger.Warning($"[LumenicMenu] KickUser failed: {ex.Message}");
        }
    }

    public List<object> BuildOnlineFriendsLeaflet()
    {
        var list = new List<object>();
        try
        {
            var friends = new List<Friend_t>();
            Friends.UpdateFriendsList(in friends);
            Dictionary<string, UserOnlineChangeCohtml> onlineMap = GetOnlineStateMap();
            var online = new List<(string Id, string Name, string WorldName, string Status)>();

            foreach (Friend_t f in friends)
            {
                if (string.IsNullOrEmpty(f.UserId)) continue;
                if (!onlineMap.TryGetValue(f.UserId, out UserOnlineChangeCohtml st) || !st.IsOnline)
                    continue;

                string worldName = st.Instance?.Name ?? string.Empty;
                online.Add((
                    f.UserId,
                    f.UserName ?? string.Empty,
                    worldName,
                    st.IsConnected ? (string.IsNullOrEmpty(worldName) ? "Private Instance" : "In World") : "Online"));
            }

            online.Sort((a, b) => string.Compare(a.Name, b.Name, StringComparison.OrdinalIgnoreCase));
            for (int i = 0; i < online.Count; i++)
            {
                var o = online[i];
                list.Add(new
                {
                    id = o.Id,
                    name = o.Name,
                    world_name = o.WorldName,
                    status = o.Status
                });
            }
        }
        catch
        {
            // Keep UI alive if CVR social state mutates mid-frame.
        }
        return list;
    }

    public List<object> BuildActiveWorldsLeaflet()
    {
        lock (_lock)
        {
            return _activeWorlds.Take(12).Select(w => (object)new
            {
                id = w.Id,
                name = w.Name,
                player_count = w.PlayerCount,
                region = w.Region
            }).ToList();
        }
    }

    public List<object> BuildActivityFeed()
    {
        var items = new List<object>();
        try
        {
            Dictionary<string, UserOnlineChangeCohtml> onlineMap = GetOnlineStateMap();
            var recent = onlineMap.Values
                .Where(s => !string.IsNullOrWhiteSpace(s.ChangeTitle))
                .OrderByDescending(s => s.UpdateTime)
                .Take(20)
                .ToList();

            DateTime now = DateTime.Now;
            foreach (UserOnlineChangeCohtml s in recent)
            {
                TimeSpan age = now - s.UpdateTime;
                string timeTxt = age.TotalMinutes < 1 ? "now"
                    : age.TotalHours < 1 ? $"{(int)age.TotalMinutes}m ago"
                    : $"{(int)age.TotalHours}h ago";

                items.Add(new
                {
                    id = $"{s.Id}_{s.UpdateTime.Ticks}",
                    title = s.ChangeTitle ?? string.Empty,
                    detail = s.ChangeBody ?? string.Empty,
                    time_text = timeTxt,
                    kind = "friend"
                });
            }
        }
        catch
        {
            // Ignore transient mutations.
        }
        return items;
    }

    public object BuildSocialSnapshot(ImageLoaderService images)
    {
        var allFriends = new List<object>();
        try
        {
            var friends = new List<Friend_t>();
            Friends.UpdateFriendsList(in friends);
            Dictionary<string, UserOnlineChangeCohtml> onlineMap = GetOnlineStateMap();

            var ordered = friends
                .Where(f => !string.IsNullOrEmpty(f.UserId))
                .Select(f =>
                {
                    onlineMap.TryGetValue(f.UserId, out UserOnlineChangeCohtml st);
                    bool online = st != null && st.IsOnline;
                    bool connected = st != null && st.IsConnected;
                    string instName = st?.Instance?.Name ?? string.Empty;
                    string instId = st?.Instance?.Id ?? string.Empty;
                    return (Friend: f, Online: online, Connected: connected, InstName: instName, InstId: instId);
                })
                .OrderByDescending(x => x.Online)
                .ThenBy(x => x.Friend.UserName, StringComparer.OrdinalIgnoreCase)
                .ToList();

            int offlineThumbCount = 0;
            foreach (var x in ordered)
            {
                bool requestThumb = images != null && (x.Online || offlineThumbCount < 48);
                if (!x.Online && requestThumb)
                    offlineThumbCount++;

                allFriends.Add(new
                {
                    id = x.Friend.UserId,
                    name = x.Friend.UserName ?? string.Empty,
                    image_key = requestThumb ? images.RequestThumbnail(NativeBridge.SurfaceMain, x.Friend.UserImageUrl) : string.Empty,
                    is_online = x.Online,
                    is_connected = x.Connected,
                    instance_id = x.InstId,
                    instance_name = x.InstName
                });
            }
        }
        catch
        {
            // Ignore transient mutations.
        }

        var instancePlayers = new List<object>();
        try
        {
            string localId = MetaPort.Instance?.ownerId ?? AuthManager.UserId ?? string.Empty;
            string localName = AuthManager.Username ?? "You";
            instancePlayers.Add(new
            {
                id = localId,
                name = localName,
                rank = "You (Local)",
                image_key = "local_pfp",
                is_friend = false,
                is_local = true
            });

            var netPlayers = CVRPlayerManager.Instance?.NetworkPlayers;
            if (netPlayers != null)
            {
                foreach (CVRPlayerEntity p in netPlayers)
                {
                    if (p == null || string.IsNullOrEmpty(p.Uuid)) continue;
                    instancePlayers.Add(new
                    {
                        id = p.Uuid,
                        name = p.Username ?? "Player",
                        rank = p.ApiUserRank ?? string.Empty,
                        image_key = images != null ? images.RequestThumbnail(NativeBridge.SurfaceMain, p.ApiProfileImageUrl) : string.Empty,
                        is_friend = Friends.FriendsWith(p.Uuid),
                        is_local = false
                    });
                }
            }
        }
        catch
        {
            // Ignore transient mutations.
        }

        UserDetailDto detailCopy;
        List<UserBadgeDto> detailBadges = null;
        List<UserGroupDto> detailGroups = null;
        lock (_lock)
        {
            detailCopy = _userDetail;
            if (detailCopy != null)
            {
                detailBadges = new List<UserBadgeDto>(detailCopy.Badges);
                detailGroups = new List<UserGroupDto>(detailCopy.Groups);
            }
        }

        bool isMuted = false;
        int voiceVol = 100;
        int avatarVis = 1;
        int propVis = 1;
        if (detailCopy != null && !detailCopy.IsLocal && !string.IsNullOrEmpty(detailCopy.Id))
        {
            try
            {
                var modEntry = MetaPort.Instance?.SelfModerationManager?.GetPlayerSelfModerationProfile(detailCopy.Id);
                if (modEntry != null)
                {
                    isMuted = modEntry.mute;
                    voiceVol = UnityEngine.Mathf.Clamp(UnityEngine.Mathf.RoundToInt(modEntry.voiceVolume * 100f), 0, 200);
                    avatarVis = modEntry.userAvatarVisibility;
                    propVis = modEntry.userPropVisibility;
                }
            }
            catch
            {
                // Keep defaults if moderation manager unavailable.
            }
        }

        object userDetailDto = detailCopy == null
            ? null
            : new
            {
                id = detailCopy.Id,
                name = detailCopy.Name,
                rank = detailCopy.Rank,
                pronouns = detailCopy.Pronouns,
                bio = detailCopy.Bio,
                note = detailCopy.Note,
                note_saving = detailCopy.SavingNote,
                note_status = detailCopy.NoteStatus,
                profile_saving = detailCopy.SavingProfile,
                profile_status = detailCopy.ProfileStatus,
                featured_badge = detailCopy.FeaturedBadge,
                featured_badge_image_key = images != null ? images.RequestThumbnail(NativeBridge.SurfaceMain, detailCopy.FeaturedBadgeImageUrl) : string.Empty,
                featured_group = detailCopy.FeaturedGroup,
                featured_group_image_key = images != null ? images.RequestThumbnail(NativeBridge.SurfaceMain, detailCopy.FeaturedGroupImageUrl) : string.Empty,
                image_key = images != null ? images.RequestThumbnail(NativeBridge.SurfaceMain, detailCopy.ImageUrl) : string.Empty,
                avatar_id = detailCopy.AvatarId,
                avatar_name = detailCopy.AvatarName,
                avatar_image_key = images != null ? images.RequestThumbnail(NativeBridge.SurfaceMain, detailCopy.AvatarImageUrl) : string.Empty,
                has_3d_preview = PlayerPreviewRig.CanPreviewPlayer(detailCopy.Id),
                online_state = detailCopy.OnlineState,
                is_connected = detailCopy.IsConnected,
                is_friend = detailCopy.IsFriend,
                is_blocked = detailCopy.IsBlocked,
                is_local = detailCopy.IsLocal,
                is_muted = isMuted,
                voice_volume = voiceVol,
                avatar_visibility = avatarVis,
                prop_visibility = propVis,
                instance_id = detailCopy.InstanceId,
                instance_name = detailCopy.InstanceName,
                world_name = detailCopy.WorldName,
                loading = detailCopy.Loading,
                error = detailCopy.Error,
                badges = detailBadges.Select(b => (object)new
                {
                    id = b.Id,
                    name = b.Name,
                    description = b.Description,
                    image_key = images != null ? images.RequestThumbnail(NativeBridge.SurfaceMain, b.ImageUrl) : string.Empty,
                    featured = b.Featured,
                    hidden = b.Hidden
                }).ToList(),
                groups = detailGroups.Select(g => (object)new
                {
                    id = g.Id,
                    name = g.Name,
                    tag = g.Tag,
                    image_key = images != null ? images.RequestThumbnail(NativeBridge.SurfaceMain, g.ImageUrl) : string.Empty,
                    member_count = g.MemberCount,
                    is_owner = g.IsOwner,
                    is_featured = g.IsFeatured
                }).ToList()
            };

        GroupDetailDto groupCopy;
        List<GroupMemberDto> groupMembers = null;
        List<UserGroupDto> myGroupsCopy;
        lock (_lock)
        {
            groupCopy = _groupDetail;
            if (groupCopy != null)
                groupMembers = new List<GroupMemberDto>(groupCopy.Members);
            myGroupsCopy = new List<UserGroupDto>(_myGroups);
        }

        var myGroupsDto = myGroupsCopy.Select(g => (object)new
        {
            id = g.Id,
            name = g.Name,
            tag = g.Tag,
            image_key = string.Empty,
            member_count = g.MemberCount,
            is_owner = g.IsOwner,
            is_featured = g.IsFeatured
        }).ToList();

        object groupDetailDto = groupCopy == null
            ? null
            : new
            {
                id = groupCopy.Id,
                name = groupCopy.Name,
                tag = groupCopy.Tag,
                description = groupCopy.Description,
                image_key = images != null ? images.RequestThumbnail(NativeBridge.SurfaceMain, groupCopy.ImageUrl) : string.Empty,
                member_count = groupCopy.MemberCount,
                owner_id = groupCopy.OwnerId,
                owner_name = groupCopy.OwnerName,
                is_owner = groupCopy.IsOwner,
                is_member = groupCopy.IsMember,
                is_featured = groupCopy.IsFeatured,
                loading = groupCopy.Loading,
                saving = groupCopy.Saving,
                status_msg = groupCopy.StatusMsg,
                error = groupCopy.Error,
                members = groupMembers.Select(m => (object)new
                {
                    id = m.Id,
                    name = m.Name,
                    role = m.Role,
                    image_key = images != null ? images.RequestThumbnail(NativeBridge.SurfaceMain, m.ImageUrl) : string.Empty
                }).ToList()
            };

        return new
        {
            friends = allFriends,
            instance_players = instancePlayers,
            recent_users = _recentHistory != null ? _recentHistory.BuildRecentUsersSnapshot(images) : new List<object>(),
            my_groups = myGroupsDto,
            user_detail = userDetailDto,
            group_detail = groupDetailDto
        };
    }

    public void SetFeaturedBadge(string badgeId, bool featured)
    {
        if (!AuthManager.IsAuthenticated || string.IsNullOrWhiteSpace(badgeId))
            return;

        string myId = MetaPort.Instance?.ownerId ?? AuthManager.UserId;
        Task.Run(async () =>
        {
            try
            {
                var op = featured
                    ? ApiConnection.ApiOperation.SetFeaturedBadge
                    : ApiConnection.ApiOperation.SetUnfeaturedBadge;
                var resp = await ApiConnection.MakeRequest<object>(op, new { badgeId });
                if (resp != null && resp.IsSuccessStatusCode && !string.IsNullOrEmpty(myId))
                {
                    await FetchLocalProfileAsync(myId);
                    int seq = -1;
                    lock (_lock)
                    {
                        if (_userDetail != null && string.Equals(_userDetail.Id, myId, StringComparison.OrdinalIgnoreCase))
                            seq = _userDetailSeq;
                    }
                    if (seq >= 0)
                        await FetchUserDetailAsync(myId, true, seq);
                }
            }
            catch
            {
                // Non-fatal.
            }
        });
    }

    public void SetFeaturedGroup(string groupId)
    {
        if (!AuthManager.IsAuthenticated || string.IsNullOrWhiteSpace(groupId))
            return;

        string myId = MetaPort.Instance?.ownerId ?? AuthManager.UserId;
        Task.Run(async () =>
        {
            try
            {
                var resp = await ApiConnection.MakeRequest<object>(
                    ApiConnection.ApiOperation.GroupSetFeatured,
                    new { id = groupId });
                if (resp != null && resp.IsSuccessStatusCode && !string.IsNullOrEmpty(myId))
                {
                    await FetchLocalProfileAsync(myId);
                    int seq = -1;
                    lock (_lock)
                    {
                        if (_userDetail != null && string.Equals(_userDetail.Id, myId, StringComparison.OrdinalIgnoreCase))
                            seq = _userDetailSeq;
                    }
                    if (seq >= 0)
                        await FetchUserDetailAsync(myId, true, seq);
                }
            }
            catch
            {
                // Non-fatal.
            }
        });
    }

    private static Dictionary<string, UserOnlineChangeCohtml> GetOnlineStateMap()
    {
        var map = new Dictionary<string, UserOnlineChangeCohtml>(StringComparer.Ordinal);
        if (ViewManager.Instance == null || FriendsOnlineStateField == null)
            return map;

        if (FriendsOnlineStateField.GetValue(ViewManager.Instance) is List<UserOnlineChangeCohtml> list)
        {
            for (int i = 0; i < list.Count; i++)
            {
                UserOnlineChangeCohtml item = list[i];
                if (item != null && !string.IsNullOrEmpty(item.Id))
                {
                    map[item.Id] = item;
                }
            }
        }
        return map;
    }

    private async Task FetchLocalProfileAsync(string userId)
    {
        try
        {
            var profileTask = ApiConnection.MakeRequest<UserDetailsResponse>(
                ApiConnection.ApiOperation.UserDetails,
                new { userID = userId });
            var groupsTask = ApiConnection.MakeRequest<OwnedMemberGroupResponse>(
                ApiConnection.ApiOperation.Groups);

            BaseResponse<UserDetailsResponse> resp = await profileTask;
            BaseResponse<OwnedMemberGroupResponse> groupsResp = null;
            try { groupsResp = await groupsTask; } catch { /* Non-fatal groups failure. */ }

            if (resp?.Data != null)
            {
                string featuredGroupId = resp.Data.FeaturedGroup?.Id ?? string.Empty;
                var myGroups = new List<UserGroupDto>();
                if (groupsResp?.Data != null)
                {
                    if (groupsResp.Data.Owned != null)
                    {
                        foreach (GroupResponse g in groupsResp.Data.Owned)
                        {
                            if (g == null || string.IsNullOrEmpty(g.Name)) continue;
                            string gid = g.Id.ToString();
                            myGroups.Add(new UserGroupDto
                            {
                                Id = gid,
                                Name = g.Name ?? string.Empty,
                                Tag = g.Tag ?? string.Empty,
                                ImageUrl = g.Image != null ? g.Image.ToString() : string.Empty,
                                MemberCount = g.MemberCount,
                                IsOwner = true,
                                IsFeatured = !string.IsNullOrEmpty(featuredGroupId)
                                    && string.Equals(gid, featuredGroupId, StringComparison.OrdinalIgnoreCase)
                            });
                        }
                    }
                    if (groupsResp.Data.Member != null)
                    {
                        foreach (GroupResponse g in groupsResp.Data.Member)
                        {
                            if (g == null || string.IsNullOrEmpty(g.Name)) continue;
                            string gid = g.Id.ToString();
                            if (myGroups.Any(x => string.Equals(x.Id, gid, StringComparison.OrdinalIgnoreCase)))
                                continue;
                            myGroups.Add(new UserGroupDto
                            {
                                Id = gid,
                                Name = g.Name ?? string.Empty,
                                Tag = g.Tag ?? string.Empty,
                                ImageUrl = g.Image != null ? g.Image.ToString() : string.Empty,
                                MemberCount = g.MemberCount,
                                IsOwner = false,
                                IsFeatured = !string.IsNullOrEmpty(featuredGroupId)
                                    && string.Equals(gid, featuredGroupId, StringComparison.OrdinalIgnoreCase)
                            });
                        }
                    }
                }

                lock (_lock)
                {
                    _localFeaturedBadge = resp.Data.FeaturedBadge?.Name ?? string.Empty;
                    _localFeaturedBadgeImageUrl = resp.Data.FeaturedBadge?.Image ?? string.Empty;
                    _localFeaturedGroup = resp.Data.FeaturedGroup?.Name ?? string.Empty;
                    _localFeaturedGroupImageUrl = resp.Data.FeaturedGroup?.Image != null
                        ? resp.Data.FeaturedGroup.Image.ToString()
                        : string.Empty;
                    _localRank = resp.Data.Rank ?? string.Empty;
                    _localImageUrl = resp.Data.ImageUrl ?? string.Empty;
                    if (groupsResp?.Data != null)
                        _myGroups = myGroups;
                }
                _onDirty?.Invoke();
            }
            else
            {
                lock (_lock) { _localProfileFetched = false; }
            }
        }
        catch
        {
            lock (_lock) { _localProfileFetched = false; }
        }
    }

    private async Task FetchActiveWorldsAsync(string activeKey)
    {
        try
        {
            BaseResponse<PaginatedList<ContentWorldListEntry>> resp =
                await ApiConnection.MakeRequest<PaginatedList<ContentWorldListEntry>>(
                    ApiConnection.ApiOperation.WorldsSystemCategory,
                    new { category = activeKey, page = 0, order = ContentOrder.Default, sortAscending = false },
                    "2");
            if (resp?.Data?.Entries != null)
            {
                var next = new List<ActiveWorldDto>();
                foreach (ContentWorldListEntry e in resp.Data.Entries)
                {
                    if (string.IsNullOrEmpty(e.Id) || string.IsNullOrEmpty(e.Name)) continue;
                    next.Add(new ActiveWorldDto
                    {
                        Id = e.Id,
                        Name = e.Name,
                        PlayerCount = e.PlayerCount,
                        Region = "Active"
                    });
                }
                lock (_lock)
                {
                    _activeWorlds = next;
                }
            }
            else
            {
                MelonLogger.Warning($"[LumenicMenu] Active worlds fetch returned empty for key '{activeKey}' (status={resp?.HttpStatusCode}, msg={resp?.Message}).");
            }
        }
        catch (Exception ex)
        {
            MelonLogger.Warning($"[LumenicMenu] Failed to fetch active worlds leaflet: {ex.Message}");
        }
        finally
        {
            lock (_lock) { _activeWorldsFetching = false; }
            _onDirty?.Invoke();
        }
    }

    private async Task FetchUserDetailAsync(string userId, bool isLocal, int seq)
    {
        try
        {
            var detailTask = ApiConnection.MakeRequest<UserDetailsResponse>(
                ApiConnection.ApiOperation.UserDetails,
                new { userID = userId });
            var badgesTask = ApiConnection.MakeRequest<UserBadge[]>(
                ApiConnection.ApiOperation.GetUserBadges,
                new { userId });
            var groupsTask = isLocal
                ? ApiConnection.MakeRequest<OwnedMemberGroupResponse>(ApiConnection.ApiOperation.Groups)
                : ApiConnection.MakeRequest<OwnedMemberGroupResponse>(
                    ApiConnection.ApiOperation.GroupGetUsersList,
                    new { id = userId });

            BaseResponse<UserDetailsResponse> resp = await detailTask;
            BaseResponse<UserBadge[]> badgesResp = null;
            try { badgesResp = await badgesTask; } catch { /* Non-fatal badges failure. */ }
            BaseResponse<OwnedMemberGroupResponse> groupsResp = null;
            try { groupsResp = await groupsTask; } catch { /* Non-fatal groups failure. */ }

            var next = new UserDetailDto
            {
                Id = userId,
                IsLocal = isLocal,
                Loading = false
            };

            string featuredGroupId = string.Empty;
            if (resp?.Data != null)
            {
                UserDetailsResponse d = resp.Data;
                next.Name = d.Name ?? string.Empty;
                next.Rank = d.Rank ?? string.Empty;
                next.Pronouns = d.ProfilePronouns ?? string.Empty;
                next.Bio = d.ProfileBio ?? string.Empty;
                next.Note = d.Note ?? string.Empty;
                next.FeaturedBadge = d.FeaturedBadge?.Name ?? string.Empty;
                next.FeaturedBadgeImageUrl = d.FeaturedBadge?.Image ?? string.Empty;
                next.FeaturedGroup = d.FeaturedGroup?.Name ?? string.Empty;
                next.FeaturedGroupImageUrl = d.FeaturedGroup?.Image != null ? d.FeaturedGroup.Image.ToString() : string.Empty;
                featuredGroupId = d.FeaturedGroup?.Id ?? string.Empty;
                next.ImageUrl = d.ImageUrl ?? string.Empty;
                next.AvatarId = d.Avatar?.Id ?? string.Empty;
                next.AvatarName = d.Avatar?.Name ?? string.Empty;
                next.AvatarImageUrl = d.Avatar?.ImageUrl ?? string.Empty;
                next.OnlineState = d.OnlineState;
                next.IsConnected = d.IsConnected;
                next.IsFriend = d.IsFriend;
                next.IsBlocked = d.IsBlocked;
                next.InstanceId = d.Instance?.Id ?? string.Empty;
                next.InstanceName = d.Instance?.Name ?? string.Empty;
                next.WorldName = d.Instance?.World?.Name ?? string.Empty;
            }
            else
            {
                next.Error = resp?.Message ?? "Failed to load user details.";
            }

            if (badgesResp?.Data != null)
            {
                foreach (UserBadge b in badgesResp.Data)
                {
                    if (b == null || string.IsNullOrEmpty(b.BadgeName)) continue;
                    next.Badges.Add(new UserBadgeDto
                    {
                        Id = b.BadgeIdent ?? string.Empty,
                        Name = b.BadgeName ?? string.Empty,
                        Description = b.BadgeDescription ?? string.Empty,
                        ImageUrl = b.BadgeImage ?? string.Empty,
                        Featured = b.Featured,
                        Hidden = b.Hidden
                    });
                }
            }

            if (groupsResp?.Data != null)
            {
                if (groupsResp.Data.Owned != null)
                {
                    foreach (GroupResponse g in groupsResp.Data.Owned)
                    {
                        if (g == null || string.IsNullOrEmpty(g.Name)) continue;
                        string gid = g.Id.ToString();
                        next.Groups.Add(new UserGroupDto
                        {
                            Id = gid,
                            Name = g.Name ?? string.Empty,
                            Tag = g.Tag ?? string.Empty,
                            ImageUrl = g.Image != null ? g.Image.ToString() : string.Empty,
                            MemberCount = g.MemberCount,
                            IsOwner = true,
                            IsFeatured = !string.IsNullOrEmpty(featuredGroupId)
                                && string.Equals(gid, featuredGroupId, StringComparison.OrdinalIgnoreCase)
                        });
                    }
                }
                if (groupsResp.Data.Member != null)
                {
                    foreach (GroupResponse g in groupsResp.Data.Member)
                    {
                        if (g == null || string.IsNullOrEmpty(g.Name)) continue;
                        string gid = g.Id.ToString();
                        if (next.Groups.Any(x => string.Equals(x.Id, gid, StringComparison.OrdinalIgnoreCase)))
                            continue;
                        next.Groups.Add(new UserGroupDto
                        {
                            Id = gid,
                            Name = g.Name ?? string.Empty,
                            Tag = g.Tag ?? string.Empty,
                            ImageUrl = g.Image != null ? g.Image.ToString() : string.Empty,
                            MemberCount = g.MemberCount,
                            IsOwner = false,
                            IsFeatured = !string.IsNullOrEmpty(featuredGroupId)
                                && string.Equals(gid, featuredGroupId, StringComparison.OrdinalIgnoreCase)
                        });
                    }
                }
            }

            lock (_lock)
            {
                if (isLocal && groupsResp?.Data != null)
                {
                    _myGroups = new List<UserGroupDto>(next.Groups);
                }
                if (_userDetailSeq == seq)
                {
                    _userDetail = next;
                }
            }
            _onDirty?.Invoke();
        }
        catch (Exception ex)
        {
            lock (_lock)
            {
                if (_userDetailSeq == seq && _userDetail != null)
                {
                    _userDetail.Loading = false;
                    _userDetail.Error = ex.Message;
                }
            }
            _onDirty?.Invoke();
        }
    }
}
