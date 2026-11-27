using System;
using System.Collections.Generic;
using System.Globalization;
using System.Linq;
using LumenicMenu.Api;
using UnityEngine;

namespace LumenicEventCalendar;

internal sealed class EventCalendarUi : IDisposable
{
    public const string ModId = "com.lenserror.eventcalendar";

    private static readonly string[] ViewTabs = { "Upcoming", "By Day" };
    private static readonly string[] StatusOptions = { "Active", "All", "Postponed", "Cancelled" };

    private readonly CalendarApiService _api;
    private readonly ReminderStore _reminders;
    private readonly Func<bool> _getUse24Hour;
    private readonly Action<bool> _setUse24Hour;

    private ExtTabHandle _mainTab;
    private ExtRegistrationHandle _quickToolsSlot;

    private int _viewMode; // 0 = Upcoming, 1 = By Day
    private int _selectedDayIndex = CalendarEvent.DayOfWeekToMondayIndex(DateTimeOffset.Now.DayOfWeek);
    private int _statusFilterIndex; // 0 = Active, 1 = All, 2 = Postponed, 3 = Cancelled
    private string _selectedTag = "All Tags";
    private string _searchQuery = string.Empty;
    private string _selectedEventId = string.Empty;
    private bool _mainMenuOpen;

    public EventCalendarUi(
        CalendarApiService api,
        ReminderStore reminders,
        Func<bool> getUse24Hour,
        Action<bool> setUse24Hour)
    {
        _api = api;
        _reminders = reminders;
        _getUse24Hour = getUse24Hour;
        _setUse24Hour = setUse24Hour;
    }

    public void Register()
    {
        _api.OnDataChanged += MarkDirty;
        LumenicApi.OnMainMenuToggled += OnMainMenuToggled;

        _mainTab = LumenicApi.RegisterMainMenuTab(
            ModId,
            new ExtTabOptions
            {
                Id = "calendar",
                Title = "Events",
                Subtitle = "ChilloutVR Community Event Calendar",
                Icon = "calendar_month",
                BadgeText = GetTabBadgeText,
                Priority = 20,
                HideHeader = true,
                HideFromTabBar = true
            },
            BuildMainMenuPage);

        _quickToolsSlot = LumenicApi.Sections.Inject(
            ModId,
            LumenicSections.MmHomeQuickTools,
            ExtSlotPosition.InsideEnd,
            BuildQuickToolsButton,
            priority: 20,
            slotId: "event_calendar_btn");
    }

    public void MarkDirty()
    {
        _mainTab?.MarkDirty();
        _quickToolsSlot?.MarkDirty();
    }

    public void TickCountdownRefresh()
    {
        if (_mainMenuOpen)
            MarkDirty();
    }

    private void OnMainMenuToggled(bool open)
    {
        _mainMenuOpen = open;
        if (open)
            MarkDirty();
    }

    private string GetTabBadgeText()
    {
        var (events, _, _, _, _) = _api.GetSnapshot();
        DateTimeOffset nowUtc = DateTimeOffset.UtcNow;
        int liveCount = 0;
        for (int i = 0; i < events.Count; i++)
        {
            if (events[i].IsLiveNow(nowUtc))
                liveCount++;
        }
        return liveCount > 0 ? $"{liveCount} LIVE" : null;
    }

