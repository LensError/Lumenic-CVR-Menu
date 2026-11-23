using System;
using System.Collections.Generic;
using System.Reflection;
using ABI.CCK.Scripts;
using ABI_RC.Core.Player;
using ABI_RC.Core.Savior;
using ABI_RC.Core.Util.AnimatorManager;
using ABI_RC.Systems.AvatarScaling;
using ABI_RC.Systems.UI.MenuPreview;
using HarmonyLib;
using MelonLoader;
using UnityEngine;

namespace LumenicMenu.Bridge;

/// <summary>
/// Exposes the local avatar's Advanced Avatar Settings (parameters, staged preview values,
/// auto-apply state, and full profile management) and drives CVR's AvatarSettingsPreview rig.
/// </summary>
internal static class AvatarSettingsBridge
{
    private static readonly FieldInfo StagedParamsField =
        AccessTools.Field(typeof(AvatarSettingsPreview), "_stagedParameters");

    private static readonly MethodInfo UpdateRenderingMethod =
        AccessTools.Method(typeof(MenuPreviewRig), "UpdateRendering");

    private static Camera _cachedPreviewCamera;
    private static Camera _cachedWorldPanoCamera;
    private static string _activeWorldPanoUrl = string.Empty;

    public static object BuildSnapshot()
    {
        var entries = new List<object>();
        var profiles = new List<string>();
        bool autoApply = true;
        bool hasStaged = false;
        string currentProfile = string.Empty;
        string stagedProfile = string.Empty;
        float avatarHeight = 1.6f;
        float initialHeight = 1.6f;
        float scaleMin = UniversalScaling.UserMinHeight;
        float scaleMax = UniversalScaling.UserMaxHeight;
        bool scalingAllowed = true;

