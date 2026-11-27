using System;
using System.Collections.Generic;
using System.Globalization;
using Newtonsoft.Json;

namespace LumenicEventCalendar;

internal sealed class CalendarEvent
{
    [JsonProperty("id")] public string Id { get; set; } = string.Empty;
    [JsonProperty("title")] public string Title { get; set; } = string.Empty;
    [JsonProperty("status")] public string Status { get; set; } = "active";
    [JsonProperty("day")] public int Day { get; set; }
    [JsonProperty("startHour")] public int StartHour { get; set; }
    [JsonProperty("startMinute")] public int StartMinute { get; set; }
    [JsonProperty("timezone")] public object Timezone { get; set; }
    [JsonProperty("durationMinutes")] public int DurationMinutes { get; set; }
    [JsonProperty("recurrence")] public string Recurrence { get; set; } = "weekly";
    [JsonProperty("description")] public string Description { get; set; } = string.Empty;
    [JsonProperty("serverCode")] public string ServerCode { get; set; } = string.Empty;
    [JsonProperty("logoUrl")] public string LogoUrl { get; set; } = string.Empty;
    [JsonProperty("bannerUrl")] public string BannerUrl { get; set; } = string.Empty;
    [JsonProperty("tags")] public List<string> Tags { get; set; } = new List<string>();
    [JsonProperty("originalStartHour")] public int? OriginalStartHour { get; set; }
    [JsonProperty("originalStartMinute")] public int? OriginalStartMinute { get; set; }
    [JsonProperty("originalTimezone")] public object OriginalTimezone { get; set; }
    [JsonProperty("nextOccurrenceUtc")] public string NextOccurrenceUtcRaw { get; set; } = string.Empty;

    [JsonIgnore] public DateTimeOffset? NextStartUtc { get; private set; }
    [JsonIgnore] public DateTimeOffset? NextStartLocal { get; private set; }
    [JsonIgnore] public DateTimeOffset? NextEndUtc { get; private set; }
    [JsonIgnore] public DateTimeOffset? NextEndLocal { get; private set; }
    [JsonIgnore] public int LocalDayIndex { get; private set; }
    [JsonIgnore] public string LogoImageKey { get; private set; } = string.Empty;
    [JsonIgnore] public string BannerImageKey { get; private set; } = string.Empty;
    [JsonIgnore] public string ResolvedLogoUrl { get; private set; } = string.Empty;
    [JsonIgnore] public string ResolvedBannerUrl { get; private set; } = string.Empty;

    [JsonIgnore] public string NormalizedStatus => (Status ?? "active").Trim().ToLowerInvariant();
    [JsonIgnore] public bool IsActive => NormalizedStatus == "active";
    [JsonIgnore] public bool IsCancelled => NormalizedStatus == "cancelled";
    [JsonIgnore] public bool IsPostponed => NormalizedStatus == "postponed";

    [JsonIgnore]
    public string DiscordInviteUrl =>
        IsValidServerCode(ServerCode)
            ? $"https://discord.gg/{ServerCode.Trim()}"
            : string.Empty;

    private static bool IsValidServerCode(string code)
    {
        if (string.IsNullOrWhiteSpace(code)) return false;
        string trimmed = code.Trim();
        if (trimmed.Length == 0 || trimmed.Length > 30) return false;
        for (int i = 0; i < trimmed.Length; i++)
        {
            char c = trimmed[i];
            bool ok = (c >= 'a' && c <= 'z')
                || (c >= 'A' && c <= 'Z')
                || (c >= '0' && c <= '9')
                || c == '_'
                || c == '-';
            if (!ok) return false;
        }
        return true;
    }

