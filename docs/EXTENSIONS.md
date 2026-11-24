# LumenicMenu Extension API Guide

LumenicMenu includes a built-in **MelonLoader C# Extension API** (`LumenicMenu.Api`) that lets third-party ChilloutVR mods:
- Add rich custom **Quick Menu** (`800x800`) and **Main Menu** (`1280x720`) tabs and sub-page stacks using a full declarative `UiBuilder` (layouts, cards, grids, scroll views, sliders, inputs, progress bars, badges, images, icons, custom modals).
- Inject UI into, restyle, hide, or completely replace well-known built-in Lumenic sections (`qm.home.banner`, `qm.home.actions`, `qm.player_details.actions`, `mm.settings.lumenic`, etc.).
- Override global typography, font scale, density/spacing, colors, panel opacity, and register custom `.ttf`/`.otf` fonts via `LumenicApi.Theme`.
- Upload PNG/JPG images, remote URLs, raw RGBA buffers, or live Unity `Texture2D` frames.

Extension updates travel over a dedicated `PushExtensionsJson` FFI channel and only rebuild currently visible pages/slots when `MarkDirty()` is called, preserving LumenicMenu's idle-frame skipping.

---

## 1. Project Setup (Hard or Soft Dependency)

Extensions are normal **MelonLoader** `.dll` mods placed in `ChilloutVR/Mods/`.

1. Add a reference to `LumenicMenu.dll` (`<Private>false</Private>`) in your `.csproj`.
2. If your mod should also run when LumenicMenu is not installed, mark it as an optional dependency and isolate all `LumenicMenu.Api` calls in a helper class with `[MethodImpl(MethodImplOptions.NoInlining)]`:

```csharp
using System.Linq;
using System.Runtime.CompilerServices;
using MelonLoader;

[assembly: MelonOptionalDependencies("LumenicMenu")]

public class MyVideoMod : MelonMod
{
    public override void OnLateInitializeMelon()
    {
        if (MelonBase.RegisteredMelons.Any(m => m.Info.Name == "LumenicMenu"))
        {
            LumenicIntegration.Initialize(this);
        }
    }
}

internal static class LumenicIntegration
{
    [MethodImpl(MethodImplOptions.NoInlining)]
    public static void Initialize(MyVideoMod mod)
    {
        if (LumenicMenu.Api.LumenicApi.ApiVersion < 1)
            return;

        // Register tabs, hub entries, slots, or theme overrides here.
        // Note: Registering with LumenicApi does NOT require skipping BTKUILib.
        // Keeping BTKUILib registered lets users toggle back to vanilla CVR menus seamlessly.
    }
}
```

### Runtime Active Checks & Fallback

- `LumenicApi.IsAvailable` : True when `LumenicMenu` is loaded.
- `LumenicApi.IsQuickMenuActive` : True when the native renderer is healthy **and** Lumenic Quick Menu is enabled (`Settings.EnableQuickMenu`).
- `LumenicApi.IsMainMenuActive` : True when the native renderer is healthy **and** Lumenic Main Menu is enabled (`Settings.EnableMainMenu`).
- `LumenicApi.IsActive` : `IsQuickMenuActive || IsMainMenuActive`.

If your mod wants to skip BTKUILib only when Lumenic is actually running, check `LumenicApi.IsQuickMenuActive` (or subscribe to `LumenicApi.OnReady`).

### Shared Quick Menu Hub (`RegisterQuickMenuHubEntry`) & Main Menu Tabs

For smaller utility mods (or a suite of mods sharing one tab like `LensErrorsMods`), avoid crowding the top-level Quick Menu tab bar by registering inside a shared hub tab:

```csharp
// Groups under the shared "Extensions" tab (LumenicHubs.Mods) or your own custom hub ID:
ExtTabHandle handle = LumenicApi.RegisterQuickMenuHubEntry(
    hubId: LumenicHubs.Mods, // or "com.myname.suite"
    hubOptions: null,        // null uses default "Extensions" tab; pass ExtTabOptions for a custom suite tab
    modId: "com.example.externaltts",
    entry: new ExtHubEntryOptions
    {
        Id = "main",
        Title = "ExternalTTS",
        Subtitle = "Kokoro / Fish Audio TTS",
        Icon = "record_voice_over",
        BadgeText = () => isSpeaking ? "LIVE" : null,
        Priority = 10
    },
    build: BuildTtsPage);
```

For full widescreen (`1280x720`) Main Menu tabs:

