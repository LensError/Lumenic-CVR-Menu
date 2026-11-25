using System;
using System.Collections.Concurrent;
using MelonLoader;

namespace LumenicMenu.Bridge;

/// <summary>
/// Logs diagnostic warnings at most once per call-site key when <see cref="Settings.PerfLogging"/>
/// is enabled, preventing silent exception swallowing without spamming logs.
/// </summary>
internal static class DiagLog
{
    private static readonly ConcurrentDictionary<string, byte> LoggedKeys =
        new ConcurrentDictionary<string, byte>(StringComparer.Ordinal);

    public static void WarnOnce(string key, Exception ex)
    {
        if (!Settings.PerfLogging || ex == null)
            return;

        if (LoggedKeys.TryAdd(key, 0))
        {
            MelonLogger.Warning($"[LumenicMenu][Diag][{key}] {ex.GetType().Name}: {ex.Message}");
        }
    }
}
