using System;
using System.Collections.Generic;
using Newtonsoft.Json;

namespace LumenicMenu.Api;

public enum ExtSurface
{
    QuickMenu,
    MainMenu,
    Both
}

public enum ExtImageFit
{
    Cover,
    Contain,
    Fill
}

public enum ExtSlotPosition
{
    Before,
    After,
    InsideStart,
    InsideEnd,
    Replace
}

/// <summary>
/// Full layout and styling properties for any declarative Lumenic extension node.
/// Color strings accept theme tokens ("card", "card_inner", "tile", "tile_hover", "border",
/// "accent", "accent_soft", "accent_hover", "accent_border", "primary", "muted", "dim",
/// "emerald", "amber", "red", "transparent") or hex ("#RRGGBB" / "#RRGGBBAA").
/// </summary>
public sealed class ExtStyle
{
    [JsonProperty("w", NullValueHandling = NullValueHandling.Ignore)] public float? W { get; set; }
    [JsonProperty("h", NullValueHandling = NullValueHandling.Ignore)] public float? H { get; set; }
    [JsonProperty("w_frac", NullValueHandling = NullValueHandling.Ignore)] public float? WFrac { get; set; }
    [JsonProperty("h_frac", NullValueHandling = NullValueHandling.Ignore)] public float? HFrac { get; set; }
    [JsonProperty("min_w", NullValueHandling = NullValueHandling.Ignore)] public float? MinW { get; set; }
    [JsonProperty("max_w", NullValueHandling = NullValueHandling.Ignore)] public float? MaxW { get; set; }
    [JsonProperty("min_h", NullValueHandling = NullValueHandling.Ignore)] public float? MinH { get; set; }
    [JsonProperty("max_h", NullValueHandling = NullValueHandling.Ignore)] public float? MaxH { get; set; }
    [JsonProperty("grow", NullValueHandling = NullValueHandling.Ignore)] public float? Grow { get; set; }

    [JsonProperty("p", NullValueHandling = NullValueHandling.Ignore)] public float? P { get; set; }
    [JsonProperty("px", NullValueHandling = NullValueHandling.Ignore)] public float? Px { get; set; }
    [JsonProperty("py", NullValueHandling = NullValueHandling.Ignore)] public float? Py { get; set; }
    [JsonProperty("pt", NullValueHandling = NullValueHandling.Ignore)] public float? Pt { get; set; }
    [JsonProperty("pb", NullValueHandling = NullValueHandling.Ignore)] public float? Pb { get; set; }
    [JsonProperty("pl", NullValueHandling = NullValueHandling.Ignore)] public float? Pl { get; set; }
    [JsonProperty("pr", NullValueHandling = NullValueHandling.Ignore)] public float? Pr { get; set; }
    [JsonProperty("gap", NullValueHandling = NullValueHandling.Ignore)] public float? Gap { get; set; }

    /// <summary>"start" | "center" | "end" | "stretch"</summary>
    [JsonProperty("align", NullValueHandling = NullValueHandling.Ignore)] public string Align { get; set; }
    /// <summary>"start" | "center" | "end" | "between" | "around"</summary>
    [JsonProperty("justify", NullValueHandling = NullValueHandling.Ignore)] public string Justify { get; set; }
    [JsonProperty("center", DefaultValueHandling = DefaultValueHandling.Ignore)] public bool Center { get; set; }

    [JsonProperty("bg", NullValueHandling = NullValueHandling.Ignore)] public string Bg { get; set; }
    [JsonProperty("hover_bg", NullValueHandling = NullValueHandling.Ignore)] public string HoverBg { get; set; }
    [JsonProperty("border_color", NullValueHandling = NullValueHandling.Ignore)] public string BorderColor { get; set; }
    [JsonProperty("border_w", NullValueHandling = NullValueHandling.Ignore)] public float? BorderW { get; set; }
    [JsonProperty("radius", NullValueHandling = NullValueHandling.Ignore)] public float? Radius { get; set; }
    [JsonProperty("bevel", NullValueHandling = NullValueHandling.Ignore)] public float? Bevel { get; set; }
    [JsonProperty("hexagon", DefaultValueHandling = DefaultValueHandling.Ignore)] public bool Hexagon { get; set; }
    [JsonProperty("shadow", NullValueHandling = NullValueHandling.Ignore)] public float? Shadow { get; set; }
    [JsonProperty("opacity", NullValueHandling = NullValueHandling.Ignore)] public float? Opacity { get; set; }
    [JsonProperty("clip", DefaultValueHandling = DefaultValueHandling.Ignore)] public bool Clip { get; set; }
    [JsonProperty("passthrough", DefaultValueHandling = DefaultValueHandling.Ignore)] public bool Passthrough { get; set; }
}