    public void Hydrate(string apiBaseUrl)
    {
        Tags ??= new List<string>();
        Title ??= string.Empty;
        Description ??= string.Empty;
        ServerCode ??= string.Empty;

        if (!string.IsNullOrWhiteSpace(NextOccurrenceUtcRaw)
            && DateTimeOffset.TryParse(
                NextOccurrenceUtcRaw,
                CultureInfo.InvariantCulture,
                DateTimeStyles.AssumeUniversal | DateTimeStyles.AdjustToUniversal,
                out DateTimeOffset parsedUtc))
        {
            NextStartUtc = parsedUtc;
        }
        else
        {
            NextStartUtc = ComputeFallbackUtc();
        }

        if (NextStartUtc.HasValue)
        {
            DateTimeOffset local = NextStartUtc.Value.ToLocalTime();
            NextStartLocal = local;
            int effDuration = DurationMinutes > 0 ? DurationMinutes : 60;
            NextEndUtc = NextStartUtc.Value.AddMinutes(effDuration);
            NextEndLocal = NextEndUtc.Value.ToLocalTime();
            LocalDayIndex = DayOfWeekToMondayIndex(local.DayOfWeek);
        }
        else
        {
            LocalDayIndex = Math.Clamp(Day, 0, 6);
        }

        ResolvedLogoUrl = ResolveUrl(apiBaseUrl, LogoUrl);
        ResolvedBannerUrl = ResolveUrl(apiBaseUrl, BannerUrl);

        // Include URL hash suffix so replaced images on the server get a fresh texture key.
        LogoImageKey = string.IsNullOrEmpty(ResolvedLogoUrl)
            ? string.Empty
            : $"ext:evcal_logo_{Id}_{StableHash(ResolvedLogoUrl)}";
        BannerImageKey = string.IsNullOrEmpty(ResolvedBannerUrl)
            ? string.Empty
            : $"ext:evcal_banner_{Id}_{StableHash(ResolvedBannerUrl)}";
    }

    public bool IsLiveNow(DateTimeOffset nowUtc)
    {
        if (!IsActive || !NextStartUtc.HasValue || !NextEndUtc.HasValue)
            return false;
        return nowUtc >= NextStartUtc.Value && nowUtc < NextEndUtc.Value;
    }

    public bool IsStartingSoon(DateTimeOffset nowUtc, int withinMinutes = 60)
    {
        if (!IsActive || !NextStartUtc.HasValue)
            return false;
        TimeSpan delta = NextStartUtc.Value - nowUtc;
        return delta > TimeSpan.Zero && delta.TotalMinutes <= withinMinutes;
    }

    public float GetLiveProgress(DateTimeOffset nowUtc)
    {
        if (!NextStartUtc.HasValue || !NextEndUtc.HasValue)
            return 0f;
        double total = (NextEndUtc.Value - NextStartUtc.Value).TotalSeconds;
        if (total <= 0)
            return 0f;
        double elapsed = (nowUtc - NextStartUtc.Value).TotalSeconds;
        return (float)Math.Clamp(elapsed / total, 0.0, 1.0);
    }

    public string FormatCountdown(DateTimeOffset nowUtc)
    {
        if (IsCancelled) return "Cancelled";
        if (IsPostponed) return "Postponed";
        if (!NextStartUtc.HasValue) return "TBD";

        if (IsLiveNow(nowUtc))
        {
            TimeSpan remaining = NextEndUtc.Value - nowUtc;
            if (remaining.TotalHours >= 1)
                return $"LIVE NOW ({(int)remaining.TotalHours}h {remaining.Minutes}m left)";
            return $"LIVE NOW ({Math.Max(1, (int)Math.Ceiling(remaining.TotalMinutes))}m left)";
        }

        TimeSpan diff = NextStartUtc.Value - nowUtc;
        if (diff <= TimeSpan.Zero)
            return "Starting now";
        if (diff.TotalDays >= 1)
            return $"In {(int)diff.TotalDays}d {diff.Hours}h";
        if (diff.TotalHours >= 1)
            return $"In {(int)diff.TotalHours}h {diff.Minutes}m";
        return $"In {Math.Max(1, (int)Math.Ceiling(diff.TotalMinutes))}m";
    }

    public string FormatLocalTimeRange(bool use24Hour)
    {
        if (!NextStartLocal.HasValue)
            return $"{StartHour:00}:{StartMinute:00} UTC";

        string fmt = use24Hour ? "HH:mm" : "h:mm tt";
        string start = NextStartLocal.Value.ToString(fmt, CultureInfo.InvariantCulture);
        if (DurationMinutes > 0 && NextEndLocal.HasValue)
        {
            string end = NextEndLocal.Value.ToString(fmt, CultureInfo.InvariantCulture);
            return $"{start} - {end}";
        }
        return start;
    }

    public string FormatLocalDateShort()
    {
        if (!NextStartLocal.HasValue)
            return DayShortName(LocalDayIndex);

        DateTimeOffset local = NextStartLocal.Value;
        DateTime today = DateTimeOffset.Now.Date;
        if (local.Date == today)
            return "Today";
        if (local.Date == today.AddDays(1))
            return "Tomorrow";
        return local.ToString("ddd, MMM d", CultureInfo.InvariantCulture);
    }

