using System;
using System.Collections.Generic;
using ABI_RC.Core.InteractionSystem;
using ABI_RC.Core.Networking.IO.Instancing;
using Newtonsoft.Json.Linq;

namespace LumenicMenu.Bridge.Actions;

internal static class ContentActions
{
    public static void Register(Dictionary<string, Action<JObject, LumenicMenuMod>> map)
    {
        map["ContentLoadList"] = (obj, mod) => mod.Content?.LoadList(
            obj.Value<string>("kind") ?? "worlds",
            obj.Value<string>("category_key") ?? string.Empty,
            obj.Value<bool>("is_system"),
            obj.Value<int>("page"),
            obj.Value<int>("sort_order"),
            obj.Value<bool>("ascending"));

        map["ContentLoadMore"] = (obj, mod) => mod.Content?.LoadMore(obj.Value<string>("kind") ?? "worlds");
        map["ContentLoadDetail"] = (obj, mod) => mod.Content?.LoadDetail(
            obj.Value<string>("kind") ?? "world",
            obj.Value<string>("id") ?? string.Empty);
        map["ContentCloseDetail"] = (_, mod) => mod.Content?.CloseDetail();
        map["ContentGlobalSearch"] = (obj, mod) => mod.Content?.RunGlobalSearch(obj.Value<string>("query") ?? string.Empty);
        map["ContentWearAvatar"] = (obj, mod) => mod.Content?.WearAvatar(obj.Value<string>("avatar_id") ?? string.Empty);
        map["ContentSpawnProp"] = (obj, mod) => mod.Content?.SpawnProp(obj.Value<string>("prop_id") ?? string.Empty);
        map["ContentJoinInstance"] = (obj, mod) => mod.Content?.JoinInstance(obj.Value<string>("instance_id") ?? string.Empty);
        map["ContentDropPortal"] = (obj, mod) => mod.Content?.DropPortal(obj.Value<string>("instance_id") ?? string.Empty);
        map["ContentDropCurrentInstancePortal"] = (_, mod) => mod.Content?.DropPortal(Instances.CurrentInstanceId ?? string.Empty);

        map["ContentLoadInstanceDetail"] = (obj, mod) =>
        {
            string instId = obj.Value<string>("instance_id") ?? string.Empty;
            if (instId == "__current__")
                instId = Instances.CurrentInstanceId ?? string.Empty;
            mod.Content?.LoadInstanceDetail(instId);
        };
        map["ContentCloseInstanceDetail"] = (_, mod) => mod.Content?.CloseInstanceDetail();

        map["ContentToggleCategory"] = (obj, mod) => mod.Content?.ToggleContentCategory(
            obj.Value<string>("kind") ?? string.Empty,
            obj.Value<string>("id") ?? string.Empty,
            obj.Value<string>("category_key") ?? string.Empty);

        map["ContentCreateInstance"] = (obj, mod) => mod.Content?.CreateInstance(
            obj.Value<string>("world_id") ?? string.Empty,
            obj.Value<string>("privacy") ?? "Friends",
            obj.Value<string>("region") ?? "Eu",
            obj.Value<string>("group_id") ?? string.Empty);

        map["ContentResolveGroupModPrompt"] = (obj, mod) => mod.Content?.ResolveGroupModPrompt(
            obj.Value<bool?>("accept") ?? false,
            obj.Value<bool?>("never_show") ?? false);

        map["ContentSetHomeWorld"] = (obj, mod) => mod.Content?.SetHomeWorld(obj.Value<string>("world_id") ?? string.Empty);
        map["ContentTogglePin"] = (obj, mod) => mod.Content?.TogglePin(
            obj.Value<string>("kind") ?? string.Empty,
            obj.Value<string>("id") ?? string.Empty);
        map["ContentUnpin"] = (obj, mod) => mod.Content?.Unpin(
            obj.Value<string>("kind") ?? string.Empty,
            obj.Value<string>("id") ?? string.Empty);

        map["ContentLoadDiscover"] = (_, mod) => mod.Content?.LoadDiscover();
        map["ContentRefreshIfStale"] = (obj, mod) => mod.Content?.RefreshIfStale(obj.Value<string>("kind") ?? "worlds");

        map["QmOpenWorldDetails"] = (obj, mod) =>
        {
            string worldId = obj.Value<string>("world_id") ?? string.Empty;
            CVR_MenuManager.Instance?.ToggleQuickMenu(show: false);
            if (!string.IsNullOrEmpty(worldId))
            {
                mod.Content?.LoadDetail("world", worldId);
                mod.NavigateMainMenuTab("worlds");
                if (ViewManager.Instance != null)
                {
                    mod.SuppressVanillaNavHook = true;
                    try
                    {
                        ViewManager.Instance.UiStateToggle(show: true);
                    }
                    finally
                    {
                        mod.SuppressVanillaNavHook = false;
                    }
                }
            }
        };
    }
}