/// <summary>
/// Optional popup menu styling overrides for <see cref="UiBuilder.Dropdown"/>.
/// Defaults match Lumenic's Obsidian Slate + Player Accent beveled theme.
/// </summary>
public sealed class ExtDropdownStyle
{
    [JsonProperty("bg", NullValueHandling = NullValueHandling.Ignore)] public string Bg { get; set; }
    [JsonProperty("border_color", NullValueHandling = NullValueHandling.Ignore)] public string BorderColor { get; set; }
    [JsonProperty("border_w", NullValueHandling = NullValueHandling.Ignore)] public float? BorderW { get; set; }
    [JsonProperty("radius", NullValueHandling = NullValueHandling.Ignore)] public float? Radius { get; set; }
    [JsonProperty("bevel", NullValueHandling = NullValueHandling.Ignore)] public float? Bevel { get; set; }
    [JsonProperty("max_h", NullValueHandling = NullValueHandling.Ignore)] public float? MaxH { get; set; }
    [JsonProperty("min_w", NullValueHandling = NullValueHandling.Ignore)] public float? MinW { get; set; }
    [JsonProperty("padding", NullValueHandling = NullValueHandling.Ignore)] public float? Padding { get; set; }
    [JsonProperty("gap", NullValueHandling = NullValueHandling.Ignore)] public float? Gap { get; set; }
    [JsonProperty("font_size", NullValueHandling = NullValueHandling.Ignore)] public float? FontSize { get; set; }
    [JsonProperty("item_h", NullValueHandling = NullValueHandling.Ignore)] public float? ItemH { get; set; }
    [JsonProperty("item_bg", NullValueHandling = NullValueHandling.Ignore)] public string ItemBg { get; set; }
    [JsonProperty("item_hover_bg", NullValueHandling = NullValueHandling.Ignore)] public string ItemHoverBg { get; set; }
    [JsonProperty("item_fg", NullValueHandling = NullValueHandling.Ignore)] public string ItemFg { get; set; }
    [JsonProperty("selected_bg", NullValueHandling = NullValueHandling.Ignore)] public string SelectedBg { get; set; }
    [JsonProperty("selected_hover_bg", NullValueHandling = NullValueHandling.Ignore)] public string SelectedHoverBg { get; set; }
    [JsonProperty("selected_fg", NullValueHandling = NullValueHandling.Ignore)] public string SelectedFg { get; set; }
    [JsonProperty("selected_border", NullValueHandling = NullValueHandling.Ignore)] public string SelectedBorder { get; set; }
    [JsonProperty("shadow", NullValueHandling = NullValueHandling.Ignore)] public float? Shadow { get; set; }
}

internal sealed class ExtNodeDto
{
    [JsonProperty("kind")] public string Kind { get; set; } = string.Empty;
    [JsonProperty("id", NullValueHandling = NullValueHandling.Ignore)] public string Id { get; set; }
    [JsonProperty("style", NullValueHandling = NullValueHandling.Ignore)] public ExtStyle Style { get; set; }

    [JsonProperty("text", NullValueHandling = NullValueHandling.Ignore)] public string Text { get; set; }
    [JsonProperty("placeholder", NullValueHandling = NullValueHandling.Ignore)] public string Placeholder { get; set; }
    [JsonProperty("icon", NullValueHandling = NullValueHandling.Ignore)] public string Icon { get; set; }
    [JsonProperty("image_key", NullValueHandling = NullValueHandling.Ignore)] public string ImageKey { get; set; }
    [JsonProperty("fit", NullValueHandling = NullValueHandling.Ignore)] public string Fit { get; set; }
    [JsonProperty("font", NullValueHandling = NullValueHandling.Ignore)] public string Font { get; set; }
    [JsonProperty("size", NullValueHandling = NullValueHandling.Ignore)] public float? Size { get; set; }
    [JsonProperty("weight", NullValueHandling = NullValueHandling.Ignore)] public string Weight { get; set; }
    [JsonProperty("text_align", NullValueHandling = NullValueHandling.Ignore)] public string TextAlign { get; set; }
    [JsonProperty("fg", NullValueHandling = NullValueHandling.Ignore)] public string Fg { get; set; }
    [JsonProperty("wrap", DefaultValueHandling = DefaultValueHandling.Ignore)] public bool Wrap { get; set; }
    [JsonProperty("ellipsis", DefaultValueHandling = DefaultValueHandling.Ignore)] public bool Ellipsis { get; set; }

