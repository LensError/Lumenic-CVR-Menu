using System;
using LumenicMenu.Api;

namespace LumenicUiShowcase;

internal sealed partial class ShowcaseUi
{
    // -------------------------------------------------------------------------
    // 1. Controls: Buttons, Toggles, Sliders, Inputs, Dropdowns
    // -------------------------------------------------------------------------
    private void BuildControlsSection(UiBuilder ui, bool compact)
    {
        ui.Card(c =>
        {
            c.Row(r =>
            {
                r.Text("Buttons & Variants", size: 12.5f, weight: "bold", style: new ExtStyle { Grow = 1f });
                r.Badge($"Clicks: {_clickCount}", icon: "touch_app", fg: "accent");
            }, new ExtStyle { Align = "center" });

            c.Grid(compact ? 3 : 5, g =>
            {
                g.Button("btn_def", "Default", () => OnDemoClick("Default"), icon: "smart_button", tooltip: "Standard tile button");
                g.Button("btn_acc", "Accent", () => { _accentBtnActive = !_accentBtnActive; OnDemoClick("Accent"); },
                    icon: "bolt", variant: "accent", active: _accentBtnActive);
                g.Button("btn_em", "Emerald", () => OnDemoClick("Emerald"), icon: "check_circle", variant: "emerald");
                g.Button("btn_dan", "Danger", () => OnDemoClick("Danger"), icon: "warning", variant: "danger");
                g.Button("btn_gho", "Ghost", () => OnDemoClick("Ghost"), icon: "visibility", variant: "ghost");
            }, new ExtStyle { Gap = 6f });

            c.Row(r =>
            {
                r.Button("btn_dis", "Disabled", () => { }, icon: "block", disabled: true);
                r.IconButton("ib_1", "favorite", () => OnDemoClick("IconButton"), variant: "accent", tooltip: "IconButton (accent)");
                r.IconButton("ib_2", "delete", () => OnDemoClick("IconButton Danger"), variant: "danger", tooltip: "IconButton (danger)");
                r.IconButton("ib_3", "lock", () => { }, disabled: true, tooltip: "Disabled IconButton");

                // CustomButton lets you compose multi-line tiles matching QM Quick Actions
                r.CustomButton("cb_tile", tile =>
                {
                    tile.Col(col =>
                    {
                        col.Icon("rocket_launch", size: 18f, fg: _accentBtnActive ? "accent" : "primary");
                        col.Text("CustomButton", size: 11f, weight: "semibold", align: "center");
                        col.Text(_accentBtnActive ? "Active State" : "Idle State", size: 9.5f,
                            fg: _accentBtnActive ? "accent" : "muted", align: "center");
                    }, new ExtStyle { Center = true, Gap = 2f });
                }, () => { _accentBtnActive = !_accentBtnActive; OnDemoClick("CustomButton"); },
                    active: _accentBtnActive,
                    tooltip: "CustomButton with nested Icon + Texts",
                    style: new ExtStyle { Grow = 1f, MinH = 54f, P = 6f, Radius = 8f, Bevel = 1.4f });
            }, new ExtStyle { Gap = 6f, Align = "center" });
        }, new ExtStyle { Gap = 8f });

        ui.Card(c =>
        {
            c.Text("Toggles & Sliders", size: 12.5f, weight: "bold");
            c.Grid(compact ? 1 : 3, g =>
            {
                g.Toggle("tog_a", "Interactive Toggle", _toggleA, v => { _toggleA = v; RefreshAll(); }, icon: "toggle_on");
                g.Toggle("tog_b", "Secondary Option", _toggleB, v => { _toggleB = v; RefreshAll(); }, icon: "tune");
                g.Toggle("tog_dis", "Disabled Toggle", true, _ => { }, icon: "lock", disabled: true);
            }, new ExtStyle { Gap = 6f });

            c.Slider("sl_cont", $"Continuous ({_continuousSlider:0.00})", _continuousSlider, 0f, 1f,
                v => { _continuousSlider = v; _progressValue = v; RefreshAll(); },
                tooltip: "Drives the ProgressBar in the Media tab too");

            c.Slider("sl_step", $"Stepped ({_steppedSlider:0}%)", _steppedSlider, 0f, 100f,
                v => { _steppedSlider = v; RefreshAll(); },
                step: 10f, fg: "emerald");

            c.Slider("sl_dis", "Disabled Slider (40%)", 40f, 0f, 100f, _ => { }, disabled: true);
        }, new ExtStyle { Gap = 7f });

        ui.Card(c =>
        {
            c.Text("Text Inputs & Dropdowns", size: 12.5f, weight: "bold");

            c.Row(r =>
            {
                r.TextInput("in_plain", _plainInput, placeholder: "Type and press Enter...",
                    onChange: s => { _plainInput = s; },
                    onSubmit: s => { _plainInput = s; _lastSubmitted = s; LogEvent($"Submitted: {s}"); RefreshAll(); },
                    style: new ExtStyle { Grow = 1f });

                r.TextInput("in_secret", _secretInput, placeholder: "Password / API key...",
                    onChange: s => { _secretInput = s; },
                    onSubmit: s => { _secretInput = s; LogEvent($"Secret updated ({s.Length} chars)"); RefreshAll(); },
                    password: true,
                    tooltip: "password: true masks characters and disables copy/cut",
                    style: new ExtStyle { Grow = 1f });
            }, new ExtStyle { Gap = 8f });

            c.Text($"Last submitted: {_lastSubmitted}", size: 10.5f, fg: "muted", ellipsis: true);

            c.Row(r =>
            {
                r.Col(col =>
                {
                    col.Text("Standard + Custom Popup Style", size: 10.5f, fg: "muted");
                    col.Dropdown("dd_std", DropdownFruits, _standardDropdownIdx,
                        idx => { _standardDropdownIdx = idx; RefreshAll(); },
                        searchable: false,
                        style: new ExtStyle { WFrac = 1f },
                        popupStyle: new ExtDropdownStyle
                        {
                            Bg = "#12161EF8",
                            BorderColor = "accent",
                            Bevel = 1.5f,
                            Radius = 8f,
                            ItemH = 28f,
                            ItemHoverBg = "tile_hover",
                            SelectedBg = "accent_soft",
                            SelectedBorder = "accent",
                            FontSize = 11f
                        });
                }, new ExtStyle { Grow = 1f, Gap = 4f });

                r.Col(col =>
                {
                    col.Text("Searchable Combobox", size: 10.5f, fg: "muted");
                    col.Dropdown("dd_search", SearchableIcons, _searchableDropdownIdx,
                        idx => { _searchableDropdownIdx = idx; RefreshAll(); },
                        searchable: true,
                        tooltip: "Type to filter Material icon names",
                        style: new ExtStyle { WFrac = 1f });
                }, new ExtStyle { Grow = 1f, Gap = 4f });
            }, new ExtStyle { Gap = 8f });
        }, new ExtStyle { Gap = 7f });
    }