        try
        {
            autoApply = MetaPort.Instance?.settings?.GetSettingsBool(AvatarSettingsPreview.AutoApplySettingKey, true) ?? true;
            scalingAllowed = UniversalScaling.ManualScalingAllowed;
            scaleMin = UniversalScaling.HeightMinimum;
            scaleMax = Mathf.Max(scaleMin + 0.05f, UniversalScaling.HeightMaximum);
            if (PlayerSetup.Instance != null)
            {
                if (PlayerSetup.Instance.InitialHeight > 0.01f)
                    initialHeight = PlayerSetup.Instance.InitialHeight;
                if (PlayerSetup.Instance.DesiredHeight > 0.01f)
                    avatarHeight = PlayerSetup.Instance.DesiredHeight;
                else
                    avatarHeight = initialHeight;
            }

            AvatarSettingsPreview preview = AvatarSettingsPreview.Instance;
            Dictionary<string, float> stagedParams = null;
            if (preview != null)
            {
                stagedParams = StagedParamsField?.GetValue(preview) as Dictionary<string, float>;
                hasStaged = (stagedParams != null && stagedParams.Count > 0) || preview.IsProfileStaged;
                if (preview.IsProfileStaged)
                {
                    stagedProfile = string.IsNullOrEmpty(preview.StagedProfileName)
                        ? "default"
                        : preview.StagedProfileName;
                }
            }

            PlayerSetup ps = PlayerSetup.Instance;
            if (ps == null || ps.AvatarObject == null || ps.AvatarDescriptor == null || !ps.AvatarDescriptor.avatarUsesAdvancedSettings)
            {
                return new
                {
                    auto_apply = autoApply,
                    has_staged = hasStaged,
                    current_profile = currentProfile,
                    staged_profile = stagedProfile,
                    avatar_height = avatarHeight,
                    initial_height = initialHeight,
                    scale_min = scaleMin,
                    scale_max = scaleMax,
                    scaling_allowed = scalingAllowed,
                    entries,
                    profiles
                };
            }

            currentProfile = ps.CurrentAvatarSettings?.CurrentProfileName ?? CVRAdvancedAvatarSettings.CurrentProfile ?? string.Empty;

            CVRAdvancedSettingsUi[] defs = ps.getCurrentAvatarSettings();
            CVRAnimatorManager am = ps.AnimatorManager;

            if (defs != null)
            {
                for (int i = 0; i < defs.Length; i++)
                {
                    CVRAdvancedSettingsUi d = defs[i];
                    if (string.IsNullOrEmpty(d.parameterName))
                        continue;

                    string kind = (d.type ?? "slider").Trim().ToLowerInvariant();
                    float vx = d.defaultValueX;
                    float vy = d.defaultValueY;
                    float vz = d.defaultValueZ;

                    switch (kind)
                    {
                        case "colorpicker":
                            vx = ReadEffectiveParam(stagedParams, am, d.parameterName + "-r", vx, autoApply);
                            vy = ReadEffectiveParam(stagedParams, am, d.parameterName + "-g", vy, autoApply);
                            vz = ReadEffectiveParam(stagedParams, am, d.parameterName + "-b", vz, autoApply);
                            break;
                        case "joystick2d":
                        case "inputvector2":
                            vx = ReadEffectiveParam(stagedParams, am, d.parameterName + "-x", vx, autoApply);
                            vy = ReadEffectiveParam(stagedParams, am, d.parameterName + "-y", vy, autoApply);
                            break;
                        case "joystick3d":
                        case "inputvector3":
                            vx = ReadEffectiveParam(stagedParams, am, d.parameterName + "-x", vx, autoApply);
                            vy = ReadEffectiveParam(stagedParams, am, d.parameterName + "-y", vy, autoApply);
                            vz = ReadEffectiveParam(stagedParams, am, d.parameterName + "-z", vz, autoApply);
                            break;
                        default:
                            vx = ReadEffectiveParam(stagedParams, am, d.parameterName, vx, autoApply);
                            break;
                    }

                    float minX = d.minValueX;
                    float maxX = d.maxValueX > d.minValueX ? d.maxValueX : d.minValueX + 1f;
                    float minY = d.minValueY;
                    float maxY = d.maxValueY > d.minValueY ? d.maxValueY : d.minValueY + 1f;
                    float minZ = d.minValueZ;
                    float maxZ = d.maxValueZ > d.minValueZ ? d.maxValueZ : d.minValueZ + 1f;

                    entries.Add(new
                    {
                        name = string.IsNullOrWhiteSpace(d.name) ? d.parameterName : d.name,
                        param = d.parameterName,
                        kind,
                        val_x = vx,
                        val_y = vy,
                        val_z = vz,
                        min_x = minX,
                        max_x = maxX,
                        min_y = minY,
                        max_y = maxY,
                        min_z = minZ,
                        max_z = maxZ,
                        options = d.optionList ?? Array.Empty<string>()
                    });
                }
            }

            string[] profArr = ps.getCurrentAvatarSettingsProfiles();
            if (profArr != null)
            {
                for (int i = 0; i < profArr.Length; i++)
                {
                    if (!string.IsNullOrWhiteSpace(profArr[i]))
                        profiles.Add(profArr[i]);
                }
            }
        }
        catch (Exception ex)
        {
            MelonLogger.Warning($"[LumenicMenu] AvatarSettingsBridge.BuildSnapshot failed: {ex.Message}");
        }

        return new
        {
            auto_apply = autoApply,
            has_staged = hasStaged,
            current_profile = currentProfile,
            staged_profile = stagedProfile,
            avatar_height = avatarHeight,
            initial_height = initialHeight,
            scale_min = scaleMin,
            scale_max = scaleMax,
            scaling_allowed = scalingAllowed,
            entries,
            profiles
        };
    }

    public static void SetAvatarHeight(float meters)
    {
        if (float.IsNaN(meters) || float.IsInfinity(meters))
            return;

        try
        {
            UniversalScaling.TrySetHeight(meters, HeightSource.User);
        }
        catch (Exception ex)
        {
            MelonLogger.Warning($"[LumenicMenu] AvatarSettingsBridge.SetAvatarHeight failed: {ex.Message}");
        }
    }

    public static void ResetAvatarHeight()
    {
        try
        {
            if (PlayerSetup.Instance != null && PlayerSetup.Instance.InitialHeight > 0.01f)
            {
                UniversalScaling.TrySetHeight(PlayerSetup.Instance.InitialHeight, HeightSource.User);
            }
        }
        catch (Exception ex)
        {
            MelonLogger.Warning($"[LumenicMenu] AvatarSettingsBridge.ResetAvatarHeight failed: {ex.Message}");
        }
    }