    [JsonProperty("active", DefaultValueHandling = DefaultValueHandling.Ignore)] public bool Active { get; set; }
    [JsonProperty("disabled", DefaultValueHandling = DefaultValueHandling.Ignore)] public bool Disabled { get; set; }
    [JsonProperty("variant", NullValueHandling = NullValueHandling.Ignore)] public string Variant { get; set; }
    [JsonProperty("tooltip", NullValueHandling = NullValueHandling.Ignore)] public string Tooltip { get; set; }
    [JsonProperty("value", DefaultValueHandling = DefaultValueHandling.Ignore)] public float Value { get; set; }
    [JsonProperty("min", DefaultValueHandling = DefaultValueHandling.Ignore)] public float Min { get; set; }
    [JsonProperty("max", DefaultValueHandling = DefaultValueHandling.Ignore)] public float Max { get; set; }
    [JsonProperty("step", DefaultValueHandling = DefaultValueHandling.Ignore)] public float Step { get; set; }
    [JsonProperty("columns", DefaultValueHandling = DefaultValueHandling.Ignore)] public int Columns { get; set; }
    [JsonProperty("options", NullValueHandling = NullValueHandling.Ignore)] public List<string> Options { get; set; }
    [JsonProperty("selected_index", DefaultValueHandling = DefaultValueHandling.Ignore)] public int SelectedIndex { get; set; }
    [JsonProperty("hide_scrollbar", DefaultValueHandling = DefaultValueHandling.Ignore)] public bool HideScrollbar { get; set; }
    [JsonProperty("max_len", DefaultValueHandling = DefaultValueHandling.Ignore)] public int MaxLen { get; set; }
    [JsonProperty("password", DefaultValueHandling = DefaultValueHandling.Ignore)] public bool Password { get; set; }
    [JsonProperty("searchable", DefaultValueHandling = DefaultValueHandling.Ignore)] public bool Searchable { get; set; }
    [JsonProperty("dropdown_style", NullValueHandling = NullValueHandling.Ignore)] public ExtDropdownStyle DropdownStyle { get; set; }

    [JsonProperty("children", NullValueHandling = NullValueHandling.Ignore)] public List<ExtNodeDto> Children { get; set; }
}

internal sealed class ExtCallbackTable
{
    public readonly Dictionary<string, Action> Clicks = new Dictionary<string, Action>(StringComparer.Ordinal);
    public readonly Dictionary<string, Action<bool>> BoolChanges = new Dictionary<string, Action<bool>>(StringComparer.Ordinal);
    public readonly Dictionary<string, Action<float>> FloatChanges = new Dictionary<string, Action<float>>(StringComparer.Ordinal);
    public readonly Dictionary<string, Action<int>> IntChanges = new Dictionary<string, Action<int>>(StringComparer.Ordinal);
    public readonly Dictionary<string, Action<string>> StringChanges = new Dictionary<string, Action<string>>(StringComparer.Ordinal);
    public readonly Dictionary<string, Action<string>> StringSubmits = new Dictionary<string, Action<string>>(StringComparer.Ordinal);
}

/// <summary>
/// Declarative fluent UI builder passed to extension page, modal, and slot render callbacks.
/// </summary>
public sealed class UiBuilder
{
    private readonly List<ExtNodeDto> _nodes;
    private readonly ExtCallbackTable _callbacks;

    internal UiBuilder(List<ExtNodeDto> nodes, ExtCallbackTable callbacks)
    {
        _nodes = nodes;
        _callbacks = callbacks;
    }

    private void AddContainer(string kind, string id, Action<UiBuilder> content, ExtStyle style, Action<ExtNodeDto> configure = null)
    {
        var children = new List<ExtNodeDto>();
        content?.Invoke(new UiBuilder(children, _callbacks));
        var node = new ExtNodeDto
        {
            Kind = kind,
            Id = id,
            Style = style,
            Children = children.Count > 0 ? children : null
        };
        configure?.Invoke(node);
        _nodes.Add(node);
    }

    public void Col(Action<UiBuilder> content, ExtStyle style = null, string tooltip = null)
        => AddContainer("col", null, content, style, n => n.Tooltip = tooltip);

