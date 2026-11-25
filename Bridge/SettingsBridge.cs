using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Text;
using System.Text.RegularExpressions;
using ABI_RC.Core.Savior;
using HarmonyLib;
using MelonLoader;

namespace LumenicMenu.Bridge;

/// <summary>
/// Phase 3D bridge exposing ALL ChilloutVR settings dynamically from CVRSettings._settings,
/// enriched at runtime with official labels, data-tooltip descriptions, slider bounds, and
/// dropdown choices parsed directly from ChilloutVR's CVRTest/index.html.
/// </summary>
internal static class SettingsBridge
{
    private static readonly FieldInfo SettingsDictField =
        AccessTools.Field(typeof(CVRSettings), "_settings");

    private sealed class OgSettingMeta
    {
        public string Label = string.Empty;
        public string Tooltip = string.Empty;
        public string Category = string.Empty;
        public string Group = string.Empty;
        public int Order = int.MaxValue;
        public bool HasRange;
        public int Min;
        public int Max;
        public (string Val, string Label)[] Choices;
    }

    private static bool _ogLoaded;
    private static readonly Dictionary<string, OgSettingMeta> OgMeta =
        new Dictionary<string, OgSettingMeta>(StringComparer.Ordinal);

    private static readonly Regex HtmlCommentRegex = new Regex(
        @"<!--.*?-->",
        RegexOptions.Singleline | RegexOptions.Compiled);

    private static readonly Regex TokenRegex = new Regex(
        @"<div\s+id=""settings-(?<cat>[a-zA-Z0-9_-]+)""\s+class=""settings-categorie""|" +
        @"<div\s+class=""subcategory-name""[^>]*>(?<subcat>[^<]*)</div>|" +
        @"<div\s+class=""option-caption""(?<capAttrs>[^>]*)>(?<capText>[^<]*)</div>\s*<div\s+class=""option-input""[^>]*>\s*<div\s+(?<inpAttrs>[^>]+)>",
        RegexOptions.Singleline | RegexOptions.IgnoreCase | RegexOptions.Compiled);

    private static readonly Regex WhitespaceRegex = new Regex(@"\s+", RegexOptions.Compiled);

    private static readonly Regex AttrRegex = new Regex(
        @"(?<name>[a-zA-Z0-9_-]+)\s*=\s*""(?<val>[^""]*)""",
        RegexOptions.Singleline | RegexOptions.Compiled);

    private static string MapOgCategory(string ogCat)
    {
        return (ogCat ?? string.Empty).ToLowerInvariant() switch
        {
            "graphics" => "graphics",
            "audio" => "audio",
            "ui" => "general",
            "input" => "controls",
            "filter" => "safety",
            "interaction" => "interactions",
            "implementation" => "implementation",
            "ik" => "ik",
            "experimental" => "experimental",
            _ => "general"
        };
    }

