using System;
using LumenicMenu.Bridge;
using Newtonsoft.Json;
using UnityEngine;

namespace LumenicMenu.Api;

/// <summary>
/// Options for registering a custom Quick Menu or Main Menu extension tab.
/// </summary>
public sealed class ExtTabOptions
{
    public string Id { get; set; }
    public string Title { get; set; }
    public string Subtitle { get; set; }
    /// <summary>Material Symbols Sharp ligature name (e.g. "smart_display", "music_note").</summary>
    public string Icon { get; set; } = "extension";
    /// <summary>Optional texture key registered via <see cref="LumenicApi.RegisterImageBytes"/> or URL.</summary>
    public string IconKey { get; set; }
    /// <summary>Optional dynamic badge string (e.g. queue count).</summary>
    public Func<string> BadgeText { get; set; }
    /// <summary>Higher priority tabs appear earlier.</summary>
    public int Priority { get; set; }
    /// <summary>When true, omits the default Lumenic page header card.</summary>
    public bool HideHeader { get; set; }
    /// <summary>When true, hides this tab from the Quick Menu / Main Menu extension tab bar (open via <see cref="ExtTabHandle.Open"/> from a Quick Tool or slot).</summary>
    public bool HideFromTabBar { get; set; }
    /// <summary>Optional header action buttons rendered on the right side of the page header.</summary>
    public Action<UiBuilder> HeaderActions { get; set; }
}

/// <summary>
/// Options for pushing a sub-page onto a tab's navigation stack.
/// </summary>
public sealed class ExtSubPageOptions
{
    public string Id { get; set; }
    public string Title { get; set; }
    public string Subtitle { get; set; }
    public bool HideHeader { get; set; }
    public Action<UiBuilder> HeaderActions { get; set; }
}

/// <summary>
/// Options for opening a custom extension modal dialog.
/// </summary>
public sealed class ExtModalOptions
{
    public string Id { get; set; } = "modal";
    public string Title { get; set; } = "Dialog";
    public float Width { get; set; } = 460f;
    public bool CloseOnBackdrop { get; set; } = true;
}

/// <summary>
/// Global theme, typography, density, and color overrides pushed via <see cref="LumenicThemeApi"/>.
/// Null properties leave the user's or lower-priority mod's setting untouched.
/// </summary>
public sealed class ExtThemePatch
{
    public float? FontScaleOverride { get; set; }
    public float? FontScaleMultiplier { get; set; }
    public float? TitleScale { get; set; }
    public float? BodyScale { get; set; }
    public float? CaptionScale { get; set; }
    public float? IconScale { get; set; }
    public float? SpacingScale { get; set; }
    public float? PaddingScale { get; set; }
    public float? RadiusScale { get; set; }
    public float? BevelScale { get; set; }
    public float? TileMinHeight { get; set; }
    public int? QmActionColumns { get; set; }
    public float? GlowIntensity { get; set; }
    public float? ShadowStrength { get; set; }
    public float? PanelOpacity { get; set; }

    public string CardBg { get; set; }
    public string CardInnerBg { get; set; }
    public string TileIdleBg { get; set; }
    public string TileIdleHover { get; set; }
    public string BorderSubtle { get; set; }
    public string TextPrimary { get; set; }
    public string TextMuted { get; set; }
    public string TextDim { get; set; }
    public string AccentOverride { get; set; }
}

