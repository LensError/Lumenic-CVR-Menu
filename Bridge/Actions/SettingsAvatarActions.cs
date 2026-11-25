using System;
using System.Collections.Generic;
using Newtonsoft.Json.Linq;

namespace LumenicMenu.Bridge.Actions;

internal static class SettingsAvatarActions
{
    public static void Register(Dictionary<string, Action<JObject, LumenicMenuMod>> map)
    {
        // CVR Settings
        map["CvrSetBool"] = (obj, mod) =>
        {
            SettingsBridge.SetBool(obj.Value<string>("key"), obj.Value<bool>("value"));
            mod.MarkStateDirty(StateDomain.Settings | StateDomain.Core);
        };
        map["CvrSetInt"] = (obj, mod) =>
        {
            SettingsBridge.SetInt(obj.Value<string>("key"), obj.Value<int>("value"));
            mod.MarkStateDirty(StateDomain.Settings | StateDomain.Core);
        };
        map["CvrSetFloat"] = (obj, mod) =>
        {
            SettingsBridge.SetFloat(obj.Value<string>("key"), obj.Value<float>("value"));
            mod.MarkStateDirty(StateDomain.Settings | StateDomain.Core);
        };
        map["CvrSetString"] = (obj, mod) =>
        {
            SettingsBridge.SetString(obj.Value<string>("key"), obj.Value<string>("value"));
            mod.MarkStateDirty(StateDomain.Settings | StateDomain.Core);
        };

        // Advanced Avatar Settings & 3D Previews
        map["AvatarSetParam"] = (obj, mod) =>
        {
            AvatarSettingsBridge.SetParam(obj.Value<string>("param") ?? string.Empty, obj.Value<float>("value"));
            mod.MarkStateDirty(StateDomain.AvatarSettings);
        };
        map["AvatarSetAutoApply"] = (obj, mod) => { AvatarSettingsBridge.SetAutoApply(obj.Value<bool>("enabled")); mod.MarkStateDirty(StateDomain.AvatarSettings); };
        map["AvatarApplyStaged"] = (_, mod) => { AvatarSettingsBridge.ApplyStaged(); mod.MarkStateDirty(StateDomain.AvatarSettings); };
        map["AvatarDiscardStaged"] = (_, mod) => { AvatarSettingsBridge.DiscardStaged(); mod.MarkStateDirty(StateDomain.AvatarSettings); };
        map["AvatarLoadProfile"] = (obj, mod) => { AvatarSettingsBridge.LoadProfile(obj.Value<string>("profile") ?? string.Empty); mod.MarkStateDirty(StateDomain.AvatarSettings); };
        map["AvatarSaveProfile"] = (obj, mod) => { AvatarSettingsBridge.SaveProfile(obj.Value<string>("profile") ?? string.Empty); mod.MarkStateDirty(StateDomain.AvatarSettings); };
        map["AvatarDeleteProfile"] = (obj, mod) => { AvatarSettingsBridge.DeleteProfile(obj.Value<string>("profile") ?? string.Empty); mod.MarkStateDirty(StateDomain.AvatarSettings); };
        map["AvatarReorderProfiles"] = (obj, mod) =>
        {
            string[] ordered = obj["profiles"]?.ToObject<string[]>();
            AvatarSettingsBridge.ReorderProfiles(ordered);
            mod.MarkStateDirty(StateDomain.AvatarSettings);
        };
        map["AvatarResetDefaults"] = (_, mod) => { AvatarSettingsBridge.ResetDefaults(); mod.MarkStateDirty(StateDomain.AvatarSettings); };
        map["AvatarSetHeight"] = (obj, mod) => { AvatarSettingsBridge.SetAvatarHeight(obj.Value<float>("height")); mod.MarkStateDirty(StateDomain.AvatarSettings); };
        map["AvatarResetHeight"] = (_, mod) => { AvatarSettingsBridge.ResetAvatarHeight(); mod.MarkStateDirty(StateDomain.AvatarSettings); };
        map["AvatarPreviewRotate"] = (obj, mod) => mod.RotateActiveAvatarPreview(obj.Value<float>("delta"), obj.Value<bool>("is_dragging"));
        map["WorldPanoLook"] = (obj, _) => AvatarSettingsBridge.LookWorldPano(
            obj.Value<float>("delta_x"),
            obj.Value<float>("delta_y"),
            obj.Value<bool>("is_dragging"));
        map["SyncAvatarPreview"] = (obj, mod) => mod.SyncAvatarPreview(
            obj.Value<uint>("surface_id"),
            obj.Value<bool>("active"),
            obj.Value<string>("target") ?? "local",
            obj.Value<float>("x"),
            obj.Value<float>("y"),
            obj.Value<float>("w"),
            obj.Value<float>("h"));
        map["SyncStudioStage"] = (_, _) => { };
    }
}