    private void BuildQuickToolsButton(UiBuilder ui)
    {
        var (events, isLoading, _, _, _) = _api.GetSnapshot();
        DateTimeOffset nowUtc = DateTimeOffset.UtcNow;

        int liveCount = 0;
        CalendarEvent nextActive = null;
        for (int i = 0; i < events.Count; i++)
        {
            CalendarEvent ev = events[i];
            if (ev.IsLiveNow(nowUtc))
                liveCount++;
            else if (ev.IsActive && nextActive == null && ev.NextStartUtc.HasValue && ev.NextStartUtc.Value > nowUtc)
                nextActive = ev;
        }

        string statusLine;
        string statusFg;
        if (liveCount > 0)
        {
            statusLine = liveCount == 1 ? "1 event live right now" : $"{liveCount} events live right now";
            statusFg = "emerald";
        }
        else if (nextActive != null)
        {
            statusLine = $"{nextActive.Title} - {nextActive.FormatCountdown(nowUtc)}";
            statusFg = "muted";
        }
        else if (isLoading)
        {
            statusLine = "Loading schedule...";
            statusFg = "muted";
        }
        else
        {
            statusLine = "Browse community events";
            statusFg = "muted";
        }

        ui.CustomButton(
            id: "qt_open_event_calendar",
            content: btn =>
            {
                btn.Row(row =>
                {
                    row.Col(ic =>
                    {
                        ic.Icon("calendar_month", size: 17f, fg: liveCount > 0 ? "emerald" : "accent");
                    }, new ExtStyle
                    {
                        W = 30f,
                        H = 30f,
                        Center = true,
                        Bg = liveCount > 0 ? "#10B98124" : "accent_soft",
                        Radius = 7f
                    });

                    row.Col(textCol =>
                    {
                        textCol.Row(top =>
                        {
                            top.Text("Event Calendar", size: 11.5f, weight: "semibold", fg: "primary", ellipsis: true,
                                style: new ExtStyle { Grow = 1f });
                            if (liveCount > 0)
                                top.Badge("LIVE", fg: "emerald", size: 8.5f);
                        }, new ExtStyle { Align = "center", Gap = 5f });

                        textCol.Text(statusLine, size: 9.5f, fg: statusFg, ellipsis: true);
                    }, new ExtStyle { Grow = 1f, Gap = 1.5f });

                    row.Icon("chevron_right", size: 15f, fg: "muted");
                }, new ExtStyle { Align = "center", Gap = 8f, WFrac = 1f });
            },
            onClick: () => _mainTab?.Open(),
            active: liveCount > 0,
            tooltip: "Open ChilloutVR Event Calendar",
            style: new ExtStyle
            {
                WFrac = 1f,
                Px = 9f,
                Py = 7f,
                Bg = liveCount > 0 ? "#162C27EE" : "tile",
                HoverBg = "tile_hover",
                BorderColor = liveCount > 0 ? "emerald" : "border",
                BorderW = 1f,
                Radius = 7.8f,
                Bevel = 1.4f
            });
    }