    private static void EnsureOgMetadataLoaded()
    {
        if (_ogLoaded)
            return;
        _ogLoaded = true;

        try
        {
            string htmlPath = Path.Combine(
                MetaPort.APPLICATION_STREAMINGASSETPATH,
                "Cohtml", "UIResources", "CVRTest", "index.html");
            if (!File.Exists(htmlPath))
                return;

            string html = HtmlCommentRegex.Replace(File.ReadAllText(htmlPath, Encoding.UTF8), string.Empty);
            string curCat = string.Empty;
            string curGroup = string.Empty;
            int order = 0;

            foreach (Match m in TokenRegex.Matches(html))
            {
                if (m.Groups["cat"].Success)
                {
                    curCat = MapOgCategory(m.Groups["cat"].Value);
                    curGroup = string.Empty;
                    continue;
                }
                if (m.Groups["subcat"].Success)
                {
                    curGroup = WhitespaceRegex.Replace(DecodeHtml(m.Groups["subcat"].Value), " ").Trim();
                    continue;
                }

                string capAttrs = m.Groups["capAttrs"].Value;
                string capText = WhitespaceRegex.Replace(DecodeHtml(m.Groups["capText"].Value), " ").Trim();
                if (capText.Length > 48)
                    capText = string.Empty;
                string inpAttrs = m.Groups["inpAttrs"].Value;

                string id = GetAttr(inpAttrs, "id");
                if (string.IsNullOrWhiteSpace(id))
                    continue;

                var meta = new OgSettingMeta
                {
                    Label = capText,
                    Tooltip = DecodeHtml(GetAttr(capAttrs, "data-tooltip")).Trim(),
                    Category = curCat,
                    Group = curGroup,
                    Order = order++
                };

                string minStr = GetAttr(inpAttrs, "data-min");
                string maxStr = GetAttr(inpAttrs, "data-max");
                if (int.TryParse(minStr, NumberStyles.Integer, CultureInfo.InvariantCulture, out int minV)
                    && int.TryParse(maxStr, NumberStyles.Integer, CultureInfo.InvariantCulture, out int maxV)
                    && maxV > minV)
                {
                    meta.HasRange = true;
                    meta.Min = minV;
                    meta.Max = maxV;
                }

                string optsStr = GetAttr(inpAttrs, "data-options");
                if (!string.IsNullOrWhiteSpace(optsStr))
                {
                    var pairs = new List<(string, string)>();
                    foreach (string part in optsStr.Split(','))
                    {
                        int colon = part.IndexOf(':');
                        if (colon > 0)
                        {
                            string v = part.Substring(0, colon).Trim();
                            string l = DecodeHtml(part.Substring(colon + 1)).Trim();
                            if (!string.IsNullOrEmpty(v) && !string.IsNullOrEmpty(l))
                                pairs.Add((v, l));
                        }
                    }
                    if (pairs.Count >= 2)
                        meta.Choices = pairs.ToArray();
                }

                OgMeta[id] = meta;
            }
            MelonLogger.Msg($"[LumenicMenu] Parsed {OgMeta.Count} official CVR setting tooltips/groups from CVRTest/index.html.");
        }
        catch (Exception ex)
        {
            MelonLogger.Warning($"[LumenicMenu] Failed to parse OG CVR settings metadata: {ex.Message}");
        }
    }

    private static string GetAttr(string attrs, string name)
    {
        foreach (Match m in AttrRegex.Matches(attrs))
        {
            if (string.Equals(m.Groups["name"].Value, name, StringComparison.OrdinalIgnoreCase))
                return m.Groups["val"].Value;
        }
        return string.Empty;
    }

    private static object _cachedSnapshot;

    public static void InvalidateCache()
    {
        _cachedSnapshot = null;
    }

    private static string DecodeHtml(string s)
    {
        if (string.IsNullOrEmpty(s)) return string.Empty;
        return s
            .Replace("&#10;", "\n")
            .Replace("&#13;", "")
            .Replace("&quot;", "\"")
            .Replace("&apos;", "'")
            .Replace("&#39;", "'")
            .Replace("&lt;", "<")
            .Replace("&gt;", ">")
            .Replace("&amp;", "&");
    }

    private sealed class SliderSpec
    {
        public int Min;
        public int Max;
        public string Unit;
    }