    private void OnDemoClick(string label)
    {
        _clickCount++;
        LogEvent($"Clicked {label} (#{_clickCount})");
        RefreshAll();
    }

    // -------------------------------------------------------------------------
    // 2. Layout & Styling: Grid, ZStack, HScroll, Spacer, Divider, ExtStyle
    // -------------------------------------------------------------------------
    private void BuildLayoutSection(UiBuilder ui, bool compact)
    {
        ui.Card(c =>
        {
            c.Row(r =>
            {
                r.Text("ExtStyle Shapes & Surfaces", size: 12.5f, weight: "bold", style: new ExtStyle { Grow = 1f });
                r.Toggle("tog_sb", "Show Scrollbars", _showScrollbars, v => { _showScrollbars = v; RefreshAll(); });
            }, new ExtStyle { Align = "center" });

            c.Row(r =>
            {
                r.Col(box => box.Text("Radius 12", size: 10.5f, align: "center"),
                    new ExtStyle { Center = true, Grow = 1f, H = 56f, Bg = "card_inner", BorderColor = "border", BorderW = 1f, Radius = 12f });

                r.Col(box => box.Text("Bevel 4", size: 10.5f, align: "center"),
                    new ExtStyle { Center = true, Grow = 1f, H = 56f, Bg = "accent_soft", BorderColor = "accent_border", BorderW = 1.2f, Bevel = 4f, Shadow = 8f });

                r.Col(box => box.Icon("hexagon", size: 22f, fg: "emerald"),
                    new ExtStyle { Center = true, W = 60f, H = 60f, Bg = "#0F2922", BorderColor = "emerald", BorderW = 1.5f, Hexagon = true });

                r.Col(box => box.Text("Opacity 55%", size: 10.5f, align: "center"),
                    new ExtStyle { Center = true, Grow = 1f, H = 56f, Bg = "#3B1D4A", BorderColor = "#C084FC", BorderW = 1f, Radius = 8f, Opacity = 0.55f });
            }, new ExtStyle { Gap = 8f, Align = "center" });

            c.Divider();

            // ZStack layers children in order; Passthrough lets clicks pass through overlays
            c.Text("ZStack Overlay + Spacer/Justify", size: 11f, weight: "semibold", fg: "muted");
            c.Row(r =>
            {
                r.ZStack(z =>
                {
                    z.Col(bg =>
                    {
                        bg.Text("Base Card Layer", size: 11f, fg: "muted");
                    }, new ExtStyle { Center = true, W = 180f, H = 52f, Bg = "card_inner", BorderColor = "border", BorderW = 1f, Radius = 8f });

                    z.Row(overlay =>
                    {
                        overlay.Spacer(new ExtStyle { Grow = 1f });
                        overlay.Badge("OVERLAY", fg: "amber");
                    }, new ExtStyle { W = 180f, P = 5f, Passthrough = true });
                }, new ExtStyle { W = 180f, H = 52f });

                r.Spacer(new ExtStyle { Grow = 1f });
                r.Badge("Right-Aligned via Spacer", icon: "east", fg: "accent");
            }, new ExtStyle { Align = "center" });
        }, new ExtStyle { Gap = 8f });

        ui.Card(c =>
        {
            c.Row(r =>
            {
                r.Text($"Dynamic Grid ({_gridColumns} cols)", size: 12.5f, weight: "bold", style: new ExtStyle { Grow = 1f });
                r.Button("grid_dec", "- Col", () => { _gridColumns = Math.Max(1, _gridColumns - 1); RefreshAll(); });
                r.Button("grid_inc", "+ Col", () => { _gridColumns = Math.Min(compact ? 4 : 6, _gridColumns + 1); RefreshAll(); });
            }, new ExtStyle { Align = "center", Gap = 6f });

            c.Grid(_gridColumns, g =>
            {
                for (int i = 1; i <= 6; i++)
                {
                    g.Col(cell =>
                    {
                        cell.Text($"Cell {i}", size: 11f, weight: "semibold");
                        cell.Text($"Token tile", size: 9.5f, fg: "dim");
                    }, new ExtStyle { P = 8f, Bg = "tile", Radius = 6f, Gap = 2f });
                }
            }, new ExtStyle { Gap = 6f });
        }, new ExtStyle { Gap = 8f });

        ui.Card(c =>
        {
            c.Text("Horizontal Scroll (HScroll)", size: 12.5f, weight: "bold");
            c.HScroll("demo_hscroll", strip =>
            {
                for (int i = 0; i < SearchableIcons.Length; i++)
                {
                    string iconName = SearchableIcons[i];
                    strip.Col(item =>
                    {
                        item.Icon(iconName, size: 22f, fg: "accent");
                        item.Text(iconName, size: 9.5f, fg: "muted", ellipsis: true, align: "center");
                    }, new ExtStyle { Center = true, W = 92f, H = 64f, P = 6f, Bg = "card_inner", Radius = 7f, Gap = 4f });
                }
            }, new ExtStyle { H = 74f, Gap = 6f }, hideScrollbar: !_showScrollbars);
        }, new ExtStyle { Gap = 6f });
    }