/// <summary>
/// Section-scoped style/layout overrides for a named built-in section ID.
/// </summary>
public sealed class ExtSectionStyle
{
    [JsonProperty("hidden", DefaultValueHandling = DefaultValueHandling.Ignore)] public bool Hidden { get; set; }
    [JsonProperty("font_scale", NullValueHandling = NullValueHandling.Ignore)] public float? FontScale { get; set; }
    [JsonProperty("spacing_scale", NullValueHandling = NullValueHandling.Ignore)] public float? SpacingScale { get; set; }
    [JsonProperty("padding_scale", NullValueHandling = NullValueHandling.Ignore)] public float? PaddingScale { get; set; }
    [JsonProperty("columns", NullValueHandling = NullValueHandling.Ignore)] public int? Columns { get; set; }
    [JsonProperty("min_height", NullValueHandling = NullValueHandling.Ignore)] public float? MinHeight { get; set; }
    [JsonProperty("card_bg", NullValueHandling = NullValueHandling.Ignore)] public string CardBg { get; set; }
    [JsonProperty("border", NullValueHandling = NullValueHandling.Ignore)] public string Border { get; set; }
}

/// <summary>
/// Well-known shared Quick Menu hub IDs for grouping utility mods under a single tab.
/// </summary>
public static class LumenicHubs
{
    public const string Mods = "lumenic.mods";
}

/// <summary>
/// Options for registering a mod entry inside a shared Quick Menu hub tab.
/// </summary>
public sealed class ExtHubEntryOptions
{
    public string Id { get; set; }
    public string Title { get; set; }
    public string Subtitle { get; set; }
    /// <summary>Material Symbols Sharp ligature name (e.g. "record_voice_over").</summary>
    public string Icon { get; set; } = "extension";
    public Func<string> BadgeText { get; set; }
    public int Priority { get; set; }
    public bool HideHeader { get; set; }
    public Action<UiBuilder> HeaderActions { get; set; }
}

/// <summary>
/// Well-known built-in section IDs that extensions can style, inject into, or replace.
/// </summary>
public static class LumenicSections
{
    public const string QmHomeBanner = "qm.home.banner";
    public const string QmHomeNotifications = "qm.home.notifications";
    public const string QmHomeActions = "qm.home.actions";
    public const string QmPlayerDetailsActions = "qm.player_details.actions";

    public const string MmHomeBanner = "mm.home.banner";
    public const string MmHomeQuickTools = "mm.home.quick_tools";
    public const string MmSettingsLumenic = "mm.settings.lumenic";
}

/// <summary>
/// Handle for a registered extension tab, supporting dirty notification and sub-page navigation.
/// </summary>
public sealed class ExtTabHandle : IDisposable
{
    private readonly ExtensionHost _host;
    public string ModId { get; }
    public string TabId { get; }
    public ExtSurface Surface { get; }

    internal ExtTabHandle(ExtensionHost host, string modId, string tabId, ExtSurface surface)
    {
        _host = host;
        ModId = modId;
        TabId = tabId;
        Surface = surface;
    }

    public void MarkDirty() => _host.MarkDirty();

    public void Open() => _host.SelectTab(Surface, $"{ModId}:{TabId}");

    public void Close() => _host.SelectTab(Surface, null);

    public void PushSubPage(ExtSubPageOptions options, Action<UiBuilder> build)
        => _host.PushSubPage(Surface, ModId, TabId, options, build);

    public void PopSubPage() => _host.PopSubPage(Surface);

    public void Dispose() => _host.UnregisterTab(Surface, ModId, TabId);
}

/// <summary>
/// Handle for a registered section slot or style override.
/// </summary>
public sealed class ExtRegistrationHandle : IDisposable
{
    private readonly Action _dispose;
    private readonly ExtensionHost _host;

    internal ExtRegistrationHandle(ExtensionHost host, Action dispose)
    {
        _host = host;
        _dispose = dispose;
    }

    public void MarkDirty() => _host.MarkDirty();

    public void Dispose()
    {
        _dispose?.Invoke();
        _host.MarkDirty();
    }
}

/// <summary>
/// Global theme, typography, density, and custom font API for LumenicMenu extensions.
/// </summary>
public sealed class LumenicThemeApi
{
    public IDisposable SetOverride(string modId, ExtThemePatch patch, int priority = 0)
        => LumenicApi.Host.SetThemeOverride(modId, patch, priority);

    public bool RegisterFont(string familyName, byte[] fontBytes)
        => LumenicApi.Host.RegisterFont(familyName, fontBytes);