    private static readonly Dictionary<string, SliderSpec> IntSliders = new Dictionary<string, SliderSpec>(StringComparer.Ordinal)
    {
        ["AudioMaster"] = new SliderSpec { Min = 0, Max = 100, Unit = "%" },
        ["AudioPlayerVoice"] = new SliderSpec { Min = 0, Max = 100, Unit = "%" },
        ["AudioWorldSFX"] = new SliderSpec { Min = 0, Max = 100, Unit = "%" },
        ["AudioWorldBGM"] = new SliderSpec { Min = 0, Max = 100, Unit = "%" },
        ["AudioAvatar"] = new SliderSpec { Min = 0, Max = 100, Unit = "%" },
        ["AudioProp"] = new SliderSpec { Min = 0, Max = 100, Unit = "%" },
        ["AudioUi"] = new SliderSpec { Min = 0, Max = 100, Unit = "%" },
        ["AudioVideoPlayer"] = new SliderSpec { Min = 0, Max = 100, Unit = "%" },
        ["AudioChatBox"] = new SliderSpec { Min = 0, Max = 100, Unit = "%" },
        ["AudioTTSVolume"] = new SliderSpec { Min = 0, Max = 100, Unit = "%" },
        ["AudioInputVolume"] = new SliderSpec { Min = 0, Max = 200, Unit = "%" },
        ["AudioInputSensitivity"] = new SliderSpec { Min = 0, Max = 100, Unit = "%" },
        ["AudioMutedSpeechSensitivity"] = new SliderSpec { Min = 0, Max = 100, Unit = "%" },
        ["AudioPlayerVoiceAttenuationDistanceFalloff"] = new SliderSpec { Min = 1, Max = 50, Unit = "m" },
        ["AudioPlayerVoiceAttenuationEarAngle"] = new SliderSpec { Min = 0, Max = 180, Unit = "deg" },
        ["GraphicsFramerateTarget"] = new SliderSpec { Min = 30, Max = 240, Unit = "FPS" },
        ["GraphicsMenuFramerateTarget"] = new SliderSpec { Min = 30, Max = 144, Unit = "FPS" },
        ["GraphicsDesktopFOV"] = new SliderSpec { Min = 60, Max = 110, Unit = "deg" },
        ["GraphicsUIScale"] = new SliderSpec { Min = 50, Max = 150, Unit = "%" },
        ["GeneralPlayerHeight"] = new SliderSpec { Min = 100, Max = 250, Unit = "cm" },
        ["GeneralPortalSafeDistance"] = new SliderSpec { Min = 0, Max = 300, Unit = "cm" },
        ["GeneralMinimumMenuTilt"] = new SliderSpec { Min = 0, Max = 90, Unit = "deg" },
        ["HUDCustomizationOpacity"] = new SliderSpec { Min = 0, Max = 100, Unit = "%" },
        ["ChatBoxBubbleOpacity"] = new SliderSpec { Min = 0, Max = 100, Unit = "%" },
        ["ChatBoxBubbleSize"] = new SliderSpec { Min = 50, Max = 200, Unit = "%" },
        ["ControlTurnSpeed"] = new SliderSpec { Min = 10, Max = 100, Unit = "" },
        ["ControlMouseSensitivity"] = new SliderSpec { Min = 1, Max = 50, Unit = "" },
        ["ControlGamepadSensitivityX"] = new SliderSpec { Min = 5, Max = 100, Unit = "" },
        ["ControlGamepadSensitivityY"] = new SliderSpec { Min = 5, Max = 100, Unit = "" },
        ["ControlIndexGripStrength"] = new SliderSpec { Min = 10, Max = 100, Unit = "%" },
        ["ControlDeadZoneLeft"] = new SliderSpec { Min = 0, Max = 50, Unit = "%" },
        ["ControlDeadZoneRight"] = new SliderSpec { Min = 0, Max = 50, Unit = "%" },
        ["HapticTouchStrength"] = new SliderSpec { Min = 0, Max = 100, Unit = "%" },
        ["HapticTouchMinVibration"] = new SliderSpec { Min = 0, Max = 100, Unit = "%" },
        ["ImplementationFaceTrackingMouthStrength"] = new SliderSpec { Min = 0, Max = 200, Unit = "%" },
        ["ImplementationFaceTrackingGazeSmoothness"] = new SliderSpec { Min = 0, Max = 20, Unit = "" },
        ["IKDesktopBodyLeanWeight"] = new SliderSpec { Min = 0, Max = 100, Unit = "%" },
        ["IKDesktopBodyHeadingLimit"] = new SliderSpec { Min = 0, Max = 90, Unit = "deg" },
        ["IKDesktopPelvisHeadingWeight"] = new SliderSpec { Min = 0, Max = 100, Unit = "%" },
        ["IKDesktopChestHeadingWeight"] = new SliderSpec { Min = 0, Max = 100, Unit = "%" },
        ["IKSpineRelaxIterations"] = new SliderSpec { Min = 1, Max = 30, Unit = "" },
        ["IKMaxSpineAngleFwd"] = new SliderSpec { Min = 0, Max = 90, Unit = "deg" },
        ["IKMaxSpineAngleBack"] = new SliderSpec { Min = 0, Max = 90, Unit = "deg" },
        ["IKMaxNeckAngleFwd"] = new SliderSpec { Min = 0, Max = 90, Unit = "deg" },
        ["IKMaxNeckAngleBack"] = new SliderSpec { Min = 0, Max = 90, Unit = "deg" },
        ["IKStraightSpineAngle"] = new SliderSpec { Min = 0, Max = 60, Unit = "deg" },
        ["ContentFilterBlockAvatarsAtDistanceDistance"] = new SliderSpec { Min = 1, Max = 100, Unit = "m" },
        ["ContentFilterBlockAvatarsMaximumLimit"] = new SliderSpec { Min = 1, Max = 100, Unit = "" },
        ["ContentFilterBlockPropsAtDistanceDistance"] = new SliderSpec { Min = 1, Max = 100, Unit = "m" },
        ["ExperimentalMaxConcurrentDownloads"] = new SliderSpec { Min = 1, Max = 10, Unit = "" }
    };