    // -------------------------------------------------------------------------
    // 3. Media & Display: Text, Icons, Badges, ProgressBar, Procedural Images
    // -------------------------------------------------------------------------
    private void BuildMediaSection(UiBuilder ui, bool compact)
    {
        ui.Card(c =>
        {
            c.Text("Typography, Badges & ProgressBar", size: 12.5f, weight: "bold");
            c.Row(r =>
            {
                r.Text("Normal", size: 11.5f, weight: "normal");
                r.Text("Medium", size: 11.5f, weight: "medium", fg: "muted");
                r.Text("Semibold", size: 11.5f, weight: "semibold", fg: "accent");
                r.Text("Bold Hex", size: 11.5f, weight: "bold", fg: "#F472B6");
            }, new ExtStyle { Gap = 12f, Align = "center" });

            c.Row(r =>
            {
                r.Badge("Emerald", icon: "check", fg: "emerald");
                r.Badge("Amber", icon: "schedule", fg: "amber");
                r.Badge("Red", icon: "error", fg: "red");
                r.Badge("Custom Hex", icon: "star", fg: "#A78BFA");
            }, new ExtStyle { Gap = 6f });

            c.ProgressBar(_progressValue, 0f, 1f, fg: "accent", style: new ExtStyle { H = 7f });
            c.Slider("sl_prog", $"Progress ({_progressValue * 100f:0}%)", _progressValue, 0f, 1f,
                v => { _progressValue = v; _continuousSlider = v; RefreshAll(); });
        }, new ExtStyle { Gap = 7f });

        ui.Card(c =>
        {
            c.Row(r =>
            {
                r.Text("Procedural Textures & Image Fit Modes", size: 12.5f, weight: "bold", style: new ExtStyle { Grow = 1f });
                r.Button("tex_regen", "Regenerate Textures", () =>
                {
                    _textureSeed++;
                    ShowcaseTextures.RegisterAll(_textureSeed);
                    LogEvent($"Regenerated textures (seed={_textureSeed})");
                    RefreshAll();
                }, icon: "refresh", variant: "accent");
            }, new ExtStyle { Align = "center" });

            c.Grid(compact ? 1 : 3, g =>
            {
                g.Col(col =>
                {
                    col.Image(ShowcaseTextures.RawRgbaKey, ExtImageFit.Cover, fallbackIcon: "image",
                        style: new ExtStyle { WFrac = 1f, H = 88f, Radius = 8f });
                    col.Text("RegisterRawRgba (Cover)", size: 10.5f, fg: "muted", align: "center");
                }, new ExtStyle { Gap = 4f });

                g.Col(col =>
                {
                    col.Image(ShowcaseTextures.UnityTexKey, ExtImageFit.Contain, fallbackIcon: "grid_on",
                        style: new ExtStyle { WFrac = 1f, H = 88f, Radius = 8f, Bg = "card_inner" });
                    col.Text("RegisterTexture2D (Contain)", size: 10.5f, fg: "muted", align: "center");
                }, new ExtStyle { Gap = 4f });

                g.Col(col =>
                {
                    col.Image(ShowcaseTextures.FallbackDemoKey, ExtImageFit.Fill, fallbackIcon: "broken_image",
                        style: new ExtStyle { WFrac = 1f, H = 88f, Radius = 8f, Bg = "card_inner" });
                    col.Text("Missing Key -> fallbackIcon", size: 10.5f, fg: "muted", align: "center");
                }, new ExtStyle { Gap = 4f });
            }, new ExtStyle { Gap = 8f });
        }, new ExtStyle { Gap = 8f });
    }

