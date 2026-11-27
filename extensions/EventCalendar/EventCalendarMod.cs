using System;
using System.Collections.Generic;
using System.Linq;
using System.Runtime.CompilerServices;
using LumenicMenu.Api;
using MelonLoader;
using UnityEngine;

[assembly: MelonInfo(typeof(LumenicEventCalendar.EventCalendarMod), "LumenicEventCalendar", "1.0.0", "LensError")]
[assembly: MelonGame("ChilloutVR", "ChilloutVR")]
[assembly: MelonOptionalDependencies("LumenicMenu")]

namespace LumenicEventCalendar;

public sealed class EventCalendarMod : MelonMod
{
    private MelonPreferences_Category _prefs;
    private MelonPreferences_Entry<bool> _use24Hour;
    private MelonPreferences_Entry<string> _savedReminders;

    private CalendarApiService _api;
    private ReminderStore _reminders;
    private EventCalendarUi _ui;
    private float _nextTickTime;
    private bool _lumenicBound;

    public override void OnInitializeMelon()
    {
        _prefs = MelonPreferences.CreateCategory("LumenicEventCalendar", "Event Calendar", is_hidden: true, should_save: true);
        _use24Hour = _prefs.CreateEntry("Use24HourClock", true, "Use 24-Hour Clock", description: null, is_hidden: true, dont_save_default: false);
        _savedReminders = _prefs.CreateEntry("SavedReminders", string.Empty, "Saved Event Reminder IDs", description: null, is_hidden: true, dont_save_default: false);

        _reminders = new ReminderStore(_savedReminders);
    }

    public override void OnLateInitializeMelon()
    {
        if (MelonBase.RegisteredMelons.Any(m => m.Info.Name == "LumenicMenu"))
        {
            InitializeLumenic();
        }
        else
        {
            MelonLogger.Warning("[EventCalendar] LumenicMenu not found; Event Calendar UI will remain inactive.");
        }
    }

    [MethodImpl(MethodImplOptions.NoInlining)]
    private void InitializeLumenic()
    {
        _api = new CalendarApiService();
        _ui = new EventCalendarUi(
            _api,
            _reminders,
            () => _use24Hour.Value,
            v => { _use24Hour.Value = v; _prefs.SaveToFile(false); });

        _ui.Register();
        _api.Start();
        _lumenicBound = true;
        MelonLogger.Msg("[EventCalendar] Registered Main Menu tab and Quick Tools button.");
    }

    public override void OnUpdate()
    {
        if (!_lumenicBound || _api == null || _ui == null)
            return;

        float now = Time.unscaledTime;
        if (now < _nextTickTime)
            return;

        _nextTickTime = now + 30f;
        CheckReminders();
        _ui.TickCountdownRefresh();
    }

    [MethodImpl(MethodImplOptions.NoInlining)]
    private void CheckReminders()
    {
        var (events, _, _, _, _) = _api.GetSnapshot();
        if (events.Count == 0)
            return;

        DateTimeOffset nowUtc = DateTimeOffset.UtcNow;
        for (int i = 0; i < events.Count; i++)
        {
            CalendarEvent ev = events[i];
            if (!ev.IsActive || !ev.NextStartUtc.HasValue || !_reminders.IsEnabled(ev.Id))
                continue;

            double minsUntil = (ev.NextStartUtc.Value - nowUtc).TotalMinutes;
            string occKey = ev.NextStartUtc.Value.ToString("yyyyMMddHHmm");

            if (minsUntil <= 15.0 && minsUntil > 1.0 && _reminders.TryMarkNotified(ev.Id, occKey, "15m"))
            {
                int rounded = Math.Max(1, (int)Math.Ceiling(minsUntil));
                LumenicApi.ShowToast($"Upcoming: {ev.Title} starts in {rounded}m", 5.0f);
            }
            else if (minsUntil <= 1.0 && minsUntil >= -5.0 && _reminders.TryMarkNotified(ev.Id, occKey, "live"))
            {
                LumenicApi.ShowToast($"Starting now: {ev.Title}!", 6.0f);
            }
        }
    }

    public override void OnDeinitializeMelon() => Cleanup();
    public override void OnApplicationQuit() => Cleanup();

    private void Cleanup()
    {
        if (!_lumenicBound)
            return;

        _lumenicBound = false;
        CleanupLumenic();
        _api?.Dispose();
        _api = null;
    }

    [MethodImpl(MethodImplOptions.NoInlining)]
    private void CleanupLumenic()
    {
        _ui?.Dispose();
        _ui = null;
        LumenicApi.UnregisterMod(EventCalendarUi.ModId);
    }
}

internal sealed class ReminderStore
{
    private readonly MelonPreferences_Entry<string> _entry;
    private readonly HashSet<string> _ids = new HashSet<string>(StringComparer.Ordinal);
    private readonly HashSet<string> _firedAlerts = new HashSet<string>(StringComparer.Ordinal);

    public ReminderStore(MelonPreferences_Entry<string> entry)
    {
        _entry = entry;
        string raw = entry?.Value ?? string.Empty;
        if (!string.IsNullOrWhiteSpace(raw))
        {
            string[] parts = raw.Split(new[] { ',' }, StringSplitOptions.RemoveEmptyEntries);
            for (int i = 0; i < parts.Length; i++)
            {
                string id = parts[i].Trim();
                if (id.Length > 0) _ids.Add(id);
            }
        }
    }

    public bool IsEnabled(string eventId)
        => !string.IsNullOrEmpty(eventId) && _ids.Contains(eventId);

    public bool Toggle(string eventId)
    {
        if (string.IsNullOrEmpty(eventId))
            return false;

        bool enabled;
        if (_ids.Contains(eventId))
        {
            _ids.Remove(eventId);
            enabled = false;
        }
        else
        {
            _ids.Add(eventId);
            enabled = true;
        }

        if (_entry != null)
        {
            _entry.Value = string.Join(",", _ids);
            MelonPreferences.Save();
        }
        return enabled;
    }

    public bool TryMarkNotified(string eventId, string occurrenceKey, string stage)
        => _firedAlerts.Add($"{eventId}:{occurrenceKey}:{stage}");
}