```csharp
ExtTabHandle mmTab = LumenicApi.RegisterMainMenuTab(
    modId: "com.example.mymod",
    options: new ExtTabOptions { Id = "dashboard", Title = "My Mod", Icon = "dashboard" },
    build: BuildMainMenuPage);
```

---

## 2. Example: VideoPlayer Remote Quick Menu Tab + Home Banner

This shows how to build a custom Quick Menu tab with cover art, scrub slider, transport controls, URL input, segmented tabs, a scrollable queue, and a compact "Now Playing" banner on QM Home:

```csharp
using System.Collections.Generic;
using LumenicMenu.Api;

public sealed class VideoRemoteUi
{
    private const string ModId = "com.example.videoremote";

    private ExtTabHandle _tab;
    private ExtRegistrationHandle _banner;

    private bool _playing = true;
    private float _positionSec = 84f;
    private float _durationSec = 225f;
    private float _volume = 75f;
    private int _subTab = 0;
    private string _urlInput = string.Empty;
    private readonly List<string> _queue = new() { "Track 1", "Track 2" };

    public void Register()
    {
        // Optional: load remote thumbnail or local PNG bytes
        LumenicApi.RegisterImageUrl("ext:video_thumb", "https://example.com/thumb.jpg");

        // 1. Register a full Quick Menu Tab
        _tab = LumenicApi.RegisterQuickMenuTab(ModId, new ExtTabOptions
        {
            Id = "remote",
            Title = "Video",
            Subtitle = "World VideoPlayer Remote",
            Icon = "smart_display", // Any Google Material Symbols Sharp name
            BadgeText = () => _queue.Count > 0 ? _queue.Count.ToString() : null,
            Priority = 10
        }, BuildRemotePage);

        // 2. Inject a compact "Now Playing" strip onto Quick Menu Home
        _banner = LumenicApi.Sections.Inject(
            ModId,
            LumenicSections.QmHomeBanner,
            ExtSlotPosition.InsideEnd,
            BuildNowPlayingBanner,
            priority: 10);
    }

    private void BuildNowPlayingBanner(UiBuilder ui)
    {
        ui.Card(card =>
        {
            card.Row(row =>
            {
                row.Icon("smart_display", size: 16f, fg: "accent");
                row.Col(col =>
                {
                    col.Text("Now Playing: Synthwave Mix", size: 11.5f, weight: "semibold", ellipsis: true);
                    col.ProgressBar(_positionSec, 0f, _durationSec, style: new ExtStyle { H = 4f });
                }, new ExtStyle { Grow = 1f, Gap = 3f });

                row.IconButton("bn_play", _playing ? "pause" : "play_arrow", () =>
                {
                    _playing = !_playing;
                    RefreshAll();
                }, variant: "accent");

                row.Button("bn_open", "Remote", () => _tab.Open(), icon: "open_in_new");
            }, new ExtStyle { Align = "center", Gap = 8f });
        }, new ExtStyle { Px = 10f, Py = 7f });
    }

    private void BuildRemotePage(UiBuilder ui)
    {
        // Now Playing Card
        ui.Card(card =>
        {
            card.Row(r =>
            {
                r.Image("ext:video_thumb", ExtImageFit.Cover, fallbackIcon: "smart_display",
                    style: new ExtStyle { W = 128f, H = 72f, Radius = 6f });

                r.Col(col =>
                {
                    col.Row(top =>
                    {
                        top.Text("Synthwave ChilloutVR Mix", size: 13.5f, weight: "bold", ellipsis: true,
                            style: new ExtStyle { Grow = 1f });
                        top.Badge(_playing ? "Playing" : "Paused", fg: _playing ? "emerald" : "amber");
                    }, new ExtStyle { Align = "center", Gap = 6f });

                    col.Text("World Synced VideoPlayer", size: 10.5f, fg: "muted");
                    col.Slider("scrub", $"{FormatTime(_positionSec)} / {FormatTime(_durationSec)}",
                        _positionSec, 0f, _durationSec,
                        onChange: v => { _positionSec = v; RefreshAll(); },
                        style: new ExtStyle { WFrac = 1f });
                }, new ExtStyle { Grow = 1f, Gap = 4f });
            }, new ExtStyle { Gap = 10f, Align = "center" });

            // Transport & Volume Row
            card.Row(r =>
            {
                r.IconButton("prev", "skip_previous", () => { _positionSec = 0f; RefreshAll(); });
                r.IconButton("play", _playing ? "pause" : "play_arrow", () =>
                {
                    _playing = !_playing;
                    RefreshAll();
                }, variant: "accent", style: new ExtStyle { W = 44f });
                r.IconButton("next", "skip_next", () => { RefreshAll(); });

                r.Slider("vol", $"Vol {(int)_volume}%", _volume, 0f, 100f,
                    onChange: v => { _volume = v; RefreshAll(); },
                    step: 1f,
                    style: new ExtStyle { Grow = 1f });
            }, new ExtStyle { Align = "center", Gap = 6f });
        });

        // URL Input Bar
        ui.Row(r =>
        {
            r.TextInput("url", _urlInput, placeholder: "Paste video URL...",
                onChange: s => _urlInput = s,
                onSubmit: s => { _queue.Add(s); _urlInput = string.Empty; RefreshAll(); },
                style: new ExtStyle { Grow = 1f });
            r.Button("add", "Queue", () =>
            {
                if (!string.IsNullOrWhiteSpace(_urlInput))
                {
                    _queue.Add(_urlInput.Trim());
                    _urlInput = string.Empty;
                    RefreshAll();
                }
            }, icon: "add", variant: "accent");
        }, new ExtStyle { Align = "center", Gap = 6f });

        // Segmented Tabs + Scrollable Queue
        ui.Tabs("subtabs", new[] { $"Queue ({_queue.Count})", "Settings" }, _subTab,
            idx => { _subTab = idx; RefreshAll(); });

        ui.Scroll("queue_scroll", list =>
        {
            if (_subTab == 0)
            {
                for (int i = 0; i < _queue.Count; i++)
                {
                    int idx = i;
                    list.Row(row =>
                    {
                        row.Text(_queue[idx], size: 11.5f, weight: "semibold", ellipsis: true,
                            style: new ExtStyle { Grow = 1f });
                        row.IconButton($"rm_{idx}", "delete", () =>
                        {
                            _queue.RemoveAt(idx);
                            RefreshAll();
                        }, variant: "danger");
                    }, new ExtStyle { Align = "center", Gap = 6f, Px = 9f, Py = 6f, Bg = "card_inner", Radius = 6f });
                }
            }
            else
            {
                list.Toggle("auto_sync", "Auto Resync", true, _ => RefreshAll(), icon: "sync");
            }
        }, new ExtStyle { Grow = 1f, Gap = 5f });
    }

    private void RefreshAll()
    {
        _tab?.MarkDirty();
        _banner?.MarkDirty();
    }

    private static string FormatTime(float sec)
    {
        int s = (int)sec;
        return $"{s / 60:00}:{s % 60:00}";
    }
}
```