    public void Row(Action<UiBuilder> content, ExtStyle style = null, string tooltip = null)
        => AddContainer("row", null, content, style, n => n.Tooltip = tooltip);

    public void Card(Action<UiBuilder> content, ExtStyle style = null, string tooltip = null)
        => AddContainer("card", null, content, style, n => n.Tooltip = tooltip);

    public void ZStack(Action<UiBuilder> content, ExtStyle style = null)
        => AddContainer("zstack", null, content, style);

    public void Grid(int columns, Action<UiBuilder> content, ExtStyle style = null)
        => AddContainer("grid", null, content, style, n => n.Columns = Math.Max(1, columns));

    public void Scroll(string id, Action<UiBuilder> content, ExtStyle style = null, bool hideScrollbar = true)
        => AddContainer("scroll", id, content, style, n => n.HideScrollbar = hideScrollbar);

    public void HScroll(string id, Action<UiBuilder> content, ExtStyle style = null, bool hideScrollbar = true)
        => AddContainer("hscroll", id, content, style, n => n.HideScrollbar = hideScrollbar);

    public void Spacer(ExtStyle style = null)
    {
        _nodes.Add(new ExtNodeDto { Kind = "spacer", Style = style });
    }

    public void Divider(string color = null, ExtStyle style = null)
    {
        _nodes.Add(new ExtNodeDto { Kind = "divider", Fg = color, Style = style });
    }

    public void Text(
        string text,
        float? size = null,
        string weight = null,
        string fg = null,
        string align = null,
        bool wrap = false,
        bool ellipsis = false,
        string font = null,
        string tooltip = null,
        ExtStyle style = null)
    {
        _nodes.Add(new ExtNodeDto
        {
            Kind = "text",
            Text = text ?? string.Empty,
            Size = size,
            Weight = weight,
            Fg = fg,
            TextAlign = align,
            Wrap = wrap,
            Ellipsis = ellipsis,
            Font = font,
            Tooltip = tooltip,
            Style = style
        });
    }

    public void Icon(
        string name,
        float? size = null,
        string fg = null,
        string tooltip = null,
        ExtStyle style = null)
    {
        _nodes.Add(new ExtNodeDto
        {
            Kind = "icon",
            Icon = name ?? string.Empty,
            Size = size,
            Fg = fg,
            Tooltip = tooltip,
            Style = style
        });
    }

    public void Image(
        string imageKey,
        ExtImageFit fit = ExtImageFit.Cover,
        string fallbackIcon = null,
        string tooltip = null,
        ExtStyle style = null)
    {
        _nodes.Add(new ExtNodeDto
        {
            Kind = "image",
            ImageKey = imageKey ?? string.Empty,
            Fit = fit switch
            {
                ExtImageFit.Contain => "contain",
                ExtImageFit.Fill => "fill",
                _ => "cover"
            },
            Icon = fallbackIcon,
            Tooltip = tooltip,
            Style = style
        });
    }

    public void Badge(
        string text,
        string icon = null,
        string fg = null,
        float? size = null,
        ExtStyle style = null)
    {
        _nodes.Add(new ExtNodeDto
        {
            Kind = "badge",
            Text = text ?? string.Empty,
            Icon = icon,
            Fg = fg,
            Size = size,
            Style = style
        });
    }

    public void ProgressBar(
        float value,
        float min = 0f,
        float max = 1f,
        string fg = null,
        ExtStyle style = null)
    {
        _nodes.Add(new ExtNodeDto
        {
            Kind = "progress",
            Value = value,
            Min = min,
            Max = max,
            Fg = fg,
            Style = style
        });
    }

    public void Button(
        string id,
        string label,
        Action onClick,
        string icon = null,
        string variant = null,
        bool active = false,
        bool disabled = false,
        float? size = null,
        string fg = null,
        string tooltip = null,
        ExtStyle style = null)
    {
        if (!string.IsNullOrEmpty(id) && onClick != null)
            _callbacks.Clicks[id] = onClick;

        _nodes.Add(new ExtNodeDto
        {
            Kind = "button",
            Id = id,
            Text = label ?? string.Empty,
            Icon = icon,
            Variant = variant,
            Active = active,
            Disabled = disabled,
            Size = size,
            Fg = fg,
            Tooltip = tooltip,
            Style = style
        });
    }