    // -------------------------------------------------------------------------
    // 4. Overlays & Events: Modals, Toasts, Virtual Keyboard, Lifecycle Log
    // -------------------------------------------------------------------------
    private void BuildOverlaysSection(UiBuilder ui, ExtSurface surface, ExtTabHandle tab, bool compact)
    {
        ui.Card(c =>
        {
            c.Text("Modals, Keyboard & Toasts", size: 12.5f, weight: "bold");
            c.Toggle("mod_backdrop", "Modal CloseOnBackdrop", _modalCloseOnBackdrop,
                v => { _modalCloseOnBackdrop = v; RefreshAll(); });

            c.Grid(compact ? 2 : 4, g =>
            {
                g.Button("ov_modal", "Open Modal", () => OpenDemoModal(surface), icon: "open_in_new", variant: "accent");
                g.Button("ov_kb", "Open Keyboard", () =>
                {
                    LumenicApi.OpenKeyboard(_plainInput, s =>
                    {
                        _plainInput = s;
                        _lastSubmitted = s;
                        LogEvent($"Keyboard submitted: {s}");
                        RefreshAll();
                    }, title: "Demo Text Prompt", placeholder: "Enter text...", surface: surface);
                }, icon: "keyboard");

                g.Button("ov_kb_mask", "Secret Keyboard", () =>
                {
                    LumenicApi.OpenKeyboard(_secretInput, s =>
                    {
                        _secretInput = s;
                        LogEvent($"Masked keyboard submitted ({s.Length} chars)");
                        RefreshAll();
                    }, title: "Secret Token", placeholder: "Hidden input...", masked: true, surface: surface);
                }, icon: "password");

                g.Button("ov_sub", "Push Sub-Page", () => PushDemoSubPage(tab, 1), icon: "layers");
            }, new ExtStyle { Gap = 6f });
        }, new ExtStyle { Gap = 7f });

        ui.Card(c =>
        {
            c.Row(r =>
            {
                r.Text("Live Lifecycle & Interaction Log", size: 12.5f, weight: "bold", style: new ExtStyle { Grow = 1f });
                r.Button("log_clear", "Clear", () => { _eventLog.Clear(); RefreshAll(); }, icon: "delete_sweep", variant: "ghost");
            }, new ExtStyle { Align = "center" });

            if (_eventLog.Count == 0)
            {
                c.Text("No events recorded yet.", size: 10.5f, fg: "dim");
            }
            else
            {
                c.Scroll("log_scroll", list =>
                {
                    for (int i = 0; i < _eventLog.Count; i++)
                    {
                        list.Text(_eventLog[i], size: 10.5f, fg: i == 0 ? "primary" : "muted", ellipsis: true);
                    }
                }, new ExtStyle { H = 130f, Gap = 3f }, hideScrollbar: false);
            }
        }, new ExtStyle { Gap = 6f });
    }