---

## 3. Global Theme, Font Scale & Layout Overrides (`LumenicApi.Theme`)

LumenicMenu ships built-in **Text Size** presets in Settings (`75%`, `100%`, `125%`, `150%`). Mods that need wider ranges, accessibility presets, custom fonts, or custom density/layout adjustments can push an `ExtThemePatch`:

```csharp
var handle = LumenicApi.Theme.SetOverride("com.example.bigtext", new ExtThemePatch
{
    FontScaleOverride = 1.65f,   // Override beyond the built-in 150% slider (0.5 - 2.5)
    TileMinHeight = 68f,         // Give Quick Actions tiles extra height so large text fits cleanly
    SpacingScale = 1.15f,        // Adjust gaps proportionally
    PaddingScale = 1.1f,
    QmActionColumns = 2,         // Switch Quick Actions grid from 3x3 to 2 columns
    PanelOpacity = 0.92f,
    AccentOverride = "#38BDF8"   // Optional hex color (#RRGGBB or #RRGGBBAA)
}, priority: 10);

// Load a custom TTF/OTF font and optionally make it the default:
byte[] fontBytes = System.IO.File.ReadAllBytes("MyFont.ttf");
LumenicApi.Theme.RegisterFont("MyCustomFont", fontBytes);
LumenicApi.Theme.SetDefaultFontFamily("MyCustomFont");
```

Dispose the returned `IDisposable` handle (or call `LumenicApi.UnregisterMod(modId)`) to revert cleanly.

---

## 4. Named Section Overrides, Injection & Replacement (`LumenicApi.Sections`)

Built-in regions have stable IDs defined in `LumenicSections`:

| Section ID | Location | Supports |
| :--- | :--- | :--- |
| `qm.home.banner` | Top of Quick Menu Home | Inject (`Before` / `After` / `InsideStart` / `InsideEnd` / `Replace`) |
| `qm.home.notifications` | Quick Menu Home Notifications card | `OverrideStyle`, `Inject`, `Replace`, `Hidden` |
| `qm.home.actions` | Quick Menu Home Quick Actions grid | `OverrideStyle` (incl. `Columns`, `MinHeight`, `FontScale`), `Inject`, `Replace`, `Hidden` |
| `qm.player_details.actions` | Quick Menu Player Details page | `Inject` custom player action buttons/cards |
| `mm.home.banner` | Top of Main Menu Home | `Inject` custom status/media strip |
| `mm.settings.lumenic` | Inside Main Menu -> Settings -> LumenicMenu | `Inject` custom mod settings cards |

Examples:
```csharp
// Override font scale and columns for Quick Actions only:
LumenicApi.Sections.OverrideStyle("my.mod", LumenicSections.QmHomeActions, new ExtSectionStyle
{
    FontScale = 1.25f,
    Columns = 2,
    MinHeight = 64f
});

// Hide a built-in Quick Actions tile ("mute", "flight", "seated", "camera", "mirror", "respawn", "home", "chat", "fbt"):
LumenicApi.Sections.HideBuiltInTile("my.mod", "chat");

// Completely replace a section:
LumenicApi.Sections.Replace("my.mod", LumenicSections.QmHomeNotifications, ui =>
{
    ui.Card(c => c.Text("Custom Notifications Panel", size: 12f, weight: "bold"));
});

// Inject a Quick Action tile matching Lumenic's built-in 3x3 Quick Actions style:
LumenicApi.Sections.Inject("my.mod", LumenicSections.QmHomeActions, ExtSlotPosition.InsideEnd, ui =>
{
    ui.CustomButton("qa_tts", c =>
    {
        c.Col(col =>
        {
            col.Icon("record_voice_over", size: 19f, fg: ttsEnabled ? "accent" : "primary");
            col.Text("TTS", size: 12f, weight: "semibold", align: "center");
            col.Text(ttsEnabled ? "Active" : "Off", size: 9.5f, weight: "medium",
                fg: ttsEnabled ? "accent" : "muted", align: "center");
        }, new ExtStyle { Center = true, Gap = 2.5f, Grow = 1f });
    }, onClick: ToggleTts, active: ttsEnabled, style: new ExtStyle { Grow = 1f, MinH = 52f, P = 5f, Radius = 7.8f, Bevel = 1.3f });
});
```

---

## 5. Sub-Pages, Modals, Toasts & Live Textures

- **Sub-pages**: Call `tabHandle.PushSubPage(new ExtSubPageOptions { Id = "details", Title = "Track Details" }, BuildDetails)` and `tabHandle.PopSubPage()`. A Back button is rendered automatically.
- **Custom Modals**:
  ```csharp
  LumenicApi.OpenModal(ExtSurface.QuickMenu, "my.mod", new ExtModalOptions
  {
      Id = "confirm_clear",
      Title = "Clear Queue?",
      Width = 420f
  }, ui =>
  {
      ui.Text("Remove all queued tracks?", fg: "muted", wrap: true);
      ui.Row(r =>
      {
          r.Button("cancel", "Cancel", () => LumenicApi.CloseModal(ExtSurface.QuickMenu));
          r.Button("ok", "Clear", () => { ClearQueue(); LumenicApi.CloseModal(ExtSurface.QuickMenu); }, variant: "danger");
      }, new ExtStyle { Gap = 8f, Justify = "end" });
  });
  ```
- **Toasts** (thread-safe): `LumenicApi.ShowToast("Added to queue", 2.5f);`
- **Virtual Keyboard Prompt** (thread-safe, great for API keys / secrets so they are never shown inline):
  ```csharp
  LumenicApi.OpenKeyboard(
      initialText: currentApiKey,
      onSubmit: newKey => { SaveApiKey(newKey); _tab.MarkDirty(); },
      title: "Fish Audio API Key",
      placeholder: "Paste Bearer token...",
      maxLen: 256,
      masked: true,
      surface: ExtSurface.QuickMenu);
  ```
- **Live Textures**:
  - `LumenicApi.RegisterImageBytes("ext:my_icon", pngBytes)` (thread-safe)
  - `LumenicApi.RegisterImageUrl("ext:cover", "https://...")` (thread-safe)
  - `LumenicApi.RegisterRawRgba("ext:waveform", width, height, rgbaBytes)` (thread-safe)
  - `LumenicApi.RegisterTexture2D("ext:live_thumb", myTexture2D)` (**Unity main thread only**; keys prefixed with `ext:` are pinned in the texture cache so browsing worlds/avatars never evicts them).

