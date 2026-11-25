using ABI.CCK.Components;
using ABI_RC.Core.Networking.GameServer;
using ABI_RC.Core.Player;
using ABI_RC.Core.PropManagement;
using ABI_RC.Core.Savior;
using ABI_RC.Core.UI.UIRework.Managers;
using ABI_RC.Systems.AvatarScaling;
using ABI_RC.Systems.GameEventSystem;
using ABI_RC.Systems.PlayerColors;
using GameCommon.Packets.Garrison.GameServer;
using LumenicMenu.Native;

namespace LumenicMenu.Bridge;

/// <summary>
/// Manages ChilloutVR live state event subscriptions (mic, world, instance, player, avatar,
/// props, scaling, colors, virtual keyboard) and routes them into domain-scoped dirty flags.
/// </summary>
internal sealed class LiveEventWiring
{
    private readonly LumenicMenuMod _mod;
    private bool _subscribed;

    public LiveEventWiring(LumenicMenuMod mod)
    {
        _mod = mod;
    }

    public void Subscribe()
    {
        if (_subscribed)
            return;

        CVRGameEventSystem.Microphone.OnMute.AddListener(OnExternalStateEvent);
        CVRGameEventSystem.Microphone.OnUnmute.AddListener(OnExternalStateEvent);
        CVRGameEventSystem.World.OnLoad.AddListener(OnWorldLoadForHistory);
        CVRGameEventSystem.World.OnReady.AddListener(OnWorldReadyForHistory);
        CVRGameEventSystem.Instance.OnConnected.AddListener(OnExternalWorldEvent);
        CVRGameEventSystem.Instance.OnDisconnected.AddListener(OnExternalWorldEvent);
        CVRGameEventSystem.Player.OnJoinEntity.AddListener(OnPlayerJoinedForHistory);
        CVRGameEventSystem.Player.OnLeaveEntity.AddListener(OnExternalPlayerEvent);
        CVRGameEventSystem.Avatar.OnLocalAvatarLoad.AddListener(OnLocalAvatarLoadForHistory);
        CVRGameEventSystem.Avatar.OnLocalAvatarClear.AddListener(OnExternalAvatarEvent);
        CVRGameEventSystem.Avatar.OnRemoteAvatarLoad.AddListener(OnRemoteAvatarLoadForHistory);
        CVRGameEventSystem.Spawnable.OnPropSpawned.AddListener(OnPropSpawnedForHistory);
        GSInfoHandler.OnGSInfoUpdate += OnGsInfoUpdated;
        UniversalScaling.RulesChanged += OnExternalStateEvent;
        PlayerColorsManager.OnLocalPlayerColorChanged += OnPlayerColorsChanged;
        KeyboardManager.KeyboardClosed += OnVirtualKeyboardClosed;
        _subscribed = true;
    }

    public void Unsubscribe()
    {
        if (!_subscribed)
            return;

        CVRGameEventSystem.Microphone.OnMute.RemoveListener(OnExternalStateEvent);
        CVRGameEventSystem.Microphone.OnUnmute.RemoveListener(OnExternalStateEvent);
        CVRGameEventSystem.World.OnLoad.RemoveListener(OnWorldLoadForHistory);
        CVRGameEventSystem.World.OnReady.RemoveListener(OnWorldReadyForHistory);
        CVRGameEventSystem.Instance.OnConnected.RemoveListener(OnExternalWorldEvent);
        CVRGameEventSystem.Instance.OnDisconnected.RemoveListener(OnExternalWorldEvent);
        CVRGameEventSystem.Player.OnJoinEntity.RemoveListener(OnPlayerJoinedForHistory);
        CVRGameEventSystem.Player.OnLeaveEntity.RemoveListener(OnExternalPlayerEvent);
        CVRGameEventSystem.Avatar.OnLocalAvatarLoad.RemoveListener(OnLocalAvatarLoadForHistory);
        CVRGameEventSystem.Avatar.OnLocalAvatarClear.RemoveListener(OnExternalAvatarEvent);
        CVRGameEventSystem.Avatar.OnRemoteAvatarLoad.RemoveListener(OnRemoteAvatarLoadForHistory);
        CVRGameEventSystem.Spawnable.OnPropSpawned.RemoveListener(OnPropSpawnedForHistory);
        GSInfoHandler.OnGSInfoUpdate -= OnGsInfoUpdated;
        UniversalScaling.RulesChanged -= OnExternalStateEvent;
        PlayerColorsManager.OnLocalPlayerColorChanged -= OnPlayerColorsChanged;
        KeyboardManager.KeyboardClosed -= OnVirtualKeyboardClosed;
        _subscribed = false;
    }