    private void OpenDemoModal(ExtSurface surface)
    {
        LumenicApi.OpenModal(surface, ModId, new ExtModalOptions
        {
            Id = "demo_modal",
            Title = $"Custom {surface} Modal",
            Width = surface == ExtSurface.QuickMenu ? 420f : 480f,
            CloseOnBackdrop = _modalCloseOnBackdrop
        }, m =>
        {
            m.Text("Modals accept any UiBuilder tree and run on either QuickMenu or MainMenu.",
                size: 11.5f, fg: "muted", wrap: true);

            m.Row(r =>
            {
                r.Badge($"CloseOnBackdrop: {_modalCloseOnBackdrop}", fg: "accent");
                r.Badge($"Clicks: {_clickCount}", fg: "emerald");
            }, new ExtStyle { Gap = 6f });

            m.Row(r =>
            {
                r.Button("m_toast", "Fire Toast", () => LumenicApi.ShowToast("Toast from inside modal!", 2f), icon: "campaign");
                r.Spacer(new ExtStyle { Grow = 1f });
                r.Button("m_close", "Close", () => LumenicApi.CloseModal(surface), variant: "accent");
            }, new ExtStyle { Gap = 8f, Pt = 4f });
        });

        LogEvent($"Opened modal on {surface}");
        RefreshAll();
    }