---

## 6. Thread Safety & Unload Lifecycle

| Call | Thread Safety | Notes |
| :--- | :--- | :--- |
| `handle.MarkDirty()` | **Thread-safe** | Safe from `async Task` / `HttpClient` continuations; rebuilds on next Unity main-thread frame if visible. |
| `LumenicApi.ShowToast(...)` | **Thread-safe** | Marshaled onto Unity's main thread automatically. |
| `LumenicApi.OpenKeyboard(...)` | **Thread-safe** | Marshaled onto Unity's main thread automatically; `onSubmit` runs on main thread. |
| `RegisterImageBytes` / `RegisterImageUrl` / `RegisterRawRgba` | **Thread-safe** | Queued and uploaded on Unity's main thread. |
| `RegisterTexture2D(...)` | **Main thread only** | Reads Unity `Texture2D.GetPixels32()`. |
| `UiBuilder` callbacks (`build`, `onClick`, `onChange`, `onSubmit`) | **Main thread** | Always invoked by LumenicMenu on Unity's main thread with exception isolation. |

**Cleanup on unload / quit:** Call `LumenicApi.UnregisterMod(ModId)` (or `.Dispose()` on individual handles) in `OnDeinitializeMelon()` / `OnApplicationQuit()` to remove all tabs, hub entries, slots, modals, and theme overrides cleanly.

---

## 7. Complete `UiBuilder`, `ExtStyle` & `ExtDropdownStyle` Reference

### Color Tokens (`Bg`, `HoverBg`, `BorderColor`, `Fg`, and `ExtDropdownStyle` colors)
- Surfaces: `"transparent"`, `"card"`, `"card_inner"`, `"tile"`, `"tile_hover"`, `"border"`
- Accent: `"accent"`, `"accent_soft"`, `"accent_hover"`, `"accent_border"`
- Text: `"primary"`, `"muted"`, `"dim"`
- Status: `"emerald"`, `"amber"`, `"red"`
- Custom hex: `"#RRGGBB"` or `"#RRGGBBAA"`

### `UiBuilder` Exact Method Signatures
- **Containers**:
  - `Col(Action<UiBuilder> content, ExtStyle style = null, string tooltip = null)`
  - `Row(Action<UiBuilder> content, ExtStyle style = null, string tooltip = null)`
  - `Card(Action<UiBuilder> content, ExtStyle style = null, string tooltip = null)`
  - `ZStack(Action<UiBuilder> content, ExtStyle style = null)`
  - `Grid(int columns, Action<UiBuilder> content, ExtStyle style = null)`
  - `Scroll(string id, Action<UiBuilder> content, ExtStyle style = null, bool hideScrollbar = true)`
  - `HScroll(string id, Action<UiBuilder> content, ExtStyle style = null, bool hideScrollbar = true)`
  - `Spacer(ExtStyle style = null)`
  - `Divider(string color = null, ExtStyle style = null)`
- **Display**:
  - `Text(string text, float? size = null, string weight = null, string fg = null, string align = null, bool wrap = false, bool ellipsis = false, string font = null, string tooltip = null, ExtStyle style = null)`
    - `weight`: `"normal"` | `"medium"` | `"semibold"` | `"bold"`
    - `align`: `"left"` | `"center"` | `"right"`
  - `Icon(string name, float? size = null, string fg = null, string tooltip = null, ExtStyle style = null)` (any Material Symbols Sharp ligature)
  - `Image(string imageKey, ExtImageFit fit = ExtImageFit.Cover, string fallbackIcon = null, string tooltip = null, ExtStyle style = null)`
  - `Badge(string text, string icon = null, string fg = null, float? size = null, ExtStyle style = null)`
  - `ProgressBar(float value, float min = 0f, float max = 1f, string fg = null, ExtStyle style = null)`