    public bool SetDefaultFontFamily(string familyName)
        => LumenicApi.Host.SetDefaultFontFamily(familyName);
}

/// <summary>
/// Named section styling, injection, replacement, and built-in tile visibility API.
/// </summary>
public sealed class LumenicSectionsApi
{
    public ExtRegistrationHandle Inject(
        string modId,
        string sectionId,
        ExtSlotPosition position,
        Action<UiBuilder> build,
        int priority = 0,
        string slotId = null)
        => LumenicApi.Host.RegisterSlot(modId, sectionId, position, build, priority, slotId);

    public ExtRegistrationHandle Replace(
        string modId,
        string sectionId,
        Action<UiBuilder> build,
        int priority = 0,
        string slotId = null)
        => LumenicApi.Host.RegisterSlot(modId, sectionId, ExtSlotPosition.Replace, build, priority, slotId);

    public ExtRegistrationHandle OverrideStyle(
        string modId,
        string sectionId,
        ExtSectionStyle style,
        int priority = 0)
        => LumenicApi.Host.SetSectionStyle(modId, sectionId, style, priority);

    public ExtRegistrationHandle HideBuiltInTile(string modId, string tileId)
        => LumenicApi.Host.HideBuiltInTile(modId, tileId);
}

/// <summary>
/// Public entry point for supportive ChilloutVR MelonLoader mods extending LumenicMenu.
/// </summary>
public static class LumenicApi
{
    public const int ApiVersion = 1;

    internal static readonly ExtensionHost Host = new ExtensionHost();

    public static bool IsAvailable => LumenicMenuMod.Instance != null;

    /// <summary>True when LumenicMenu Quick Menu replacement is enabled and the native renderer is healthy (or pending lazy boot).</summary>
    public static bool IsQuickMenuActive =>
        LumenicMenuMod.Instance != null && LumenicMenuMod.Instance.IsBridgeHealthyOrPending && Settings.EnableQuickMenu;

    /// <summary>True when LumenicMenu Main Menu replacement is enabled and the native renderer is healthy (or pending lazy boot).</summary>
    public static bool IsMainMenuActive =>
        LumenicMenuMod.Instance != null && LumenicMenuMod.Instance.IsBridgeHealthyOrPending && Settings.EnableMainMenu;

    /// <summary>True when at least one LumenicMenu surface is actively enabled and healthy.</summary>
    public static bool IsActive => IsQuickMenuActive || IsMainMenuActive;

    public static event Action OnReady;
    public static event Action<bool> OnQuickMenuToggled;
    public static event Action<bool> OnMainMenuToggled;

    public static LumenicThemeApi Theme { get; } = new LumenicThemeApi();
    public static LumenicSectionsApi Sections { get; } = new LumenicSectionsApi();

    internal static void NotifyReady()
    {
        try { OnReady?.Invoke(); }
        catch (Exception ex) { MelonLoader.MelonLogger.Error($"[LumenicApi] OnReady subscriber failed: {ex}"); }
    }

    internal static void NotifyQuickMenuToggled(bool open)
    {
        try { OnQuickMenuToggled?.Invoke(open); }
        catch (Exception ex) { MelonLoader.MelonLogger.Error($"[LumenicApi] OnQuickMenuToggled subscriber failed: {ex}"); }
    }

    internal static void NotifyMainMenuToggled(bool open)
    {
        try { OnMainMenuToggled?.Invoke(open); }
        catch (Exception ex) { MelonLoader.MelonLogger.Error($"[LumenicApi] OnMainMenuToggled subscriber failed: {ex}"); }
    }

    public static ExtTabHandle RegisterQuickMenuTab(string modId, ExtTabOptions options, Action<UiBuilder> build)
        => Host.RegisterTab(ExtSurface.QuickMenu, modId, options, build);