    // -------------------------------------------------------------------------
    // 5. Sections & Theme Lab: Opt-in toggles for all named slots & theme patches
    // -------------------------------------------------------------------------
    private void BuildSectionsAndThemeLab(UiBuilder ui, bool compact)
    {
        ui.Card(c =>
        {
            c.Row(r =>
            {
                r.Text("Named Section Injections (Opt-In)", size: 12.5f, weight: "bold", style: new ExtStyle { Grow = 1f });
                r.Button("ov_reset", "Reset All", () => ResetAllOverrides(markDirty: true),
                    icon: "restart_alt", variant: "danger", disabled: ActiveOverrideCount() == 0);
            }, new ExtStyle { Align = "center" });

            c.Text("All overrides default to OFF so your menus stay clean until you toggle them on.",
                size: 10.5f, fg: "muted", wrap: true);

            c.Grid(compact ? 1 : 2, g =>
            {
                g.Toggle("slot_qm_banner", "QM Home Banner (qm.home.banner)", _qmBannerHandle != null, SetQmBanner);
                g.Toggle("slot_qm_action", "QM Quick Action Tile (qm.home.actions)", _qmActionTileHandle != null, SetQmActionTile);
                g.Toggle("slot_qm_style", "Restyle QM Actions (2-Col, 1.2x)", _qmActionStyleHandle != null, SetQmActionStyle);
                g.Toggle("slot_qm_player", "QM Player Details (qm.player_details.actions)", _qmPlayerActionHandle != null, SetQmPlayerAction);
                g.Toggle("slot_qm_notif", "Replace QM Notifications (qm.home.notifications)", _qmNotifReplaceHandle != null, SetQmNotifReplace);
                g.Toggle("slot_qm_hide", "Hide Built-In 'chat' Tile", _qmHideChatTileHandle != null, SetHideChatTile);
                g.Toggle("slot_mm_banner", "MM Home Banner (mm.home.banner)", _mmBannerHandle != null, SetMmBanner);
                g.Toggle("slot_mm_tools", "MM Quick Tools (mm.home.quick_tools)", _mmQuickToolHandle != null, SetMmQuickTool);
                g.Toggle("slot_mm_set", "MM Lumenic Settings (mm.settings.lumenic)", _mmSettingsHandle != null, SetMmSettingsCard);
            }, new ExtStyle { Gap = 6f });
        }, new ExtStyle { Gap = 7f });

        ui.Card(c =>
        {
            c.Text("Global Theme Override (LumenicApi.Theme)", size: 12.5f, weight: "bold");
            c.Text("Applies an ExtThemePatch at priority 10. Select 'None' or Reset All to revert.",
                size: 10.5f, fg: "muted", wrap: true);

            c.Dropdown("dd_theme", ThemePresetLabels, _themePresetIdx, ApplyThemePreset,
                style: new ExtStyle { WFrac = 1f });
        }, new ExtStyle { Gap = 7f });
    }

    private void SetQmBanner(bool enabled)
    {
        _qmBannerHandle?.Dispose();
        _qmBannerHandle = null;
        if (enabled)
        {
            _qmBannerHandle = LumenicApi.Sections.Inject(ModId, LumenicSections.QmHomeBanner, ExtSlotPosition.InsideEnd, ui =>
            {
                ui.Card(c => c.Row(r =>
                {
                    r.Icon("widgets", size: 16f, fg: "accent");
                    r.Text("Showcase Banner injected into qm.home.banner", size: 11f, weight: "semibold", ellipsis: true, style: new ExtStyle { Grow = 1f });
                    r.Button("qm_bn_open", "Open", () => _qmTab?.Open(), variant: "accent");
                }, new ExtStyle { Align = "center", Gap = 8f }), new ExtStyle { Px = 10f, Py = 6f });
            }, priority: 10);
        }
        RefreshAll();
    }