- **Interactive** (all support `bool disabled = false`):
  - `Button(string id, string label, Action onClick, string icon = null, string variant = null, bool active = false, bool disabled = false, float? size = null, string fg = null, string tooltip = null, ExtStyle style = null)`
    - `variant`: `null`/`"default"` | `"accent"` | `"danger"` | `"emerald"` | `"ghost"`
  - `IconButton(string id, string icon, Action onClick, string variant = null, bool active = false, bool disabled = false, float? iconSize = null, string fg = null, string tooltip = null, ExtStyle style = null)`
  - `CustomButton(string id, Action<UiBuilder> content, Action onClick, string variant = null, bool active = false, bool disabled = false, string tooltip = null, ExtStyle style = null)`
  - `Toggle(string id, string label, bool value, Action<bool> onChange, string icon = null, bool disabled = false, float? size = null, string tooltip = null, ExtStyle style = null)`
  - `Slider(string id, string label, float value, float min, float max, Action<float> onChange, float step = 0f, string fg = null, bool disabled = false, string tooltip = null, ExtStyle style = null)`
    - Format values directly in `label` (e.g. `$"Speed ({speed:0.00}x)"`); `step > 0` snaps emitted values.
  - `TextInput(string id, string text, string placeholder = null, Action<string> onChange = null, Action<string> onSubmit = null, int maxLen = 256, bool password = false, bool disabled = false, string tooltip = null, ExtStyle style = null)`
    - `onChange` fires per edit on desktop; `onSubmit` fires on Enter (desktop) or VR virtual keyboard Confirm.
    - `password: true` masks characters with `*` and disables copy/cut on desktop, and passes `hidden: true` to VR keyboard.
  - `Tabs(string id, IReadOnlyList<string> options, int selectedIndex, Action<int> onChange, bool disabled = false, float? size = null, ExtStyle style = null)`
  - `Dropdown(string id, IReadOnlyList<string> options, int selectedIndex, Action<int> onChange, bool searchable = false, bool disabled = false, string tooltip = null, ExtStyle style = null, ExtDropdownStyle popupStyle = null)`
    - Out-of-range `selectedIndex` (`< 0` or `>= options.Count`) is clamped safely to `0..options.Count - 1`.

### `ExtStyle` Fields
| Property | Type | Notes |
| :--- | :--- | :--- |
| `W`, `H`, `MinW`, `MaxW`, `MinH`, `MaxH` | `float?` | Logical pixels |
| `WFrac`, `HFrac` | `float?` | Fraction of parent (`0.0` - `1.0`) |
| `Grow` | `float?` | Flex grow weight |
| `P`, `Px`, `Py`, `Pt`, `Pb`, `Pl`, `Pr`, `Gap` | `float?` | Padding & child spacing (scaled by `ThemeTokens`) |
| `Align` | `string` | `"start"` \| `"center"` \| `"end"` \| `"stretch"` |
| `Justify` | `string` | `"start"` \| `"center"` \| `"end"` \| `"between"` \| `"around"` |
| `Center` | `bool` | Shorthand for `Align = "center"` + `Justify = "center"` |
| `Bg`, `HoverBg`, `BorderColor` | `string` | Color token or `#RRGGBB` / `#RRGGBBAA` |
| `BorderW`, `Radius`, `Bevel`, `Shadow`, `Opacity` | `float?` | Border width, corner radius, 45-deg bevel fillet, shadow blur, alpha (`0..1`) |
| `Hexagon`, `Clip`, `Passthrough` | `bool` | Pointy hexagon mask, clip children, ignore pointer events |

### Styled Dropdown / Searchable Combobox Example (`ExtDropdownStyle`)

`ExtDropdownStyle` fields: `Bg`, `BorderColor`, `BorderW`, `Radius`, `Bevel`, `MaxH`, `MinW`, `Padding`, `Gap`, `FontSize`, `ItemH`, `ItemBg`, `ItemHoverBg`, `ItemFg`, `SelectedBg`, `SelectedHoverBg`, `SelectedFg`, `SelectedBorder`, `Shadow`.

```csharp
ui.Dropdown(
    id: "speed",
    options: new[] { "0.5x", "0.75x", "1.0x", "1.25x", "1.5x", "2.0x" },
    selectedIndex: _speedIndex,
    onChange: idx => { _speedIndex = idx; _tab.MarkDirty(); },
    searchable: false, // set true for live-filtered combobox
    style: new ExtStyle { W = 130f },
    popupStyle: new ExtDropdownStyle
    {
        Bg = "#12161EF8",
        BorderColor = "accent",
        Bevel = 1.5f,
        Radius = 8f,
        MaxH = 260f,
        ItemH = 28f,
        ItemHoverBg = "tile_hover",
        SelectedBg = "accent_soft",
        SelectedBorder = "accent",
        FontSize = 11f
    });
```