    public void IconButton(
        string id,
        string icon,
        Action onClick,
        string variant = null,
        bool active = false,
        bool disabled = false,
        float? iconSize = null,
        string fg = null,
        string tooltip = null,
        ExtStyle style = null)
    {
        Button(id, string.Empty, onClick, icon, variant, active, disabled, iconSize, fg, tooltip, style);
    }

    public void CustomButton(
        string id,
        Action<UiBuilder> content,
        Action onClick,
        string variant = null,
        bool active = false,
        bool disabled = false,
        string tooltip = null,
        ExtStyle style = null)
    {
        if (!string.IsNullOrEmpty(id) && onClick != null)
            _callbacks.Clicks[id] = onClick;

        AddContainer("button", id, content, style, n =>
        {
            n.Variant = variant;
            n.Active = active;
            n.Disabled = disabled;
            n.Tooltip = tooltip;
        });
    }

    public void Toggle(
        string id,
        string label,
        bool value,
        Action<bool> onChange,
        string icon = null,
        bool disabled = false,
        float? size = null,
        string tooltip = null,
        ExtStyle style = null)
    {
        if (!string.IsNullOrEmpty(id) && onChange != null)
            _callbacks.BoolChanges[id] = onChange;

        _nodes.Add(new ExtNodeDto
        {
            Kind = "toggle",
            Id = id,
            Text = label ?? string.Empty,
            Active = value,
            Icon = icon,
            Disabled = disabled,
            Size = size,
            Tooltip = tooltip,
            Style = style
        });
    }

    public void Slider(
        string id,
        string label,
        float value,
        float min,
        float max,
        Action<float> onChange,
        float step = 0f,
        string fg = null,
        bool disabled = false,
        string tooltip = null,
        ExtStyle style = null)
    {
        if (!string.IsNullOrEmpty(id) && onChange != null)
            _callbacks.FloatChanges[id] = onChange;

        _nodes.Add(new ExtNodeDto
        {
            Kind = "slider",
            Id = id,
            Text = label ?? string.Empty,
            Value = value,
            Min = min,
            Max = max,
            Step = step,
            Fg = fg,
            Disabled = disabled,
            Tooltip = tooltip,
            Style = style
        });
    }

    public void TextInput(
        string id,
        string text,
        string placeholder = null,
        Action<string> onChange = null,
        Action<string> onSubmit = null,
        int maxLen = 256,
        bool password = false,
        bool disabled = false,
        string tooltip = null,
        ExtStyle style = null)
    {
        if (!string.IsNullOrEmpty(id))
        {
            if (onChange != null) _callbacks.StringChanges[id] = onChange;
            if (onSubmit != null) _callbacks.StringSubmits[id] = onSubmit;
        }

        _nodes.Add(new ExtNodeDto
        {
            Kind = "input",
            Id = id,
            Text = text ?? string.Empty,
            Placeholder = placeholder ?? string.Empty,
            MaxLen = maxLen,
            Password = password,
            Disabled = disabled,
            Tooltip = tooltip,
            Style = style
        });
    }

    public void Dropdown(
        string id,
        IReadOnlyList<string> options,
        int selectedIndex,
        Action<int> onChange,
        bool searchable = false,
        bool disabled = false,
        string tooltip = null,
        ExtStyle style = null,
        ExtDropdownStyle popupStyle = null)
    {
        if (!string.IsNullOrEmpty(id) && onChange != null)
            _callbacks.IntChanges[id] = onChange;

        var list = options != null ? new List<string>(options) : new List<string>();
        _nodes.Add(new ExtNodeDto
        {
            Kind = "dropdown",
            Id = id,
            Options = list,
            SelectedIndex = Math.Max(0, selectedIndex),
            Searchable = searchable,
            Disabled = disabled,
            Tooltip = tooltip,
            Style = style,
            DropdownStyle = popupStyle
        });
    }

    public void Tabs(
        string id,
        IReadOnlyList<string> options,
        int selectedIndex,
        Action<int> onChange,
        bool disabled = false,
        float? size = null,
        ExtStyle style = null)
    {
        if (!string.IsNullOrEmpty(id) && onChange != null)
            _callbacks.IntChanges[id] = onChange;

        var list = options != null ? new List<string>(options) : new List<string>();
        _nodes.Add(new ExtNodeDto
        {
            Kind = "tabs",
            Id = id,
            Options = list,
            SelectedIndex = Math.Max(0, selectedIndex),
            Disabled = disabled,
            Size = size,
            Style = style
        });
    }
}