    /// <summary>
    /// Registers a mod panel inside a shared Quick Menu hub tab (such as <see cref="LumenicHubs.Mods"/>
    /// or a custom author suite ID) so multiple utility mods share one top-level tab.
    /// </summary>
    public static ExtTabHandle RegisterQuickMenuHubEntry(
        string hubId,
        ExtTabOptions hubOptions,
        string modId,
        ExtHubEntryOptions entry,
        Action<UiBuilder> build)
        => Host.RegisterQuickMenuHubEntry(hubId, hubOptions, modId, entry, build);

    public static ExtTabHandle RegisterMainMenuTab(string modId, ExtTabOptions options, Action<UiBuilder> build)
        => Host.RegisterTab(ExtSurface.MainMenu, modId, options, build);

    public static ExtRegistrationHandle OpenModal(ExtSurface surface, string modId, ExtModalOptions options, Action<UiBuilder> build)
        => Host.OpenModal(surface, modId, options, build);

    public static void CloseModal(ExtSurface surface)
        => Host.CloseModal(surface);

    /// <summary>Thread-safe. Marshals onto Unity's main thread automatically.</summary>
    public static void ShowToast(string text, float durationSeconds = 3.0f)
        => Host.ShowToast(text, durationSeconds);

    /// <summary>
    /// Thread-safe. Opens ChilloutVR's virtual keyboard (with optional password masking)
    /// and invokes <paramref name="onSubmit"/> on Unity's main thread when confirmed.
    /// </summary>
    public static void OpenKeyboard(
        string initialText,
        Action<string> onSubmit,
        string title = "Input",
        string placeholder = "",
        int maxLen = 256,
        bool masked = false,
        bool multiLine = false,
        ExtSurface surface = ExtSurface.QuickMenu)
        => Host.OpenKeyboard(initialText, onSubmit, title, placeholder, maxLen, masked, multiLine, surface);

    public static void RegisterImageBytes(string key, byte[] encodedImageBytes, ExtSurface surface = ExtSurface.Both)
        => Host.RegisterImageBytes(key, encodedImageBytes, surface);

    public static void RegisterImageUrl(string key, string url, ExtSurface surface = ExtSurface.Both)
        => Host.RegisterImageUrl(key, url, surface);

    public static void RegisterRawRgba(string key, uint width, uint height, byte[] rgbaBytes, ExtSurface surface = ExtSurface.Both)
        => Host.RegisterRawRgba(key, width, height, rgbaBytes, surface);

    /// <summary>
    /// Uploads a readable Unity Texture2D (RGBA32 / ARGB32 / RGB24) directly to Lumenic,
    /// flipping vertically from Unity's bottom-left origin to top-left UI coordinates.
    /// </summary>
    public static bool RegisterTexture2D(string key, Texture2D texture, ExtSurface surface = ExtSurface.Both)
    {
        if (string.IsNullOrEmpty(key) || texture == null || texture.width <= 0 || texture.height <= 0)
            return false;

        try
        {
            Color32[] pixels = texture.GetPixels32();
            int w = texture.width;
            int h = texture.height;
            byte[] rgba = new byte[w * h * 4];
            for (int y = 0; y < h; y++)
            {
                int srcRow = (h - 1 - y) * w;
                int dstRow = y * w * 4;
                for (int x = 0; x < w; x++)
                {
                    Color32 c = pixels[srcRow + x];
                    int o = dstRow + x * 4;
                    rgba[o] = c.r;
                    rgba[o + 1] = c.g;
                    rgba[o + 2] = c.b;
                    rgba[o + 3] = c.a;
                }
            }
            Host.RegisterRawRgba(key, (uint)w, (uint)h, rgba, surface);
            return true;
        }
        catch (Exception ex)
        {
            MelonLoader.MelonLogger.Warning($"[LumenicApi] RegisterTexture2D('{key}') failed: {ex.Message}");
            return false;
        }
    }

    public static void UnregisterMod(string modId)
        => Host.UnregisterMod(modId);
}