    public string FormatDuration()
    {
        if (DurationMinutes <= 0) return "Open-ended";
        int h = DurationMinutes / 60;
        int m = DurationMinutes % 60;
        if (h > 0 && m > 0) return $"{h}h {m}m";
        if (h > 0) return $"{h}h";
        return $"{m}m";
    }

    public string FormatRecurrence()
    {
        if (string.IsNullOrWhiteSpace(Recurrence)) return "Weekly";
        string r = Recurrence.Trim();
        return char.ToUpperInvariant(r[0]) + r.Substring(1);
    }

    public string FormatOriginalTimezone()
    {
        string tz = OriginalTimezone?.ToString();
        if (string.IsNullOrWhiteSpace(tz) || tz == "0")
            return "UTC";
        if (OriginalStartHour.HasValue && OriginalStartMinute.HasValue)
            return $"{OriginalStartHour.Value:00}:{OriginalStartMinute.Value:00} ({tz})";
        return tz;
    }

    public static int DayOfWeekToMondayIndex(DayOfWeek dow)
        => dow == DayOfWeek.Sunday ? 6 : (int)dow - 1;

    public static string DayShortName(int mondayIndex) => mondayIndex switch
    {
        0 => "Mon",
        1 => "Tue",
        2 => "Wed",
        3 => "Thu",
        4 => "Fri",
        5 => "Sat",
        6 => "Sun",
        _ => "?"
    };

    public static string DayFullName(int mondayIndex) => mondayIndex switch
    {
        0 => "Monday",
        1 => "Tuesday",
        2 => "Wednesday",
        3 => "Thursday",
        4 => "Friday",
        5 => "Saturday",
        6 => "Sunday",
        _ => "Unknown"
    };

    private DateTimeOffset ComputeFallbackUtc()
    {
        DateTimeOffset now = DateTimeOffset.UtcNow;
        int targetDay = Math.Clamp(Day, 0, 6);
        int todayIdx = DayOfWeekToMondayIndex(now.DayOfWeek);
        int deltaDays = (targetDay - todayIdx + 7) % 7;
        DateTimeOffset candidate = new DateTimeOffset(
            now.Year, now.Month, now.Day,
            Math.Clamp(StartHour, 0, 23),
            Math.Clamp(StartMinute, 0, 59),
            0, TimeSpan.Zero).AddDays(deltaDays);

        int dur = DurationMinutes > 0 ? DurationMinutes : 60;
        if (candidate.AddMinutes(dur) <= now)
            candidate = candidate.AddDays(7);
        return candidate;
    }

    private static string ResolveUrl(string baseUrl, string url)
    {
        if (string.IsNullOrWhiteSpace(url)) return string.Empty;
        string trimmed = url.Trim();

        if (!Uri.TryCreate(baseUrl, UriKind.Absolute, out Uri baseUri))
            return string.Empty;

        // Only allow relative /uploads/ paths or HTTPS URLs hosted on the Calendar API origin.
        if (trimmed.StartsWith("/uploads/", StringComparison.OrdinalIgnoreCase)
            && !trimmed.Contains("..", StringComparison.Ordinal)
            && !trimmed.Contains('\\'))
        {
            return baseUrl.TrimEnd('/') + trimmed;
        }

        if (Uri.TryCreate(trimmed, UriKind.Absolute, out Uri abs)
            && abs.Scheme == Uri.UriSchemeHttps
            && string.Equals(abs.Host, baseUri.Host, StringComparison.OrdinalIgnoreCase))
        {
            return abs.AbsoluteUri;
        }

        return string.Empty;
    }

    private static string StableHash(string input)
    {
        unchecked
        {
            uint hash = 2166136261u;
            for (int i = 0; i < input.Length; i++)
            {
                hash ^= input[i];
                hash *= 16777619u;
            }
            return hash.ToString("x8", CultureInfo.InvariantCulture);
        }
    }
}

internal sealed class WsBroadcastMessage
{
    [JsonProperty("action")] public string Action { get; set; } = string.Empty;
    [JsonProperty("event")] public CalendarEvent Event { get; set; }
    [JsonProperty("eventId")] public string EventId { get; set; } = string.Empty;
}