    private static readonly Dictionary<string, (string Val, string Label)[]> ChoiceSpecs =
        new Dictionary<string, (string, string)[]>(StringComparer.Ordinal)
        {
            ["GeneralClockFormat"] = new[] { ("24", "24h"), ("12", "12h") },
            ["GraphicsMsaaLevel"] = new[] { ("0", "Off"), ("2", "2x"), ("4", "4x"), ("8", "8x") },
            ["CameraMsaaLevel"] = new[] { ("0", "Off"), ("2", "2x"), ("4", "4x"), ("8", "8x") },
            ["MirrorMsaaLevel"] = new[] { ("0", "Off"), ("2", "2x"), ("4", "4x"), ("8", "8x") },
            ["GraphicsMaxMirrorResolution"] = new[] { ("512", "512"), ("1024", "1024"), ("2048", "2048"), ("4096", "4096") },
            ["ControlTurnDegree"] = new[] { ("15", "15"), ("30", "30"), ("45", "45"), ("60", "60"), ("90", "90") },
            ["AudioMuteMode"] = new[] { ("0", "Toggle"), ("1", "PTT"), ("2", "PTM") },
            ["IKCalibrationMode"] = new[] { ("0", "T-Pose"), ("1", "Calib 1"), ("2", "Follow") }
        };

    private static readonly (string Val, string Label)[] ContentFilterChoices =
    {
        ("0", "Off"),
        ("1", "Friends"),
        ("2", "All")
    };

    public static object BuildSnapshot()
    {
        if (_cachedSnapshot != null)
            return _cachedSnapshot;

        EnsureOgMetadataLoaded();
        var items = new List<object>();
        var settings = MetaPort.Instance?.settings;
        if (settings == null || SettingsDictField == null)
        {
            return new { items };
        }

        if (SettingsDictField.GetValue(settings) is not Dictionary<string, CVRSettingsValue> dict)
        {
            return new { items };
        }

        var ordered = dict
            .OrderBy(k => OgMeta.TryGetValue(k.Key, out var m) ? m.Order : int.MaxValue)
            .ThenBy(k => k.Key, StringComparer.OrdinalIgnoreCase);