    private void SetQmActionTile(bool enabled)
    {
        _qmActionTileHandle?.Dispose();
        _qmActionTileHandle = null;
        if (enabled)
        {
            _qmActionTileHandle = LumenicApi.Sections.Inject(ModId, LumenicSections.QmHomeActions, ExtSlotPosition.InsideEnd, ui =>
            {
                ui.CustomButton("qa_showcase", tile =>
                {
                    tile.Col(col =>
                    {
                        col.Icon("widgets", size: 19f, fg: "accent");
                        col.Text("Showcase", size: 12f, weight: "semibold", align: "center");
                        col.Text($"{_clickCount} clicks", size: 9.5f, fg: "muted", align: "center");
                    }, new ExtStyle { Center = true, Gap = 2.5f, Grow = 1f });
                }, () => _qmTab?.Open(), style: new ExtStyle { Grow = 1f, MinH = 52f, P = 5f, Radius = 7.8f, Bevel = 1.3f });
            }, priority: 10);
        }
        RefreshAll();
    }

    private void SetQmActionStyle(bool enabled)
    {
        _qmActionStyleHandle?.Dispose();
        _qmActionStyleHandle = enabled
            ? LumenicApi.Sections.OverrideStyle(ModId, LumenicSections.QmHomeActions, new ExtSectionStyle
            {
                Columns = 2,
                FontScale = 1.2f,
                MinHeight = 62f
            }, priority: 10)
            : null;
        RefreshAll();
    }

    private void SetQmPlayerAction(bool enabled)
    {
        _qmPlayerActionHandle?.Dispose();
        _qmPlayerActionHandle = null;
        if (enabled)
        {
            _qmPlayerActionHandle = LumenicApi.Sections.Inject(ModId, LumenicSections.QmPlayerDetailsActions, ExtSlotPosition.InsideEnd, ui =>
            {
                ui.Button("pd_wave", "Showcase Wave", () => LumenicApi.ShowToast("Custom player action triggered!", 2.5f),
                    icon: "waving_hand", variant: "accent");
            }, priority: 10);
        }
        RefreshAll();
    }

    private void SetQmNotifReplace(bool enabled)
    {
        _qmNotifReplaceHandle?.Dispose();
        _qmNotifReplaceHandle = null;
        if (enabled)
        {
            _qmNotifReplaceHandle = LumenicApi.Sections.Replace(ModId, LumenicSections.QmHomeNotifications, ui =>
            {
                ui.Card(c => c.Row(r =>
                {
                    r.Icon("swap_horiz", size: 16f, fg: "amber");
                    r.Text("Replaced qm.home.notifications via Sections.Replace", size: 11f, weight: "semibold", wrap: true, style: new ExtStyle { Grow = 1f });
                }, new ExtStyle { Align = "center", Gap = 8f }));
            }, priority: 10);
        }
        RefreshAll();
    }

    private void SetHideChatTile(bool enabled)
    {
        _qmHideChatTileHandle?.Dispose();
        _qmHideChatTileHandle = enabled ? LumenicApi.Sections.HideBuiltInTile(ModId, "chat") : null;
        RefreshAll();
    }

    private void SetMmBanner(bool enabled)
    {
        _mmBannerHandle?.Dispose();
        _mmBannerHandle = null;
        if (enabled)
        {
            _mmBannerHandle = LumenicApi.Sections.Inject(ModId, LumenicSections.MmHomeBanner, ExtSlotPosition.InsideEnd, ui =>
            {
                ui.Card(c => c.Row(r =>
                {
                    r.Icon("widgets", size: 16f, fg: "accent");
                    r.Text("Main Menu banner injected into mm.home.banner", size: 11.5f, weight: "semibold", style: new ExtStyle { Grow = 1f });
                    r.Button("mm_bn_open", "Open Showcase", () => _mmTab?.Open(), variant: "accent");
                }, new ExtStyle { Align = "center", Gap = 8f }), new ExtStyle { Px = 10f, Py = 6f });
            }, priority: 10);
        }
        RefreshAll();
    }