    public static RenderTexture GetPreviewTexture()
    {
        try
        {
            AvatarSettingsPreview preview = AvatarSettingsPreview.Instance;
            if (preview == null)
            {
                _cachedPreviewCamera = null;
                return null;
            }

            if (_cachedPreviewCamera == null)
            {
                _cachedPreviewCamera = preview.GetComponentInChildren<Camera>(includeInactive: true);
            }

            // Use the camera's true Linear ARGBHalf targetTexture directly (not CohtmlLiveView's
            // MenuPreviewEncode sRGB-baked Linear RT, which causes double-gamma gray washout when
            // composited into ViewTextureMips).
            return _cachedPreviewCamera != null ? _cachedPreviewCamera.targetTexture : null;
        }
        catch
        {
            return null;
        }
    }

    public static void RefreshPreviewRendering()
    {
        try
        {
            AvatarSettingsPreview preview = AvatarSettingsPreview.Instance;
            if (preview != null && UpdateRenderingMethod != null)
            {
                UpdateRenderingMethod.Invoke(preview, null);
            }
        }
        catch (Exception ex)
        {
            MelonLogger.Warning($"[LumenicMenu] RefreshPreviewRendering failed: {ex.Message}");
        }
    }

    public static void RotatePreview(float delta, bool isDragging)
    {
        if (float.IsNaN(delta) || float.IsInfinity(delta))
            return;

        try
        {
            AvatarSettingsPreview.Instance?.Rotate(delta, isDragging);
        }
        catch (Exception ex)
        {
            MelonLogger.Warning($"[LumenicMenu] AvatarSettingsBridge.RotatePreview failed: {ex.Message}");
        }
    }

    public static void SyncWorldPano(string url, bool active)
    {
        try
        {
            WorldPanoPreview pano = WorldPanoPreview.Instance;
            if (pano == null)
                return;

            if (!active || string.IsNullOrWhiteSpace(url))
            {
                if (!string.IsNullOrEmpty(_activeWorldPanoUrl))
                {
                    _activeWorldPanoUrl = string.Empty;
                    pano.SetPageOpen(state: false);
                }
                return;
            }

            if (!string.Equals(_activeWorldPanoUrl, url, StringComparison.Ordinal))
            {
                _activeWorldPanoUrl = url;
                pano.Open(url);
            }
        }
        catch (Exception ex)
        {
            MelonLogger.Warning($"[LumenicMenu] AvatarSettingsBridge.SyncWorldPano failed: {ex.Message}");
        }
    }

    public static void LookWorldPano(float deltaX, float deltaY, bool isDragging)
    {
        if (float.IsNaN(deltaX) || float.IsInfinity(deltaX) || float.IsNaN(deltaY) || float.IsInfinity(deltaY))
            return;

        try
        {
            WorldPanoPreview.Instance?.Look(deltaX, deltaY, isDragging);
        }
        catch (Exception ex)
        {
            MelonLogger.Warning($"[LumenicMenu] AvatarSettingsBridge.LookWorldPano failed: {ex.Message}");
        }
    }

    public static RenderTexture GetWorldPanoTexture()
    {
        try
        {
            WorldPanoPreview pano = WorldPanoPreview.Instance;
            if (pano == null)
            {
                _cachedWorldPanoCamera = null;
                return null;
            }

            if (_cachedWorldPanoCamera == null)
            {
                _cachedWorldPanoCamera = pano.GetComponentInChildren<Camera>(includeInactive: true);
            }

            return _cachedWorldPanoCamera != null && _cachedWorldPanoCamera.enabled
                ? _cachedWorldPanoCamera.targetTexture
                : null;
        }
        catch
        {
            return null;
        }
    }

    public static void SetAutoApply(bool enabled)
    {
        try
        {
            MetaPort.Instance?.settings?.SetSettingsBool(AvatarSettingsPreview.AutoApplySettingKey, enabled);
        }
        catch (Exception ex)
        {
            MelonLogger.Warning($"[LumenicMenu] AvatarSettingsBridge.SetAutoApply failed: {ex.Message}");
        }
    }

    public static void ApplyStaged()
    {
        try
        {
            AvatarSettingsPreview.Instance?.ApplyStagedParameters();
        }
        catch (Exception ex)
        {
            MelonLogger.Warning($"[LumenicMenu] AvatarSettingsBridge.ApplyStaged failed: {ex.Message}");
        }
    }