        foreach (var kv in ordered)
        {
            string key = kv.Key;
            CVRSettingsValue valObj = kv.Value;
            if (string.IsNullOrEmpty(key) || valObj == null)
                continue;

            if (IsHiddenInternalSetting(key))
            {
                continue;
            }

            OgMeta.TryGetValue(key, out OgSettingMeta og);
            string category = (og != null && !string.IsNullOrEmpty(og.Category)) ? og.Category : CategorizeKey(key);
            string group = og != null ? (string.IsNullOrEmpty(og.Group) ? "General" : og.Group) : "Other";
            string label = (og != null && !string.IsNullOrWhiteSpace(og.Label)) ? og.Label : FormatLabel(key);
            string tooltip = og?.Tooltip ?? string.Empty;

            if (valObj is CVRSettingsBool)
            {
                bool curBool = settings.GetSettingsBool(key);
                items.Add(new
                {
                    key,
                    label,
                    tooltip,
                    category,
                    group,
                    kind = "bool",
                    bool_val = curBool,
                    int_val = 0,
                    float_val = 0f,
                    str_val = string.Empty,
                    min_val = 0,
                    max_val = 1,
                    unit = string.Empty,
                    choice_vals = Array.Empty<string>(),
                    choice_labels = Array.Empty<string>()
                });
            }
            else if (valObj is CVRSettingsInt)
            {
                int curInt = settings.GetSettingsInt(key);
                (string Val, string Label)[] choices = og?.Choices;
                if (choices == null && ChoiceSpecs.TryGetValue(key, out var fallbackChoices))
                    choices = fallbackChoices;

                if (choices != null)
                {
                    items.Add(new
                    {
                        key,
                        label,
                        tooltip,
                        category,
                        group,
                        kind = "int_choice",
                        bool_val = false,
                        int_val = curInt,
                        float_val = 0f,
                        str_val = curInt.ToString(CultureInfo.InvariantCulture),
                        min_val = 0,
                        max_val = 0,
                        unit = string.Empty,
                        choice_vals = choices.Select(c => c.Val).ToArray(),
                        choice_labels = choices.Select(c => c.Label).ToArray()
                    });
                }
                else if (key.StartsWith("ContentFilter", StringComparison.Ordinal)
                    && !key.Contains("Distance") && !key.Contains("Limit") && !key.Contains("MaxDLSize"))
                {
                    items.Add(new
                    {
                        key,
                        label,
                        tooltip,
                        category,
                        group,
                        kind = "int_choice",
                        bool_val = false,
                        int_val = curInt,
                        float_val = 0f,
                        str_val = curInt.ToString(CultureInfo.InvariantCulture),
                        min_val = 0,
                        max_val = 2,
                        unit = string.Empty,
                        choice_vals = ContentFilterChoices.Select(c => c.Val).ToArray(),
                        choice_labels = ContentFilterChoices.Select(c => c.Label).ToArray()
                    });
                }
                else if ((og != null && og.HasRange) || IntSliders.ContainsKey(key))
                {
                    int minV = (og != null && og.HasRange) ? og.Min : IntSliders[key].Min;
                    int maxV = (og != null && og.HasRange) ? og.Max : IntSliders[key].Max;
                    string unit = IntSliders.TryGetValue(key, out var spec) ? spec.Unit : string.Empty;
                    items.Add(new
                    {
                        key,
                        label,
                        tooltip,
                        category,
                        group,
                        kind = "int_slider",
                        bool_val = false,
                        int_val = curInt,
                        float_val = 0f,
                        str_val = string.Empty,
                        min_val = minV,
                        max_val = maxV,
                        unit,
                        choice_vals = Array.Empty<string>(),
                        choice_labels = Array.Empty<string>()
                    });
                }
                else
                {
                    items.Add(new
                    {
                        key,
                        label,
                        tooltip,
                        category,
                        group,
                        kind = "int_step",
                        bool_val = false,
                        int_val = curInt,
                        float_val = 0f,
                        str_val = string.Empty,
                        min_val = 0,
                        max_val = 0,
                        unit = string.Empty,
                        choice_vals = Array.Empty<string>(),
                        choice_labels = Array.Empty<string>()
                    });
                }
            }
            else if (valObj is CVRSettingsFloat)
            {
                float curFloat = settings.GetSettingsFloat(key);
                items.Add(new
                {
                    key,
                    label,
                    tooltip,
                    category,
                    group,
                    kind = "float_step",
                    bool_val = false,
                    int_val = 0,
                    float_val = curFloat,
                    str_val = string.Empty,
                    min_val = 0,
                    max_val = 0,
                    unit = string.Empty,
                    choice_vals = Array.Empty<string>(),
                    choice_labels = Array.Empty<string>()
                });
            }
            else if (valObj is CVRSettingsString)
            {
                string curStr = settings.GetSettingsString(key) ?? string.Empty;
                (string Val, string Label)[] choices = og?.Choices;
                if (choices == null && ChoiceSpecs.TryGetValue(key, out var fallbackChoices))
                    choices = fallbackChoices;

                if (choices != null)
                {
                    items.Add(new
                    {
                        key,
                        label,
                        tooltip,
                        category,
                        group,
                        kind = "str_choice",
                        bool_val = false,
                        int_val = 0,
                        float_val = 0f,
                        str_val = curStr,
                        min_val = 0,
                        max_val = 0,
                        unit = string.Empty,
                        choice_vals = choices.Select(c => c.Val).ToArray(),
                        choice_labels = choices.Select(c => c.Label).ToArray()
                    });
                }
                else
                {
                    items.Add(new
                    {
                        key,
                        label,
                        tooltip,
                        category,
                        group,
                        kind = "string_ro",
                        bool_val = false,
                        int_val = 0,
                        float_val = 0f,
                        str_val = curStr,
                        min_val = 0,
                        max_val = 0,
                        unit = string.Empty,
                        choice_vals = Array.Empty<string>(),
                        choice_labels = Array.Empty<string>()
                    });
                }
            }
        }