    private void BuildMainMenuPage(UiBuilder ui)
    {
        var (allEvents, isLoading, wsConnected, lastError, lastUpdated) = _api.GetSnapshot();
        DateTimeOffset nowUtc = DateTimeOffset.UtcNow;
        bool use24h = _getUse24Hour();

        // Collect all unique tags for the filter dropdown.
        var tagOptions = new List<string> { "All Tags" };
        var uniqueTags = new HashSet<string>(StringComparer.OrdinalIgnoreCase);
        for (int i = 0; i < allEvents.Count; i++)
        {
            List<string> tags = allEvents[i].Tags;
            if (tags == null) continue;
            for (int t = 0; t < tags.Count; t++)
            {
                if (!string.IsNullOrWhiteSpace(tags[t]) && uniqueTags.Add(tags[t].Trim()))
                    tagOptions.Add(tags[t].Trim());
            }
        }
        if (tagOptions.Count > 1)
            tagOptions.Sort(1, tagOptions.Count - 1, StringComparer.OrdinalIgnoreCase);

        int selectedTagIdx = tagOptions.FindIndex(t => string.Equals(t, _selectedTag, StringComparison.OrdinalIgnoreCase));
        if (selectedTagIdx < 0)
        {
            selectedTagIdx = 0;
            _selectedTag = "All Tags";
        }

        // Apply filters (status, tag, search, and day when in By Day mode).
        var baseFiltered = FilterEvents(allEvents, applyDayFilter: false);
        var visibleEvents = _viewMode == 1 ? FilterEvents(allEvents, applyDayFilter: true) : baseFiltered;

        // Keep selection valid: preserve current selection if still visible, otherwise auto-select first visible event.
        CalendarEvent selectedEvent = null;
        if (!string.IsNullOrEmpty(_selectedEventId))
            selectedEvent = visibleEvents.FirstOrDefault(e => string.Equals(e.Id, _selectedEventId, StringComparison.Ordinal));
        if (selectedEvent == null && visibleEvents.Count > 0)
        {
            selectedEvent = visibleEvents[0];
            _selectedEventId = selectedEvent.Id;
        }

        int liveCount = allEvents.Count(e => e.IsLiveNow(nowUtc));

        // Single full-canvas back plane for all calendar components
        ui.Card(root =>
        {
            // 1. Compact Header + Toolbar Card
            root.Card(header =>
            {
                header.Row(top =>
                {
                    top.IconButton("ev_back", "arrow_back", () => _mainTab?.Close(), tooltip: "Back to Home");
                    top.Icon("calendar_month", size: 20f, fg: "accent");
                    top.Col(titleCol =>
                    {
                        titleCol.Row(tr =>
                        {
                            tr.Text("Event Calendar", size: 15f, weight: "bold", fg: "primary");
                            if (liveCount > 0)
                                tr.Badge($"{liveCount} LIVE", icon: "sensors", fg: "emerald", size: 9.5f);
                            tr.Badge(wsConnected ? "Live Sync" : "Polling", icon: wsConnected ? "wifi" : "wifi_off",
                                fg: wsConnected ? "accent" : "muted", size: 9f);
                        }, new ExtStyle { Align = "center", Gap = 7f });

                        string tzName = TimeZoneInfo.Local.DisplayName;
                        string updatedStr = lastUpdated.HasValue
                            ? $"Updated {lastUpdated.Value.ToString(use24h ? "HH:mm" : "h:mm tt", CultureInfo.InvariantCulture)}"
                            : "Syncing...";
                        titleCol.Text($"Local time: {tzName}  |  {updatedStr}", size: 9.5f, fg: "muted", ellipsis: true);
                    }, new ExtStyle { Grow = 1f, Gap = 1f });

                    top.Tabs("ev_view_mode", ViewTabs, _viewMode, idx =>
                    {
                        _viewMode = idx;
                        MarkDirty();
                    }, size: 10.5f);

                    top.Button("ev_clock_fmt", use24h ? "24h" : "12h", () =>
                    {
                        _setUse24Hour(!use24h);
                        MarkDirty();
                    }, icon: "schedule", size: 10.5f, tooltip: "Toggle 12-hour / 24-hour clock");

                    top.IconButton("ev_refresh", "refresh", () => _api.TriggerRefresh(manual: true),
                        variant: isLoading ? "accent" : null,
                        disabled: isLoading,
                        tooltip: "Refresh events from API");
                }, new ExtStyle { Align = "center", Gap = 8f });

                // Second row: Search + Status filter + Tag dropdown + Clear filters
                header.Row(filters =>
                {
                    filters.TextInput(
                        id: "ev_search",
                        text: _searchQuery,
                        placeholder: "Search events, tags, descriptions...",
                        onChange: s => { _searchQuery = s ?? string.Empty; MarkDirty(); },
                        onSubmit: s => { _searchQuery = s ?? string.Empty; MarkDirty(); },
                        maxLen: 80,
                        style: new ExtStyle { Grow = 1f });

                    if (!string.IsNullOrEmpty(_searchQuery))
                    {
                        filters.IconButton("ev_clear_search", "close", () =>
                        {
                            _searchQuery = string.Empty;
                            MarkDirty();
                        }, variant: "ghost", tooltip: "Clear search");
                    }

                    filters.Tabs("ev_status_filter", StatusOptions, _statusFilterIndex, idx =>
                    {
                        _statusFilterIndex = idx;
                        MarkDirty();
                    }, size: 10f);

                    filters.Dropdown(
                        id: "ev_tag_filter",
                        options: tagOptions,
                        selectedIndex: selectedTagIdx,
                        onChange: idx =>
                        {
                            if (idx >= 0 && idx < tagOptions.Count)
                                _selectedTag = tagOptions[idx];
                            MarkDirty();
                        },
                        searchable: tagOptions.Count > 8,
                        tooltip: "Filter by tag",
                        style: new ExtStyle { W = 165f });
                }, new ExtStyle { Align = "center", Gap = 8f });

                // Optional By-Day quick-jump strip (Mon - Sun)
                if (_viewMode == 1)
                {
                    int todayIdx = CalendarEvent.DayOfWeekToMondayIndex(DateTimeOffset.Now.DayOfWeek);
                    header.Row(days =>
                    {
                        for (int d = 0; d < 7; d++)
                        {
                            int dayIdx = d;
                            int count = baseFiltered.Count(e => e.LocalDayIndex == dayIdx);
                            bool isSelected = _selectedDayIndex == dayIdx;
                            bool isToday = todayIdx == dayIdx;
                            string label = isToday
                                ? $"{CalendarEvent.DayShortName(dayIdx)} (Today)"
                                : CalendarEvent.DayShortName(dayIdx);

                            days.CustomButton(
                                id: $"ev_day_{dayIdx}",
                                content: b =>
                                {
                                    b.Row(r =>
                                    {
                                        r.Text(label, size: 10.5f, weight: isSelected ? "bold" : "semibold",
                                            fg: isSelected ? "accent" : "primary");
                                        if (count > 0)
                                            r.Badge(count.ToString(CultureInfo.InvariantCulture),
                                                fg: isSelected ? "accent" : "muted", size: 9f);
                                    }, new ExtStyle { Center = true, Gap = 5f });
                                },
                                onClick: () =>
                                {
                                    _selectedDayIndex = dayIdx;
                                    MarkDirty();
                                },
                                active: isSelected,
                                style: new ExtStyle { Grow = 1f, Py = 5f, Px = 6f, Radius = 6f });
                        }
                    }, new ExtStyle { Gap = 5f, Align = "center" });
                }
            }, new ExtStyle
            {
                Px = 12f,
                Py = 9f,
                Gap = 7f,
                Bg = "card_inner",
                BorderColor = "border",
                BorderW = 1f
            });

            // Error banner if API fetch failed
            if (!string.IsNullOrEmpty(lastError))
            {
                root.Row(err =>
                {
                    err.Icon("warning", size: 15f, fg: "amber");
                    err.Text(lastError, size: 10.5f, fg: "amber", ellipsis: true, style: new ExtStyle { Grow = 1f });
                    err.Button("ev_retry", "Retry", () => _api.TriggerRefresh(manual: true), icon: "refresh", size: 10f);
                }, new ExtStyle
                {
                    Align = "center",
                    Gap = 8f,
                    Px = 10f,
                    Py = 6f,
                    Bg = "#352814EE",
                    BorderColor = "amber",
                    BorderW = 1f,
                    Radius = 7f
                });
            }

            // 2. Main Split Content: Left Event List (56%) + Right Instant Detail Pane (44%)
            root.Row(split =>
            {
                // Left Pane: Scrollable event cards
                split.Card(left =>
                {
                    left.Row(meta =>
                    {
                        string heading = _viewMode == 1
                            ? $"{CalendarEvent.DayFullName(_selectedDayIndex)} ({visibleEvents.Count})"
                            : $"Upcoming Events ({visibleEvents.Count})";
                        meta.Text(heading, size: 11f, weight: "semibold", fg: "muted", style: new ExtStyle { Grow = 1f });

                        if (_statusFilterIndex != 0 || !string.IsNullOrEmpty(_searchQuery) || _selectedTag != "All Tags")
                        {
                            meta.Button("ev_reset_filters", "Reset Filters", () =>
                            {
                                _statusFilterIndex = 0;
                                _selectedTag = "All Tags";
                                _searchQuery = string.Empty;
                                MarkDirty();
                            }, variant: "ghost", size: 9.5f);
                        }
                    }, new ExtStyle { Align = "center", Px = 2f });

                    left.Scroll("ev_list_scroll", list =>
                    {
                        if (isLoading && allEvents.Count == 0)
                        {
                            list.Card(c =>
                            {
                                c.Col(col =>
                                {
                                    col.Icon("sync", size: 26f, fg: "accent");
                                    col.Text("Loading ChilloutVR events...", size: 12f, weight: "semibold");
                                }, new ExtStyle { Center = true, Gap = 6f, Py = 32f });
                            }, new ExtStyle { Bg = "tile" });
                        }
                        else if (visibleEvents.Count == 0)
                        {
                            list.Card(c =>
                            {
                                c.Col(col =>
                                {
                                    col.Icon("event_busy", size: 26f, fg: "muted");
                                    col.Text("No matching events", size: 12.5f, weight: "semibold");
                                    col.Text("Try switching day, status, or clearing your search.", size: 10.5f, fg: "muted");
                                }, new ExtStyle { Center = true, Gap = 5f, Py = 28f });
                            }, new ExtStyle { Bg = "tile" });
                        }
                        else
                        {
                            for (int i = 0; i < visibleEvents.Count; i++)
                            {
                                CalendarEvent ev = visibleEvents[i];
                                bool isSelected = selectedEvent != null && string.Equals(ev.Id, selectedEvent.Id, StringComparison.Ordinal);
                                BuildEventListCard(list, ev, isSelected, nowUtc, use24h);
                            }
                        }
                    }, new ExtStyle { Grow = 1f, Gap = 6f }, hideScrollbar: false);
                }, new ExtStyle
                {
                    Grow = 1.25f,
                    HFrac = 1f,
                    Px = 10f,
                    Py = 9f,
                    Gap = 7f,
                    Bg = "card_inner",
                    BorderColor = "border",
                    BorderW = 1f
                });

                // Right Pane: Selected Event Details
                split.Col(right =>
                {
                    if (selectedEvent != null)
                    {
                        BuildEventDetailPanel(right, selectedEvent, nowUtc, use24h);
                    }
                    else
                    {
                        right.Card(empty =>
                        {
                            empty.Col(col =>
                            {
                                col.Icon("event_note", size: 32f, fg: "muted");
                                col.Text("Select an event", size: 13f, weight: "semibold");
                                col.Text("Pick an event on the left to view full details, Discord invite, and reminders.",
                                    size: 10.5f, fg: "muted", align: "center", wrap: true);
                            }, new ExtStyle { Center = true, Gap = 8f, P = 24f, Grow = 1f });
                        }, new ExtStyle
                        {
                            Grow = 1f,
                            WFrac = 1f,
                            Bg = "card_inner",
                            BorderColor = "border",
                            BorderW = 1f
                        });
                    }
                }, new ExtStyle { Grow = 1f, HFrac = 1f });
            }, new ExtStyle { Grow = 1f, Gap = 10f });
        }, new ExtStyle
        {
            WFrac = 1f,
            HFrac = 1f,
            P = 10f,
            Gap = 8f,
            Bg = "card",
            BorderColor = "border",
            BorderW = 1f,
            Radius = 11f,
            Bevel = 1.6f,
            Shadow = 14f
        });
    }

