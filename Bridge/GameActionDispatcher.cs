using System;
using System.Collections.Generic;
using LumenicMenu.Bridge.Actions;
using MelonLoader;
using Newtonsoft.Json.Linq;

namespace LumenicMenu.Bridge;

/// <summary>
/// Drains and dispatches typed JSON actions emitted by the Rust Lumenic UI on Unity's main thread
/// through domain-scoped handler tables.
/// </summary>
internal static class GameActionDispatcher
{
    private static readonly Dictionary<string, Action<JObject, LumenicMenuMod>> Handlers = BuildHandlers();

    private static Dictionary<string, Action<JObject, LumenicMenuMod>> BuildHandlers()
    {
        var map = new Dictionary<string, Action<JObject, LumenicMenuMod>>(160, StringComparer.Ordinal);
        CoreActions.Register(map);
        UiLibUixExtActions.Register(map);
        ContentActions.Register(map);
        SocialActions.Register(map);
        SettingsAvatarActions.Register(map);
        return map;
    }

    public static void DispatchActionsJson(string actionsJson, LumenicMenuMod mod)
    {
        if (string.IsNullOrEmpty(actionsJson) || mod == null)
            return;

        JArray array;
        try
        {
            array = JArray.Parse(actionsJson);
        }
        catch (Exception ex)
        {
            MelonLogger.Error($"[LumenicMenu] Failed to parse Rust actions JSON: {ex.Message}");
            return;
        }

        foreach (JToken token in array)
        {
            if (token is not JObject obj)
                continue;

            string type = obj.Value<string>("type");
            if (string.IsNullOrEmpty(type))
                continue;

            if (!Handlers.TryGetValue(type, out Action<JObject, LumenicMenuMod> handler))
            {
                MelonLogger.Warning($"[LumenicMenu] Unhandled UI action type: '{type}'.");
                continue;
            }

            try
            {
                handler(obj, mod);
            }
            catch (Exception ex)
            {
                MelonLogger.Error($"[LumenicMenu] Action '{type}' failed: {ex.Message}");
            }
        }
    }
}
