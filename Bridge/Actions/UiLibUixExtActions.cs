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

        // Mod Settings (Mowods / UIExpansionKit)
        map["RefreshUixSettings"] = (_, mod) =>
        {
            mod.Mowods?.EnsureProbed();
            mod.Uix?.EnsureProbed();
            mod.MarkStateDirty(StateDomain.Uix);
        };
        map["UixSetBool"] = (obj, mod) =>
        {
            string cat = obj.Value<string>("category") ?? string.Empty;
            string entry = obj.Value<string>("entry") ?? string.Empty;
            bool val = obj.Value<bool>("value");
            if (cat.StartsWith("mowods:", StringComparison.Ordinal))
                mod.Mowods?.SetBool(entry, val);
            else
                mod.Uix?.SetBool(cat, entry, val);
            mod.MarkStateDirty(StateDomain.Uix);
        };
        map["UixSetNumber"] = (obj, mod) =>
        {
            string cat = obj.Value<string>("category") ?? string.Empty;
            string entry = obj.Value<string>("entry") ?? string.Empty;
            double val = obj.Value<double>("value");
            if (cat.StartsWith("mowods:", StringComparison.Ordinal))
                mod.Mowods?.SetNumber(entry, val);
            else
                mod.Uix?.SetNumber(cat, entry, val, obj.Value<bool>("save"));
            mod.MarkStateDirty(StateDomain.Uix);
        };
        map["UixSetString"] = (obj, mod) =>
        {
            string cat = obj.Value<string>("category") ?? string.Empty;
            string entry = obj.Value<string>("entry") ?? string.Empty;
            string val = obj.Value<string>("value");
            if (cat.StartsWith("mowods:", StringComparison.Ordinal))
                mod.Mowods?.SetString(entry, val);
            else
                mod.Uix?.SetString(cat, entry, val);
            mod.MarkStateDirty(StateDomain.Uix);
        };
        map["UixSetEnum"] = (obj, mod) =>
        {
            string cat = obj.Value<string>("category") ?? string.Empty;
            string entry = obj.Value<string>("entry") ?? string.Empty;
            int idx = obj.Value<int>("index");
            if (cat.StartsWith("mowods:", StringComparison.Ordinal))
                mod.Mowods?.SetEnum(entry, idx);
            else
                mod.Uix?.SetEnum(cat, entry, idx);
            mod.MarkStateDirty(StateDomain.Uix);
        };
        map["UixCustomButton"] = (obj, mod) =>
        {
            string cat = obj.Value<string>("category") ?? string.Empty;
            int id = obj.Value<int>("global_id");
            if (cat.StartsWith("mowods:", StringComparison.Ordinal))
                mod.Mowods?.ClickButton(id);
            else
                mod.Uix?.ClickCustomButton(cat, id);
            mod.MarkStateDirty(StateDomain.Uix);
        };
        map["UixCustomToggle"] = (obj, mod) =>
        {
            mod.Uix?.ToggleCustomToggle(obj.Value<string>("category"), obj.Value<int>("global_id"), obj.Value<bool>("value"));
            mod.MarkStateDirty(StateDomain.Uix);
        };
        map["MowodsResetNode"] = (obj, mod) =>
        {
            mod.Mowods?.ResetNode(obj.Value<int>("node_id"));
            mod.MarkStateDirty(StateDomain.Uix);
        };
        map["MowodsOpenLink"] = (obj, mod) =>
        {
            mod.Mowods?.OpenLink(obj.Value<int>("page_id"));
        };
        map["SetModSettingsSource"] = (obj, mod) =>
        {
            string src = obj.Value<string>("source") ?? "mowods";
            mod.PreferredModSettingsSource = string.Equals(src, "uix", StringComparison.OrdinalIgnoreCase) ? "uix" : "mowods";
            mod.MarkStateDirty(StateDomain.Uix);
        };
    }
}