    private void BuildEventListCard(UiBuilder list, CalendarEvent ev, bool isSelected, DateTimeOffset nowUtc, bool use24h)
    {
        bool isLive = ev.IsLiveNow(nowUtc);
        bool isSoon = ev.IsStartingSoon(nowUtc, 60);
        bool hasReminder = _reminders.IsEnabled(ev.Id);

        string countdownColor = isLive ? "emerald" : (ev.IsCancelled ? "red" : (ev.IsPostponed ? "amber" : (isSoon ? "accent" : "muted")));

        list.CustomButton(
            id: $"ev_card_{ev.Id}",
            content: card =>
            {
                card.Row(row =>
                {
                    // Event Logo Thumbnail
                    row.Image(
                        imageKey: ev.LogoImageKey,
                        fit: ExtImageFit.Cover,
                        fallbackIcon: "event",
                        style: new ExtStyle
                        {
                            W = 54f,
                            H = 54f,
                            Radius = 8f,
                            Bg = "card_inner"
                        });

                    // Middle Info
                    row.Col(info =>
                    {
                        info.Row(titleRow =>
                        {
                            titleRow.Text(ev.Title, size: 12.5f, weight: "bold", fg: "primary", ellipsis: true,
                                style: new ExtStyle { Grow = 1f });

                            if (hasReminder)
                                titleRow.Icon("notifications_active", size: 13f, fg: "accent", tooltip: "Reminder active");

                            if (isLive)
                                titleRow.Badge("LIVE", fg: "emerald", size: 8.5f);
                            else if (ev.IsPostponed)
                                titleRow.Badge("Postponed", fg: "amber", size: 8.5f);
                            else if (ev.IsCancelled)
                                titleRow.Badge("Cancelled", fg: "red", size: 8.5f);
                        }, new ExtStyle { Align = "center", Gap = 5f });

                        info.Row(timeRow =>
                        {
                            timeRow.Icon("schedule", size: 11.5f, fg: "muted");
                            timeRow.Text($"{ev.FormatLocalDateShort()}  |  {ev.FormatLocalTimeRange(use24h)}",
                                size: 10.5f, weight: "medium", fg: "primary", ellipsis: true,
                                style: new ExtStyle { Grow = 1f });
                            timeRow.Text(ev.FormatCountdown(nowUtc), size: 10f, weight: "semibold", fg: countdownColor);
                        }, new ExtStyle { Align = "center", Gap = 5f });

                        // Tags + Recurrence compact strip
                        info.Row(tagRow =>
                        {
                            tagRow.Badge(ev.FormatRecurrence(), icon: "repeat", fg: "muted", size: 8.5f);
                            int shown = Math.Min(3, ev.Tags.Count);
                            for (int t = 0; t < shown; t++)
                            {
                                string tag = ev.Tags[t];
                                string tagFg = string.Equals(tag, "18+", StringComparison.OrdinalIgnoreCase) ? "red" : "accent";
                                tagRow.Badge(tag, fg: tagFg, size: 8.5f);
                            }
                            if (ev.Tags.Count > shown)
                                tagRow.Text($"+{ev.Tags.Count - shown}", size: 9f, fg: "dim");
                        }, new ExtStyle { Align = "center", Gap = 4f });
                    }, new ExtStyle { Grow = 1f, Gap = 4f });
                }, new ExtStyle { Align = "center", Gap = 10f, WFrac = 1f });
            },
            onClick: () =>
            {
                _selectedEventId = ev.Id;
                MarkDirty();
            },
            active: isSelected,
            style: new ExtStyle
            {
                WFrac = 1f,
                Px = 10f,
                Py = 8f,
                Radius = 8.5f,
                Bevel = 1.5f
            });
    }

