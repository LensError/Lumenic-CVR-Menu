using System;
using System.Collections.Generic;
using ABI_RC.Core.Savior;
using ABI_RC.Systems.PlayerColors;
using Newtonsoft.Json.Linq;
using UnityEngine;

namespace LumenicMenu.Bridge.Actions;

internal static class SocialActions
{
    public static void Register(Dictionary<string, Action<JObject, LumenicMenuMod>> map)
    {
        map["SocialRefreshFriends"] = (_, mod) => mod.Social?.RefreshFriends();
        map["SocialEnsureHomeFeeds"] = (_, mod) => mod.Social?.EnsureHomeFeedsLoaded();
        map["SocialLoadUserDetail"] = (obj, mod) => mod.Social?.LoadUserDetail(obj.Value<string>("user_id") ?? string.Empty);
        map["SocialCloseUserDetail"] = (_, mod) => mod.Social?.CloseUserDetail();
        map["SocialSaveUserNote"] = (obj, mod) => mod.Social?.SaveUserNote(
            obj.Value<string>("user_id") ?? string.Empty,
            obj.Value<string>("note") ?? string.Empty);
        map["SocialUpdateOwnProfile"] = (obj, mod) => mod.Social?.UpdateOwnProfile(
            obj.Value<string>("pronouns") ?? string.Empty,
            obj.Value<string>("bio") ?? string.Empty,
            obj.Value<bool>("update_pronouns"),
            obj.Value<bool>("update_bio"),
            uid => mod.UiLib?.InvalidateBioCache(uid));

        map["SetColorizeMenu"] = (obj, mod) =>
        {
            bool enabled = obj.Value<bool>("enabled");
            CVRSettings s = MetaPort.Instance?.settings;
            if (s != null)
            {
                s.SetSettingsBool(PlayerColorsManager.ColourizeMenuSettingKey, enabled);
                s.settingsHaveChanged = true;
            }
            SettingsBridge.InvalidateCache();
            mod.MarkStateDirty(StateDomain.Core | StateDomain.Settings);
        };
        map["SetPlayerColorPreset"] = (obj, mod) =>
        {
            int presetId = obj.Value<int>("preset_id");
            var preset = (PlayerColorPresets)presetId;
            if (PlayerColorsUtility.ColorPresetToColor.TryGetValue(preset, out var pair))
            {
                Color c1 = preset == PlayerColorPresets.ChilloutVR ? pair.Item2 : pair.Item1;
                Color c2 = preset == PlayerColorPresets.ChilloutVR ? pair.Item1 : pair.Item2;
                PlayerColors cur = PlayerColorsManager.CurrentColors;
                PlayerColorsManager.ChangePlayerColor(new PlayerColors(c1, c2, cur.UsePrimaryAsEmission));
                mod.MarkStateDirty(StateDomain.Core);
            }
        };
        map["SetPlayerColorPrimary"] = (obj, mod) =>
        {
            int swatchId = obj.Value<int>("swatch_id");
            var swatch = (PlayerColorSwatches)swatchId;
            if (PlayerColorsUtility.ColorSwatchToColor.TryGetValue(swatch, out Color c))
            {
                PlayerColors cur = PlayerColorsManager.CurrentColors;
                PlayerColorsManager.ChangePlayerColor(new PlayerColors(c, cur.SecondaryColor, cur.UsePrimaryAsEmission));
                mod.MarkStateDirty(StateDomain.Core);
            }
        };
        map["SetPlayerColorSecondary"] = (obj, mod) =>
        {
            int swatchId = obj.Value<int>("swatch_id");
            var swatch = (PlayerColorSwatches)swatchId;
            if (PlayerColorsUtility.ColorSwatchToColor.TryGetValue(swatch, out Color c))
            {
                PlayerColors cur = PlayerColorsManager.CurrentColors;
                PlayerColorsManager.ChangePlayerColor(new PlayerColors(cur.PrimaryColor, c, cur.UsePrimaryAsEmission));
                mod.MarkStateDirty(StateDomain.Core);
            }
        };
        map["SetPlayerColorEmission"] = (obj, mod) =>
        {
            bool enabled = obj.Value<bool>("enabled");
            PlayerColors cur = PlayerColorsManager.CurrentColors;
            PlayerColorsManager.ChangePlayerColor(new PlayerColors(cur.PrimaryColor, cur.SecondaryColor, enabled));
            mod.MarkStateDirty(StateDomain.Core);
        };

        map["SocialInvitePlayer"] = (obj, mod) => mod.Social?.InvitePlayer(obj.Value<string>("user_id") ?? string.Empty);
        map["SocialRequestInvite"] = (obj, mod) => mod.Social?.RequestInvite(obj.Value<string>("user_id") ?? string.Empty);
        map["SocialSetPlayerMute"] = (obj, mod) => mod.Social?.SetPlayerMute(obj.Value<string>("user_id") ?? string.Empty, obj.Value<bool>("muted"));
        map["SocialSetPlayerVolume"] = (obj, mod) => mod.Social?.SetPlayerVoiceVolume(obj.Value<string>("user_id") ?? string.Empty, obj.Value<int>("volume"));
        map["SocialSetAvatarVisibility"] = (obj, mod) => mod.Social?.SetPlayerAvatarVisibility(obj.Value<string>("user_id") ?? string.Empty, obj.Value<int>("mode"));
        map["SocialSetPropVisibility"] = (obj, mod) => mod.Social?.SetPlayerPropVisibility(obj.Value<string>("user_id") ?? string.Empty, obj.Value<int>("mode"));
        map["SocialReloadPlayerAvatar"] = (obj, mod) => mod.Social?.ReloadPlayerAvatar(obj.Value<string>("user_id") ?? string.Empty);
        map["SocialKickUser"] = (obj, mod) => mod.Social?.KickUser(obj.Value<string>("user_id") ?? string.Empty);
        map["SocialSendFriendRequest"] = (obj, mod) => mod.Social?.SendFriendRequest(obj.Value<string>("user_id") ?? string.Empty);
        map["SocialUnfriend"] = (obj, mod) => mod.Social?.Unfriend(obj.Value<string>("user_id") ?? string.Empty);
        map["SocialSetBlocked"] = (obj, mod) => mod.Social?.SetBlocked(obj.Value<string>("user_id") ?? string.Empty, obj.Value<bool>("blocked"));
        map["SocialSetFeaturedBadge"] = (obj, mod) => mod.Social?.SetFeaturedBadge(obj.Value<string>("badge_id") ?? string.Empty, obj.Value<bool>("featured"));
        map["SocialSetFeaturedGroup"] = (obj, mod) => mod.Social?.SetFeaturedGroup(obj.Value<string>("group_id") ?? string.Empty);

        map["SocialLoadGroupDetail"] = (obj, mod) => mod.Social?.LoadGroupDetail(obj.Value<string>("group_id") ?? string.Empty);
        map["SocialCloseGroupDetail"] = (_, mod) => mod.Social?.CloseGroupDetail();
        map["SocialUpdateGroupName"] = (obj, mod) => mod.Social?.UpdateGroupName(
            obj.Value<string>("group_id") ?? string.Empty,
            obj.Value<string>("name") ?? string.Empty);
        map["SocialUpdateGroupDesc"] = (obj, mod) => mod.Social?.UpdateGroupDescription(
            obj.Value<string>("group_id") ?? string.Empty,
            obj.Value<string>("description") ?? string.Empty);
        map["SocialKickGroupMember"] = (obj, mod) => mod.Social?.KickGroupMember(
            obj.Value<string>("group_id") ?? string.Empty,
            obj.Value<string>("target_id") ?? string.Empty);
        map["SocialLeaveGroup"] = (obj, mod) => mod.Social?.LeaveGroup(obj.Value<string>("group_id") ?? string.Empty);
        map["SocialDeleteGroup"] = (obj, mod) => mod.Social?.DeleteGroup(obj.Value<string>("group_id") ?? string.Empty);
        map["SocialLoadMoreGroupMembers"] = (obj, mod) => mod.Social?.LoadMoreGroupMembers(obj.Value<string>("group_id") ?? string.Empty);
        map["SocialSetGroupMemberRole"] = (obj, mod) => mod.Social?.SetGroupMemberRole(
            obj.Value<string>("group_id") ?? string.Empty,
            obj.Value<string>("user_id") ?? string.Empty,
            obj.Value<string>("role") ?? string.Empty);
        map["SocialMassSetGroupMembersToMember"] = (obj, mod) =>
        {
            string groupId = obj.Value<string>("group_id") ?? string.Empty;
            var userIds = new List<string>();
            if (obj["user_ids"] is JArray arr)
            {
                foreach (JToken tok in arr)
                {
                    string uid = tok?.Value<string>();
                    if (!string.IsNullOrWhiteSpace(uid))
                        userIds.Add(uid);
                }
            }
            mod.Social?.MassSetGroupMembersToMember(groupId, userIds);
        };
        map["SocialCancelMassGroupRole"] = (_, mod) => mod.Social?.CancelMassGroupRole();
        map["SocialUpdateGroupSettings"] = (obj, mod) => mod.Social?.UpdateGroupSettings(
            obj.Value<string>("group_id") ?? string.Empty,
            obj.Value<bool>("listed"),
            obj.Value<int>("member_publicity"),
            obj.Value<int>("event_publicity"),
            obj.Value<int>("privacy_join"));
    }
}
