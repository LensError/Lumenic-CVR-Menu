using System;
using System.Reflection;
using ABI_RC.Core.InteractionSystem;
using ABI_RC.Core.Player;
using ABI_RC.Core.Savior;
using ABI_RC.Systems.ContentClones;
using HarmonyLib;
using UnityEngine;

namespace LumenicMenu.Bridge;

/// <summary>
/// Hides CVR's world-space MiniMe clone while Lumenic Quick Menu owns the surface
/// (replaced by the GPU-composited turntable preview) and restores MiniMe scale on disable.
/// </summary>
internal sealed class AvatarStageManager : IDisposable
{
    private static readonly FieldInfo MiniMeCloneHolderField =
        AccessTools.Field(typeof(MiniMe), "_cloneHolderObject");

    private static readonly FieldInfo MiniMeAnchorModeField =
        AccessTools.Field(typeof(MiniMe), "_currentAnchorMode");

    public static bool IsMiniMeEnabledInQuickMenu()
    {
        try
        {
            bool settingOn = MetaPort.Instance?.settings != null
                && MetaPort.Instance.settings.GetSettingsBool(MiniMe.SettingMiniMeEnabled);
            if (!settingOn)
                return false;

            if (MiniMeAnchorModeField != null)
            {
                int mode = Convert.ToInt32(MiniMeAnchorModeField.GetValue(null));
                return mode == 0; // AnchorMode.QuickMenu == 0
            }
            return true;
        }
        catch
        {
            return false;
        }
    }

    /// <summary>
    /// Called from the Harmony postfix on MiniMe.UpdateQuickMenuPosition.
    /// Zero-scales MiniMe while Lumenic Quick Menu is active without mutating activeSelf.
    /// </summary>
    public void ApplyMiniMePostfix()
    {
        if (CVR_MenuManager.Instance == null || !CVR_MenuManager.Instance.IsViewShown || MiniMeCloneHolderField == null)
            return;

        GameObject cloneHolder = MiniMeCloneHolderField.GetValue(null) as GameObject;
        if (cloneHolder == null || !cloneHolder.activeSelf)
            return;

        cloneHolder.transform.localScale = Vector3.zero;
    }

    public string QuerySelectedPlayerPreviewStatus()
    {
        return PlayerPreviewRig.QuerySelectedQuickMenuPlayerStatus();
    }

    public void RestoreMiniMeScale()
    {
        try
        {
            if (MiniMeCloneHolderField?.GetValue(null) is GameObject cloneHolder && cloneHolder != null)
            {
                if (cloneHolder.transform.localScale == Vector3.zero)
                {
                    cloneHolder.transform.localScale = Vector3.one;
                }
            }
        }
        catch
        {
            // Ignore if MiniMe is not initialized.
        }
    }

    public void Dispose()
    {
        RestoreMiniMeScale();
    }
}