    private void BuildEventDetailPanel(UiBuilder parent, CalendarEvent ev, DateTimeOffset nowUtc, bool use24h)
    {
        bool isLive = ev.IsLiveNow(nowUtc);
        bool hasReminder = _reminders.IsEnabled(ev.Id);
        bool hasDiscord = !string.IsNullOrEmpty(ev.DiscordInviteUrl);

        parent.Card(card =>
        {
            card.Scroll("ev_detail_scroll", body =>
            {
                // Banner Header (with logo fallback if no banner)
                string heroKey = !string.IsNullOrEmpty(ev.BannerImageKey) ? ev.BannerImageKey : ev.LogoImageKey;
                body.ZStack(hero =>
                {
                    hero.Image(
                        imageKey: heroKey,
                        fit: ExtImageFit.Cover,
                        fallbackIcon: "calendar_month",
                        style: new ExtStyle
                        {
                            WFrac = 1f,
                            H = 128f,
                            Radius = 8f,
                            Bg = "card_inner"
                        });
                }, new ExtStyle { WFrac = 1f, H = 128f, Radius = 8f, Clip = true });

                // Title + Logo + Status
                body.Row(head =>
                {
                    if (!string.IsNullOrEmpty(ev.LogoImageKey))
                    {
                        head.Image(ev.LogoImageKey, ExtImageFit.Cover, fallbackIcon: "event",
                            style: new ExtStyle { W = 42f, H = 42f, Radius = 7f, Bg = "card_inner" });
                    }

                    head.Col(tc =>
                    {
                        tc.Text(ev.Title, size: 15f, weight: "bold", fg: "primary", wrap: true);
                        tc.Row(badges =>
                        {
                            if (isLive)
                                badges.Badge("LIVE NOW", icon: "sensors", fg: "emerald", size: 9.5f);
                            else if (ev.IsActive)
                                badges.Badge("Active", fg: "accent", size: 9.5f);
                            else if (ev.IsPostponed)
                                badges.Badge("Postponed", fg: "amber", size: 9.5f);
                            else
                                badges.Badge("Cancelled", fg: "red", size: 9.5f);

                            badges.Text(ev.FormatCountdown(nowUtc), size: 10.5f, weight: "semibold",
                                fg: isLive ? "emerald" : "muted");
                        }, new ExtStyle { Align = "center", Gap = 6f });
                    }, new ExtStyle { Grow = 1f, Gap = 3f });
                }, new ExtStyle { Align = "center", Gap = 9f });

                // Live progress bar if currently happening
                if (isLive)
                {
                    body.ProgressBar(ev.GetLiveProgress(nowUtc), 0f, 1f, fg: "emerald",
                        style: new ExtStyle { WFrac = 1f, H = 5f });
                }

                body.Divider();

                // Schedule info grid (2 columns)
                body.Grid(2, grid =>
                {
                    BuildInfoTile(grid, "today", "When", ev.FormatLocalDateShort());
                    BuildInfoTile(grid, "schedule", "Local Time", ev.FormatLocalTimeRange(use24h));
                    BuildInfoTile(grid, "timer", "Duration", ev.FormatDuration());
                    BuildInfoTile(grid, "repeat", "Recurrence", ev.FormatRecurrence());
                }, new ExtStyle { Gap = 6f, WFrac = 1f });

                // Original timezone note
                body.Row(tz =>
                {
                    tz.Icon("public", size: 13f, fg: "muted");
                    tz.Text($"Host schedule: {ev.FormatOriginalTimezone()}", size: 10f, fg: "muted", ellipsis: true);
                }, new ExtStyle { Align = "center", Gap = 6f, Px = 8f, Py = 5f, Bg = "tile", Radius = 6f });

                // Description
                body.Col(desc =>
                {
                    desc.Text("About", size: 10f, weight: "semibold", fg: "muted");
                    desc.Text(
                        string.IsNullOrWhiteSpace(ev.Description) ? "No description provided." : ev.Description,
                        size: 11f,
                        fg: string.IsNullOrWhiteSpace(ev.Description) ? "dim" : "primary",
                        wrap: true);
                }, new ExtStyle { Gap = 3f, Px = 9f, Py = 8f, Bg = "tile", Radius = 7f, WFrac = 1f });

                // Tags
                if (ev.Tags.Count > 0)
                {
                    body.HScroll("ev_detail_tags", tagsRow =>
                    {
                        tagsRow.Icon("label", size: 13f, fg: "muted");
                        for (int i = 0; i < ev.Tags.Count; i++)
                        {
                            string tag = ev.Tags[i];
                            string fg = string.Equals(tag, "18+", StringComparison.OrdinalIgnoreCase) ? "red" : "accent";
                            tagsRow.Badge(tag, fg: fg, size: 9.5f);
                        }
                    }, new ExtStyle { Align = "center", Gap = 5f, Px = 8f, Py = 5f, Bg = "tile", Radius = 6f, WFrac = 1f });
                }

                // Discord Invite Box
                if (hasDiscord)
                {
                    body.Row(disc =>
                    {
                        disc.Icon("forum", size: 14f, fg: "accent");
                        disc.Col(dc =>
                        {
                            dc.Text("Discord Server", size: 9.5f, fg: "muted");
                            dc.Text(ev.DiscordInviteUrl, size: 10.5f, weight: "semibold", fg: "primary", ellipsis: true);
                        }, new ExtStyle { Grow = 1f, Gap = 1f });
                    }, new ExtStyle
                    {
                        Align = "center",
                        Gap = 8f,
                        Px = 9f,
                        Py = 7f,
                        Bg = "accent_soft",
                        BorderColor = "accent_border",
                        BorderW = 1f,
                        Radius = 7f
                    });
                }
            }, new ExtStyle { Grow = 1f, Gap = 8f }, hideScrollbar: false);

            // Pinned Action Bar at bottom of Detail Card
            card.Divider();
            card.Row(actions =>
            {
                if (hasDiscord)
                {
                    actions.Button(
                        id: "ev_copy_discord",
                        label: "Copy Discord",
                        onClick: () =>
                        {
                            GUIUtility.systemCopyBuffer = ev.DiscordInviteUrl;
                            LumenicApi.ShowToast($"Copied {ev.DiscordInviteUrl}", 2.5f);
                        },
                        icon: "content_copy",
                        variant: "accent",
                        style: new ExtStyle { Grow = 1f });

                    actions.Button(
                        id: "ev_open_discord",
                        label: "Open",
                        onClick: () =>
                        {
                            Application.OpenURL(ev.DiscordInviteUrl);
                            LumenicApi.ShowToast("Opened Discord invite in browser", 2.5f);
                        },
                        icon: "open_in_new",
                        tooltip: "Open Discord invite in your system browser");
                }

                actions.Button(
                    id: "ev_toggle_reminder",
                    label: hasReminder ? "Reminding" : "Remind Me",
                    onClick: () =>
                    {
                        bool nowEnabled = _reminders.Toggle(ev.Id);
                        LumenicApi.ShowToast(
                            nowEnabled ? $"Reminder set for {ev.Title}" : $"Reminder removed for {ev.Title}",
                            2.5f);
                        MarkDirty();
                    },
                    icon: hasReminder ? "notifications_active" : "notifications",
                    variant: hasReminder ? "emerald" : null,
                    active: hasReminder,
                    disabled: !ev.IsActive,
                    tooltip: "Show an in-game toast 15 minutes before and when the event starts",
                    style: hasDiscord ? null : new ExtStyle { Grow = 1f });
            }, new ExtStyle { Align = "center", Gap = 6f });
        }, new ExtStyle
        {
            Grow = 1f,
            WFrac = 1f,
            Px = 11f,
            Py = 10f,
            Gap = 8f,
            Bg = "card_inner",
            BorderColor = "border",
            BorderW = 1f
        });
    }