    private void SetMmQuickTool(bool enabled)
    {
        _mmQuickToolHandle?.Dispose();
        _mmQuickToolHandle = null;
        if (enabled)
        {
            _mmQuickToolHandle = LumenicApi.Sections.Inject(ModId, LumenicSections.MmHomeQuickTools, ExtSlotPosition.InsideEnd, ui =>
            {
                ui.Button("mm_qt_showcase", "UI Showcase", () => _mmTab?.Open(), icon: "widgets", variant: "accent");
            }, priority: 10);
        }
        RefreshAll();
    }

    private void SetMmSettingsCard(bool enabled)
    {
        _mmSettingsHandle?.Dispose();
        _mmSettingsHandle = null;
        if (enabled)
        {
            _mmSettingsHandle = LumenicApi.Sections.Inject(ModId, LumenicSections.MmSettingsLumenic, ExtSlotPosition.InsideEnd, ui =>
            {
                ui.Card(c => c.Row(r =>
                {
                    r.Col(col =>
                    {
                        col.Text("UI Showcase Extension", size: 12f, weight: "bold");
                        col.Text("Injected into mm.settings.lumenic", size: 10.5f, fg: "muted");
                    }, new ExtStyle { Grow = 1f, Gap = 2f });
                    r.Button("mm_set_open", "Open Lab", () => _mmTab?.Open(), icon: "open_in_new");
                }, new ExtStyle { Align = "center", Gap = 8f }));
            }, priority: 10);
        }
        RefreshAll();
    }

    private void ApplyThemePreset(int index)
    {
        _themePresetIdx = index;
        _themeOverrideHandle?.Dispose();
        _themeOverrideHandle = null;

        ExtThemePatch patch = index switch
        {
            1 => new ExtThemePatch { FontScaleOverride = 1.35f, TileMinHeight = 62f, QmActionColumns = 2 },
            2 => new ExtThemePatch { AccentOverride = "#38BDF8", BevelScale = 2.0f, RadiusScale = 0.5f },
            3 => new ExtThemePatch { SpacingScale = 0.78f, PaddingScale = 0.8f, FontScaleOverride = 0.92f },
            _ => null
        };

        if (patch != null)
            _themeOverrideHandle = LumenicApi.Theme.SetOverride(ModId, patch, priority: 10);

        LogEvent($"Theme preset: {ThemePresetLabels[index]}");
        RefreshAll();
    }

    private void ResetAllOverrides(bool markDirty)
    {
        _qmBannerHandle?.Dispose(); _qmBannerHandle = null;
        _qmActionTileHandle?.Dispose(); _qmActionTileHandle = null;
        _qmActionStyleHandle?.Dispose(); _qmActionStyleHandle = null;
        _qmPlayerActionHandle?.Dispose(); _qmPlayerActionHandle = null;
        _qmNotifReplaceHandle?.Dispose(); _qmNotifReplaceHandle = null;
        _qmHideChatTileHandle?.Dispose(); _qmHideChatTileHandle = null;
        _mmBannerHandle?.Dispose(); _mmBannerHandle = null;
        _mmQuickToolHandle?.Dispose(); _mmQuickToolHandle = null;
        _mmSettingsHandle?.Dispose(); _mmSettingsHandle = null;
        _themeOverrideHandle?.Dispose(); _themeOverrideHandle = null;
        _themePresetIdx = 0;

        if (markDirty)
        {
            LogEvent("Reset all section & theme overrides");
            RefreshAll();
        }
    }
}