    private void OnExternalStateEvent() => _mod.MarkStateDirty(StateDomain.Core);
    private void OnExternalWorldEvent(string _) => _mod.MarkStateDirty(StateDomain.Core | StateDomain.Social);
    private void OnExternalPlayerEvent(CVRPlayerEntity _) => _mod.MarkStateDirty(StateDomain.Core | StateDomain.Social);
    private void OnExternalAvatarEvent(CVRAvatar _) => _mod.MarkStateDirty(StateDomain.Core | StateDomain.AvatarSettings);

    private void OnWorldLoadForHistory(string worldId)
    {
        _mod.RecentHistory?.RecordWorldVisited(worldId);
        _mod.Content?.RefreshLocalRecentIfActive("worlds");
        _mod.MarkStateDirty(StateDomain.Core | StateDomain.Content);
    }

    private void OnWorldReadyForHistory(string worldId)
    {
        _mod.RecentHistory?.RecordWorldVisited(worldId, MetaPort.Instance?.CurrentWorldName);
        _mod.RecentHistory?.RefreshCurrentInstanceUsers();
        _mod.Content?.RefreshLocalRecentIfActive("worlds");
        _mod.MarkStateDirty(StateDomain.Core | StateDomain.Content | StateDomain.Social);
    }

    private void OnPlayerJoinedForHistory(CVRPlayerEntity player)
    {
        _mod.RecentHistory?.RecordUserSeen(player);
        _mod.MarkStateDirty(StateDomain.Core | StateDomain.Social);
    }

    private void OnLocalAvatarLoadForHistory(CVRAvatar _)
    {
        string avatarId = MetaPort.Instance?.currentAvatarGuid;
        _mod.RecentHistory?.RecordAvatarUsed(avatarId);
        _mod.Content?.RefreshLocalRecentIfActive("avatars");
        _mod.MarkStateDirty(StateDomain.Core | StateDomain.Content | StateDomain.AvatarSettings);
    }

    private void OnRemoteAvatarLoadForHistory(CVRPlayerEntity player, CVRAvatar _)
    {
        if (player != null)
        {
            _mod.RecentHistory?.RecordAvatarSeen(player.ContentMetadata.AssetId, player.ContentMetadata.AssetName);
            _mod.Content?.RefreshLocalRecentIfActive("avatars");
        }
        _mod.MarkStateDirty(StateDomain.Content);
    }

    private void OnPropSpawnedForHistory(string _, PropSoul prop)
    {
        _mod.RecentHistory?.RecordPropSpawned(prop);
        _mod.Content?.RefreshLocalRecentIfActive("props");
        _mod.MarkStateDirty(StateDomain.Content);
    }

    private void OnGsInfoUpdated(ServerInfoPacket _)
    {
        _mod.RecentHistory?.RefreshCurrentInstanceUsers();
        _mod.MarkStateDirty(StateDomain.Core | StateDomain.Social);
    }

    private void OnPlayerColorsChanged(PlayerColors _) => _mod.MarkStateDirty(StateDomain.Core);

    private void OnVirtualKeyboardClosed()
    {
        _mod.ClearSurfaceFocus(NativeBridge.SurfaceMain);
        _mod.ClearSurfaceFocus(NativeBridge.SurfaceQuick);
    }
}