        var snapshot = new { items };
        _cachedSnapshot = snapshot;
        return snapshot;
    }

    public static void SetBool(string key, bool value)
    {
        if (!string.IsNullOrWhiteSpace(key))
        {
            MetaPort.Instance?.settings?.SetSetting(key, value ? "True" : "False");
            InvalidateCache();
        }
    }

    public static void SetInt(string key, int value)
    {
        if (!string.IsNullOrWhiteSpace(key))
        {
            MetaPort.Instance?.settings?.SetSetting(key, value.ToString(CultureInfo.InvariantCulture));
            InvalidateCache();
        }
    }

    public static void SetFloat(string key, float value)
    {
        if (!string.IsNullOrWhiteSpace(key) && !float.IsNaN(value) && !float.IsInfinity(value))
        {
            MetaPort.Instance?.settings?.SetSetting(key, value.ToString("0.###", CultureInfo.InvariantCulture));
            InvalidateCache();
        }
    }

    public static void SetString(string key, string value)
    {
        if (!string.IsNullOrWhiteSpace(key))
        {
            MetaPort.Instance?.settings?.SetSetting(key, value ?? string.Empty);
            InvalidateCache();
        }
    }

    private static readonly HashSet<string> HiddenExactKeys = new HashSet<string>(StringComparer.Ordinal)
    {
        "GeneralVideoPlayerTrustedHosts",
        "GeneralUIPrimaryColourMaxLuma",
        "GeneralUIColourizeMenu",
        "GeneralUIGroupModerationNeverShow",
        "GeneralUIChangelogVersion",
        "NameplateRank",
        "MiniMeEnabled",
        "QuickMenuAnchorToHand",
        "AvatarSettingsAutoApply",
        "AvatarHeight",
        "IncludeNSFWInSearch",
        "IncludeNSFWInDiscover",
        "PlayerColorUsePrimaryAsEmission",
        "PenSelectedPalette",
        "PenThickness",
        "PenSmoothness",
        "PenAnalogThickness",
        "CVRVideoPlayerDefaultResolution",
        "CVRVideoPlayerDefaultAudioOnly",
        "CVRVideoPlayerUseLowLatency",
        "ExperimentalConceptMirror"
    };

    private static bool IsHiddenInternalSetting(string key)
    {
        if (HiddenExactKeys.Contains(key))
            return true;

        if (key.EndsWith("-Last", StringComparison.Ordinal)
            || key.EndsWith("Last", StringComparison.Ordinal)
            || key.EndsWith("Vr", StringComparison.Ordinal)
            || key.EndsWith("Desktop", StringComparison.Ordinal))
            return true;

        return key.StartsWith("Version", StringComparison.Ordinal)
            || key.StartsWith("SelfModeration", StringComparison.Ordinal)
            || key.StartsWith("InstanceCreationInitialSetting", StringComparison.Ordinal)
            || key.StartsWith("HomeInstanceCreationSetting", StringComparison.Ordinal)
            || key.StartsWith("Keyboard", StringComparison.Ordinal)
            || key.StartsWith("PersonalMirror", StringComparison.Ordinal)
            || key.StartsWith("SelfPortrait", StringComparison.Ordinal)
            || key.StartsWith("ComfortAlignment", StringComparison.Ordinal)
            || key.StartsWith("PlayerColorPrimary", StringComparison.Ordinal)
            || key.StartsWith("PlayerColorSecondary", StringComparison.Ordinal)
            || key.StartsWith("PenColorPalette", StringComparison.Ordinal)
            || key.StartsWith("PortableCamera_", StringComparison.Ordinal)
            || key.StartsWith("StereoscopicCapture_", StringComparison.Ordinal)
            || key.StartsWith("CaptureIntegrations_", StringComparison.Ordinal)
            || key.StartsWith("GraphicsTransitionColor", StringComparison.Ordinal)
            || key.StartsWith("Last", StringComparison.Ordinal)
            || key.StartsWith("ContentSort", StringComparison.Ordinal);
    }