    public static void DiscardStaged()
    {
        try
        {
            AvatarSettingsPreview.Instance?.DiscardStagedParameters();
        }
        catch (Exception ex)
        {
            MelonLogger.Warning($"[LumenicMenu] AvatarSettingsBridge.DiscardStaged failed: {ex.Message}");
        }
    }

    public static void SetParam(string param, float value)
    {
        if (string.IsNullOrWhiteSpace(param) || float.IsNaN(value) || float.IsInfinity(value))
            return;

        try
        {
            AvatarSettingsPreview preview = AvatarSettingsPreview.Instance;
            if (preview == null || !preview.StageParameter(param, value))
            {
                PlayerSetup.Instance?.ChangeAnimatorParam(param, value, PlayerSetup.ParameterChangeSource.MainMenu);
            }
        }
        catch (Exception ex)
        {
            MelonLogger.Warning($"[LumenicMenu] AvatarSettingsBridge.SetParam('{param}') failed: {ex.Message}");
        }
    }

    public static void LoadProfile(string profile)
    {
        if (string.IsNullOrWhiteSpace(profile))
            return;

        try
        {
            AvatarSettingsPreview preview = AvatarSettingsPreview.Instance;
            if (string.Equals(profile, "default", StringComparison.OrdinalIgnoreCase))
            {
                if (preview == null || !preview.StageDefaultProfile())
                {
                    PlayerSetup.Instance?.LoadCurrentAvatarSettingsDefault(openPage: false);
                }
            }
            else
            {
                if (preview == null || !preview.StageProfile(profile))
                {
                    PlayerSetup.Instance?.LoadCurrentAvatarSettingsProfile(profile, openPage: false);
                }
            }
        }
        catch (Exception ex)
        {
            MelonLogger.Warning($"[LumenicMenu] AvatarSettingsBridge.LoadProfile('{profile}') failed: {ex.Message}");
        }
    }

    public static void SaveProfile(string profile)
    {
        string trimmed = profile?.Trim();
        if (string.IsNullOrWhiteSpace(trimmed) || string.Equals(trimmed, "default", StringComparison.OrdinalIgnoreCase))
            return;

        if (trimmed.Length > 12)
            trimmed = trimmed.Substring(0, 12);

        try
        {
            AvatarSettingsPreview.Instance?.ApplyStagedParameters();
            PlayerSetup.Instance?.SaveCurrentAvatarSettingsProfile(trimmed);
        }
        catch (Exception ex)
        {
            MelonLogger.Warning($"[LumenicMenu] AvatarSettingsBridge.SaveProfile('{trimmed}') failed: {ex.Message}");
        }
    }

    public static void DeleteProfile(string profile)
    {
        if (string.IsNullOrWhiteSpace(profile))
            return;

        try
        {
            PlayerSetup.Instance?.DeleteCurrentAvatarSettingsProfile(profile);
        }
        catch (Exception ex)
        {
            MelonLogger.Warning($"[LumenicMenu] AvatarSettingsBridge.DeleteProfile('{profile}') failed: {ex.Message}");
        }
    }

    public static void ReorderProfiles(string[] profiles)
    {
        if (profiles == null || profiles.Length == 0)
            return;

        try
        {
            PlayerSetup.Instance?.CurrentAvatarSettings?.ReorderProfiles(profiles);
        }
        catch (Exception ex)
        {
            MelonLogger.Warning($"[LumenicMenu] AvatarSettingsBridge.ReorderProfiles failed: {ex.Message}");
        }
    }

    public static void ResetDefaults()
    {
        LoadProfile("default");
    }

    private static float ReadEffectiveParam(
        Dictionary<string, float> stagedParams,
        CVRAnimatorManager liveAnimator,
        string name,
        float fallback,
        bool autoApply)
    {
        if (!autoApply && stagedParams != null && stagedParams.TryGetValue(name, out float stagedVal))
            return stagedVal;
        return liveAnimator != null ? ReadParam(liveAnimator, name, fallback) : fallback;
    }

    private static float ReadParam(CVRAnimatorManager am, string name, float fallback)
    {
        if (am.GetParameter(name, out float fVal))
            return fVal;
        if (am.GetParameter(name, out int iVal))
            return iVal;
        if (am.GetParameter(name, out bool bVal))
            return bVal ? 1f : 0f;
        return fallback;
    }
}