    private static void BuildInfoTile(UiBuilder grid, string icon, string label, string value)
    {
        grid.Row(tile =>
        {
            tile.Icon(icon, size: 15f, fg: "accent");
            tile.Col(col =>
            {
                col.Text(label, size: 9f, fg: "muted");
                col.Text(value, size: 11f, weight: "semibold", fg: "primary", ellipsis: true);
            }, new ExtStyle { Grow = 1f, Gap = 1f });
        }, new ExtStyle
        {
            Align = "center",
            Gap = 7f,
            Px = 8f,
            Py = 6f,
            Bg = "tile",
            Radius = 6.5f
        });
    }

    private List<CalendarEvent> FilterEvents(List<CalendarEvent> source, bool applyDayFilter)
    {
        string query = _searchQuery?.Trim() ?? string.Empty;
        bool hasQuery = query.Length > 0;
        bool hasTag = !string.Equals(_selectedTag, "All Tags", StringComparison.OrdinalIgnoreCase);

        var result = new List<CalendarEvent>(source.Count);
        for (int i = 0; i < source.Count; i++)
        {
            CalendarEvent ev = source[i];

            // Status filter: 0=Active, 1=All, 2=Postponed, 3=Cancelled
            if (_statusFilterIndex == 0 && !ev.IsActive) continue;
            if (_statusFilterIndex == 2 && !ev.IsPostponed) continue;
            if (_statusFilterIndex == 3 && !ev.IsCancelled) continue;

            if (applyDayFilter && ev.LocalDayIndex != _selectedDayIndex)
                continue;

            if (hasTag && !ev.Tags.Any(t => string.Equals(t, _selectedTag, StringComparison.OrdinalIgnoreCase)))
                continue;

            if (hasQuery)
            {
                bool match = ev.Title.IndexOf(query, StringComparison.OrdinalIgnoreCase) >= 0
                    || ev.Description.IndexOf(query, StringComparison.OrdinalIgnoreCase) >= 0
                    || ev.Tags.Any(t => t.IndexOf(query, StringComparison.OrdinalIgnoreCase) >= 0);
                if (!match) continue;
            }

            result.Add(ev);
        }

        DateTimeOffset nowUtc = DateTimeOffset.UtcNow;
        result.Sort((a, b) =>
        {
            int aRank = a.IsLiveNow(nowUtc) ? 0 : (a.IsActive ? 1 : 2);
            int bRank = b.IsLiveNow(nowUtc) ? 0 : (b.IsActive ? 1 : 2);
            int cmp = aRank.CompareTo(bRank);
            if (cmp != 0) return cmp;

            DateTimeOffset aTime = a.NextStartUtc ?? DateTimeOffset.MaxValue;
            DateTimeOffset bTime = b.NextStartUtc ?? DateTimeOffset.MaxValue;
            cmp = aTime.CompareTo(bTime);
            if (cmp != 0) return cmp;

            return string.Compare(a.Title, b.Title, StringComparison.OrdinalIgnoreCase);
        });

        return result;
    }

    public void Dispose()
    {
        _api.OnDataChanged -= MarkDirty;
        LumenicApi.OnMainMenuToggled -= OnMainMenuToggled;
        _quickToolsSlot?.Dispose();
        _mainTab?.Dispose();
        _quickToolsSlot = null;
        _mainTab = null;
    }
}
