using System;
using ABI_RC.Core.InteractionSystem;
using LumenicMenu.Native;
using UnityEngine;

namespace LumenicMenu.Bridge;

/// <summary>
/// Coordinates 3D local avatar preview, remote player preview rig, and world 360 panorama
/// overlay textures across Main Menu and Quick Menu surfaces.
/// </summary>
internal sealed class PreviewRigCoordinator
{
    private string _mainTarget = "local";
    private string _quickTarget = "local";

    private static bool IsWorldPanoTarget(string target)
    {
        return !string.IsNullOrEmpty(target) && target.StartsWith("worldpano:", StringComparison.Ordinal);
    }

    public bool TryGetActiveOverlayTarget(
        LumenicMenuMod mod,
        MenuSurfaceHost mainSurface,
        MenuSurfaceHost quickSurface,
        uint surfaceId,
        out string target)
    {
        if (surfaceId == NativeBridge.SurfaceMain)
        {
            ViewManager vm = ViewManager.Instance;
            if (vm != null
                && vm.IsViewShown
                && mod.IsLumenicActiveFor(vm.cohtmlView, vm, NativeBridge.SurfaceMain)
                && mainSurface != null
                && mainSurface.IsLiveOverlayActive)
            {
                target = _mainTarget;
                return true;
            }
        }
        else
        {
            CVR_MenuManager qm = CVR_MenuManager.Instance;
            if (qm != null
                && qm.IsViewShown
                && mod.IsLumenicActiveFor(qm.cohtmlView, qm, NativeBridge.SurfaceQuick)
                && quickSurface != null
                && quickSurface.IsLiveOverlayActive)
            {
                target = _quickTarget;
                return true;
            }
        }

        target = null;
        return false;
    }

    public bool ShouldRenderLocalAvatarPreview(LumenicMenuMod mod, MenuSurfaceHost mainSurface, MenuSurfaceHost quickSurface)
    {
        return (TryGetActiveOverlayTarget(mod, mainSurface, quickSurface, NativeBridge.SurfaceMain, out string mmTarget)
                && PlayerPreviewRig.IsLocalTarget(mmTarget))
            || (TryGetActiveOverlayTarget(mod, mainSurface, quickSurface, NativeBridge.SurfaceQuick, out string qmTarget)
                && PlayerPreviewRig.IsLocalTarget(qmTarget));
    }

    public string GetActiveRemoteTarget(LumenicMenuMod mod, MenuSurfaceHost mainSurface, MenuSurfaceHost quickSurface)
    {
        if (TryGetActiveOverlayTarget(mod, mainSurface, quickSurface, NativeBridge.SurfaceMain, out string mmTarget)
            && !PlayerPreviewRig.IsLocalTarget(mmTarget)
            && !IsWorldPanoTarget(mmTarget))
        {
            return mmTarget;
        }

        if (TryGetActiveOverlayTarget(mod, mainSurface, quickSurface, NativeBridge.SurfaceQuick, out string qmTarget)
            && !PlayerPreviewRig.IsLocalTarget(qmTarget)
            && !IsWorldPanoTarget(qmTarget))
        {
            return qmTarget;
        }

        return null;
    }

    private string GetActiveWorldPanoUrl(LumenicMenuMod mod, MenuSurfaceHost mainSurface, MenuSurfaceHost quickSurface)
    {
        if (TryGetActiveOverlayTarget(mod, mainSurface, quickSurface, NativeBridge.SurfaceMain, out string mmTarget)
            && IsWorldPanoTarget(mmTarget))
        {
            return mmTarget.Substring("worldpano:".Length);
        }
        return null;
    }

    public void SyncRemoteAndPano(LumenicMenuMod mod, MenuSurfaceHost mainSurface, MenuSurfaceHost quickSurface)
    {
        string remoteTarget = GetActiveRemoteTarget(mod, mainSurface, quickSurface);
        if (!string.IsNullOrEmpty(remoteTarget))
        {
            PlayerPreviewRig.Instance.SyncTarget(remoteTarget, active: true);
        }
        else if (PlayerPreviewRig.HasInstance)
        {
            PlayerPreviewRig.Instance.SyncTarget(string.Empty, active: false);
        }

        string panoUrl = GetActiveWorldPanoUrl(mod, mainSurface, quickSurface);
        AvatarSettingsBridge.SyncWorldPano(panoUrl, !string.IsNullOrEmpty(panoUrl));
    }

    public RenderTexture GetOverlayTexture(LumenicMenuMod mod, MenuSurfaceHost mainSurface, MenuSurfaceHost quickSurface, uint surfaceId)
    {
        MenuSurfaceHost surf = surfaceId == NativeBridge.SurfaceMain ? mainSurface : quickSurface;
        if (surf == null || !surf.IsLiveOverlayActive)
            return null;

        string target = surfaceId == NativeBridge.SurfaceMain ? _mainTarget : _quickTarget;
        if (IsWorldPanoTarget(target))
            return AvatarSettingsBridge.GetWorldPanoTexture();
        if (PlayerPreviewRig.IsLocalTarget(target))
            return ShouldRenderLocalAvatarPreview(mod, mainSurface, quickSurface) ? AvatarSettingsBridge.GetPreviewTexture() : null;

        return PlayerPreviewRig.HasInstance ? PlayerPreviewRig.Instance.Texture : null;
    }

    public void RotateActive(LumenicMenuMod mod, MenuSurfaceHost mainSurface, MenuSurfaceHost quickSurface, float delta, bool isDragging)
    {
        string remoteTarget = GetActiveRemoteTarget(mod, mainSurface, quickSurface);
        if (!string.IsNullOrEmpty(remoteTarget))
        {
            PlayerPreviewRig.Instance.Rotate(delta, isDragging);
        }
        else
        {
            AvatarSettingsBridge.RotatePreview(delta, isDragging);
        }
    }

    public void SyncSurfacePreview(
        LumenicMenuMod mod,
        MenuSurfaceHost mainSurface,
        MenuSurfaceHost quickSurface,
        uint surfaceId,
        bool active,
        string target,
        float normX,
        float normY,
        float normW,
        float normH)
    {
        if (surfaceId == NativeBridge.SurfaceMain)
            _mainTarget = string.IsNullOrEmpty(target) ? "local" : target;
        else
            _quickTarget = string.IsNullOrEmpty(target) ? "local" : target;

        MenuSurfaceHost surf = surfaceId == NativeBridge.SurfaceMain ? mainSurface : quickSurface;
        surf?.SetLiveOverlay(active, normX, normY, normW, normH);
        AvatarSettingsBridge.RefreshPreviewRendering();
        SyncRemoteAndPano(mod, mainSurface, quickSurface);
    }
}