    private static string CategorizeKey(string key)
    {
        if (key.StartsWith("Audio", StringComparison.Ordinal)) return "audio";
        if (key.StartsWith("Graphics", StringComparison.Ordinal) || key.EndsWith("MsaaLevel", StringComparison.Ordinal) || key.StartsWith("Portal", StringComparison.Ordinal)) return "graphics";
        if (key.StartsWith("Control", StringComparison.Ordinal) || key.StartsWith("FingerGesture", StringComparison.Ordinal) || key.StartsWith("GravityRotation", StringComparison.Ordinal)) return "controls";
        if (key.StartsWith("IK", StringComparison.Ordinal) || key == "GeneralEnableRunningAnimationFullBody") return "ik";
        if (key.StartsWith("HUD", StringComparison.Ordinal) || key.StartsWith("Hud", StringComparison.Ordinal) || key.StartsWith("Nameplate", StringComparison.Ordinal) || key.StartsWith("MessageSystem", StringComparison.Ordinal)) return "hud";
        if (key.StartsWith("Interaction", StringComparison.Ordinal) || key.StartsWith("Haptic", StringComparison.Ordinal)) return "interactions";
        if (key.StartsWith("Implementation", StringComparison.Ordinal)) return "implementation";
        if (key.StartsWith("ContentFilter", StringComparison.Ordinal) || key.StartsWith("FilterUser", StringComparison.Ordinal) || key.Contains("NSFW")) return "safety";
        if (key.StartsWith("Experimental", StringComparison.Ordinal)) return "experimental";
        return "general";
    }

    private static string FormatLabel(string key)
    {
        string trimmed = key;
        string[] prefixes =
        {
            "HUDCustomization", "HudCustomization", "NameplateCustomization", "NameplateLocalCustomization",
            "MessageSystemQM", "ContentFilterProps", "ContentFilter", "ImplementationRichPresence",
            "ImplementationFaceTracking", "Implementation", "InteractionDynamicBoneInterCollision",
            "InteractionDynamicBone", "InteractionHapticsTriggers", "InteractionProps", "Interaction",
            "ExperimentalAdvancedSafety", "ExperimentalBundleVerifier", "ExperimentalGizmos", "Experimental",
            "AudioPlayerVoiceAttenuation", "AudioInput", "Audio", "Graphics", "General", "Control",
            "TextComms", "ChatBox", "HapticTouch", "IKDesktop", "IK"
        };
        foreach (string p in prefixes)
        {
            if (trimmed.StartsWith(p, StringComparison.Ordinal) && trimmed.Length > p.Length)
            {
                string sub = trimmed.Substring(p.Length);
                if (p == "ContentFilterProps") trimmed = "Props " + sub;
                else if (p == "IKDesktop") trimmed = "Desktop " + sub;
                else if (p == "AudioInput") trimmed = "Mic " + sub;
                else if (p == "AudioPlayerVoiceAttenuation") trimmed = "Voice " + sub;
                else if (p == "ImplementationRichPresence") trimmed = "Rich Presence " + sub;
                else if (p == "ImplementationFaceTracking") trimmed = "Face Track " + sub;
                else if (p == "ExperimentalAdvancedSafety") trimmed = "Adv Safety " + sub;
                else if (p == "ExperimentalBundleVerifier") trimmed = "Verifier " + sub;
                else if (p == "ExperimentalGizmos") trimmed = "Gizmo " + sub;
                else if (p == "InteractionDynamicBoneInterCollision") trimmed = "DynBone " + sub;
                else if (p == "InteractionHapticsTriggers") trimmed = "Haptic " + sub;
                else trimmed = sub;
                break;
            }
        }

        var sb = new StringBuilder(trimmed.Length + 8);
        for (int i = 0; i < trimmed.Length; i++)
        {
            char c = trimmed[i];
            if (i > 0 && char.IsUpper(c) && (char.IsLower(trimmed[i - 1]) || (i + 1 < trimmed.Length && char.IsLower(trimmed[i + 1]))))
            {
                if (sb.Length > 0 && sb[sb.Length - 1] != ' ')
                    sb.Append(' ');
            }
            sb.Append(c);
        }
        return sb.ToString().Trim();
    }
}
