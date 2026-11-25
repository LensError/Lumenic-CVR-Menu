using System;
using System.Collections.Generic;
using Newtonsoft.Json.Linq;

namespace LumenicMenu.Bridge.Actions;

internal static class UiLibUixExtActions
{
    public static void Register(Dictionary<string, Action<JObject, LumenicMenuMod>> map)
    {
        // Extension API
        map["ExtSelectTab"] = (obj, _) =>
        {
            string surf = obj.Value<string>("surface") ?? "quick";
            string tabId = obj.Value<string>("tab_id") ?? string.Empty;
            Api.LumenicApi.Host.SelectTab(surf == "main" ? Api.ExtSurface.MainMenu : Api.ExtSurface.QuickMenu, tabId);
        };
        map["ExtGoBack"] = (obj, _) =>
        {
            string surf = obj.Value<string>("surface") ?? "quick";
            Api.LumenicApi.Host.PopSubPage(surf == "main" ? Api.ExtSurface.MainMenu : Api.ExtSurface.QuickMenu);
        };
        map["ExtCloseModal"] = (obj, _) =>
        {
            string surf = obj.Value<string>("surface") ?? "quick";
            Api.LumenicApi.Host.CloseModal(surf == "main" ? Api.ExtSurface.MainMenu : Api.ExtSurface.QuickMenu);
        };
        map["ExtEvent"] = (obj, _) => Api.LumenicApi.Host.DispatchEvent(
            obj.Value<string>("ext_id") ?? string.Empty,
            obj.Value<string>("target_id") ?? string.Empty,
            obj.Value<string>("node_id") ?? string.Empty,
            obj.Value<string>("kind") ?? string.Empty,
            obj.Value<string>("string_val") ?? string.Empty,
            obj.Value<float>("float_val"),
            obj.Value<bool>("bool_val"));

        // UILib Mod Compatibility
        map["UiLibSelectTab"] = (obj, mod) =>
        {
            Api.LumenicApi.Host.ClearActiveTab(Api.ExtSurface.QuickMenu);
            mod.UiLib.SelectTab(obj.Value<string>("tab_id"));
            mod.MarkStateDirty(StateDomain.UiLib);
        };
        map["UiLibGoBack"] = (_, mod) => { mod.UiLib.HandleGoBack(); mod.MarkStateDirty(StateDomain.UiLib); };
        map["UiLibToggleCategory"] = (obj, mod) => { mod.UiLib.ToggleCategory(obj.Value<string>("category_id")); mod.MarkStateDirty(StateDomain.UiLib); };
        map["UiLibClick"] = (obj, mod) => { mod.UiLib.ClickElement(obj.Value<string>("element_id")); mod.MarkStateDirty(StateDomain.UiLib); };
        map["UiLibHold"] = (obj, mod) => { mod.UiLib.HoldElement(obj.Value<string>("element_id")); mod.MarkStateDirty(StateDomain.UiLib); };
        map["UiLibToggle"] = (obj, mod) => { mod.UiLib.ToggleElement(obj.Value<string>("element_id"), obj.Value<bool>("value")); mod.MarkStateDirty(StateDomain.UiLib); };
        map["UiLibSlider"] = (obj, mod) =>
        {
            mod.UiLib.SetSlider(
                obj.Value<string>("element_id"),
                obj.Value<float>("value"),
                obj.Value<bool>("reset"));
            mod.MarkStateDirty(StateDomain.UiLib);
        };
        map["UiLibTextInput"] = (obj, mod) => { mod.UiLib.OpenTextInput(obj.Value<string>("element_id")); mod.MarkStateDirty(StateDomain.UiLib); };
        map["UiLibModalConfirm"] = (obj, mod) => { mod.UiLib.ModalConfirm(obj.Value<bool>("accepted")); mod.MarkStateDirty(StateDomain.UiLib); };
        map["UiLibModalNoticeOk"] = (_, mod) => { mod.UiLib.ModalNoticeOk(); mod.MarkStateDirty(StateDomain.UiLib); };
        map["UiLibModalSelectOption"] = (obj, mod) => { mod.UiLib.ModalSelectOption(obj.Value<int>("index")); mod.MarkStateDirty(StateDomain.UiLib); };
        map["UiLibModalNumberSubmit"] = (obj, mod) => { mod.UiLib.ModalNumberSubmit(obj.Value<float>("value")); mod.MarkStateDirty(StateDomain.UiLib); };
        map["UiLibModalDismiss"] = (_, mod) => { mod.UiLib.ModalDismiss(); mod.MarkStateDirty(StateDomain.UiLib); };

        // UIExpansionKit Mod Settings
        map["RefreshUixSettings"] = (_, mod) => { mod.Uix?.EnsureProbed(); mod.MarkStateDirty(StateDomain.Uix); };
        map["UixSetBool"] = (obj, mod) =>
        {
            mod.Uix?.SetBool(obj.Value<string>("category"), obj.Value<string>("entry"), obj.Value<bool>("value"));
            mod.MarkStateDirty(StateDomain.Uix);
        };
        map["UixSetNumber"] = (obj, mod) =>
        {
            mod.Uix?.SetNumber(
                obj.Value<string>("category"),
                obj.Value<string>("entry"),
                obj.Value<double>("value"),
                obj.Value<bool>("save"));
            mod.MarkStateDirty(StateDomain.Uix);
        };
        map["UixSetString"] = (obj, mod) =>
        {
            mod.Uix?.SetString(obj.Value<string>("category"), obj.Value<string>("entry"), obj.Value<string>("value"));
            mod.MarkStateDirty(StateDomain.Uix);
        };
        map["UixSetEnum"] = (obj, mod) =>
        {
            mod.Uix?.SetEnum(obj.Value<string>("category"), obj.Value<string>("entry"), obj.Value<int>("index"));
            mod.MarkStateDirty(StateDomain.Uix);
        };
        map["UixCustomButton"] = (obj, mod) =>
        {
            mod.Uix?.ClickCustomButton(obj.Value<string>("category"), obj.Value<int>("global_id"));
            mod.MarkStateDirty(StateDomain.Uix);
        };
        map["UixCustomToggle"] = (obj, mod) =>
        {
            mod.Uix?.ToggleCustomToggle(obj.Value<string>("category"), obj.Value<int>("global_id"), obj.Value<bool>("value"));
            mod.MarkStateDirty(StateDomain.Uix);
        };
    }
}
