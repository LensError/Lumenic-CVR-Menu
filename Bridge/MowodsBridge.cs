using System;
using System.Collections.Generic;
using System.Globalization;
using System.Reflection;
using MelonLoader;
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;

namespace LumenicMenu.Bridge;

/// <summary>
/// Zero-hard-dependency reflection bridge for Mowods ("Mowods.dll" by NotAKidoS).
/// Reads Mowods' unified Pages/Sections/Items (both custom ModsMenu API pages and MelonIngest pages)
/// and forwards Set/Reset/Click/OpenLink actions through NAK.Mowods.Menu.
/// </summary>
internal sealed class MowodsBridge
{
    private bool _probed;
    private bool _available;
    private MethodInfo _getPagesMethod;
    private MethodInfo _getPageMethod;
    private MethodInfo _setMethod;
    private MethodInfo _resetMethod;
    private MethodInfo _clickMethod;
    private MethodInfo _openLinkMethod;

    public bool IsAvailable
    {
        get
        {
            EnsureProbed();
            return _available;
        }
    }

    public void EnsureProbed()
    {
        if (_probed)
            return;
        _probed = true;

        try
        {
            const BindingFlags flags = BindingFlags.Static | BindingFlags.NonPublic | BindingFlags.Public;
            foreach (Assembly asm in AppDomain.CurrentDomain.GetAssemblies())
            {
                Type menuType = asm.GetType("NAK.Mowods.Menu", throwOnError: false);
                if (menuType == null)
                    continue;

                MethodInfo getPages = menuType.GetMethod("GetPages", flags, null, Type.EmptyTypes, null);
                MethodInfo getPage = menuType.GetMethod("GetPage", flags, null, new[] { typeof(int) }, null);
                MethodInfo set = menuType.GetMethod("Set", flags, null, new[] { typeof(int), typeof(string) }, null);
                MethodInfo reset = menuType.GetMethod("Reset", flags, null, new[] { typeof(int) }, null);
                MethodInfo click = menuType.GetMethod("Click", flags, null, new[] { typeof(int) }, null);
                MethodInfo openLink = menuType.GetMethod("OpenLink", flags, null, new[] { typeof(int) }, null);

                if (getPages == null || getPage == null || set == null)
                    continue;

                _getPagesMethod = getPages;
                _getPageMethod = getPage;
                _setMethod = set;
                _resetMethod = reset;
                _clickMethod = click;
                _openLinkMethod = openLink;
                _available = true;
                MelonLogger.Msg("[LumenicMenu] Detected Mowods - native Mod Settings bridge enabled.");
                break;
            }
        }
        catch (Exception ex)
        {
            MelonLogger.Warning($"[LumenicMenu] Mowods probe failed: {ex.Message}");
            _available = false;
        }
    }

    public List<object> BuildCategoriesSnapshot()
    {
        var result = new List<object>();
        if (!IsAvailable)
            return result;

        try
        {
            string pagesJson = _getPagesMethod.Invoke(null, null) as string;
            if (string.IsNullOrWhiteSpace(pagesJson))
                return result;

            JArray pagesArr = JArray.Parse(pagesJson);
            foreach (JToken pageTok in pagesArr)
            {
                if (pageTok is not JObject pageStub)
                    continue;

                int pageId = pageStub.Value<int?>("id") ?? -1;
                if (pageId < 0)
                    continue;

                string pageJson = _getPageMethod.Invoke(null, new object[] { pageId }) as string;
                if (string.IsNullOrWhiteSpace(pageJson))
                    continue;

                JObject pageObj = JObject.Parse(pageJson);
                string pageName = pageObj.Value<string>("name") ?? pageStub.Value<string>("name") ?? string.Empty;
                string info = pageObj.Value<string>("info") ?? string.Empty;
                bool hasLink = pageObj.Value<bool?>("hasLink") ?? false;

                var colors = new List<string>();
                if (pageObj["colors"] is JArray colorArr)
                {
                    foreach (JToken c in colorArr)
                    {
                        string hex = c?.ToString();
                        if (!string.IsNullOrWhiteSpace(hex))
                            colors.Add(hex);
                    }
                }

                var entriesList = new List<object>();
                if (pageObj["sections"] is JArray sectionsArr)
                {
                    foreach (JToken secTok in sectionsArr)
                    {
                        if (secTok is not JObject secObj)
                            continue;

                        int secId = secObj.Value<int?>("id") ?? -1;
                        string secName = secObj.Value<string>("name") ?? string.Empty;
                        string secDesc = secObj.Value<string>("description") ?? string.Empty;

                        if (secObj["items"] is not JArray itemsArr)
                            continue;

                        foreach (JToken itemTok in itemsArr)
                        {
                            if (itemTok is not JObject itemObj)
                                continue;

                            object dto = BuildItemDto(secId, secName, secDesc, itemObj);
                            if (dto != null)
                                entriesList.Add(dto);
                        }
                    }
                }

                if (entriesList.Count == 0)
                    continue;

                result.Add(new
                {
                    id = $"mowods:{pageId}",
                    display_name = pageName,
                    node_id = pageId,
                    info,
                    colors,
                    has_link = hasLink,
                    entries = entriesList,
                    custom_controls = Array.Empty<object>()
                });
            }
        }
        catch (Exception ex)
        {
            MelonLogger.Warning($"[LumenicMenu] Failed to build Mowods snapshot: {ex.Message}");
        }

        return result;
    }

    private static object BuildItemDto(int secId, string secName, string secDesc, JObject item)
    {
        int itemId = item.Value<int?>("id") ?? -1;
        if (itemId < 0)
            return null;

        string rawKind = (item.Value<string>("kind") ?? string.Empty).ToLowerInvariant();
        string name = item.Value<string>("name") ?? string.Empty;
        string tooltip = item.Value<string>("tooltip") ?? string.Empty;
        bool isDefault = item.Value<bool?>("isDefault") ?? true;

        switch (rawKind)
        {
            case "toggle":
                return new
                {
                    id = itemId.ToString(CultureInfo.InvariantCulture),
                    node_id = itemId,
                    section_id = secId,
                    section_name = secName,
                    section_desc = secDesc,
                    display_name = name,
                    description = tooltip,
                    kind = "bool",
                    bool_val = item.Value<bool?>("value") ?? false,
                    is_default = isDefault,
                    can_reset = true
                };

            case "slider":
                return new
                {
                    id = itemId.ToString(CultureInfo.InvariantCulture),
                    node_id = itemId,
                    section_id = secId,
                    section_name = secName,
                    section_desc = secDesc,
                    display_name = name,
                    description = tooltip,
                    kind = "slider",
                    num_val = item.Value<double?>("value") ?? 0.0,
                    min_val = item.Value<double?>("min") ?? 0.0,
                    max_val = item.Value<double?>("max") ?? 1.0,
                    snap_val = item.Value<double?>("step") ?? 0.0,
                    is_int = item.Value<bool?>("integer") ?? false,
                    is_default = isDefault,
                    can_reset = true
                };

            case "number":
                return new
                {
                    id = itemId.ToString(CultureInfo.InvariantCulture),
                    node_id = itemId,
                    section_id = secId,
                    section_name = secName,
                    section_desc = secDesc,
                    display_name = name,
                    description = tooltip,
                    kind = "number",
                    num_val = item.Value<double?>("value") ?? 0.0,
                    is_int = item.Value<bool?>("integer") ?? false,
                    is_default = isDefault,
                    can_reset = true
                };

            case "text":
                return new
                {
                    id = itemId.ToString(CultureInfo.InvariantCulture),
                    node_id = itemId,
                    section_id = secId,
                    section_name = secName,
                    section_desc = secDesc,
                    display_name = name,
                    description = tooltip,
                    kind = "string",
                    string_val = item.Value<string>("value") ?? string.Empty,
                    is_default = isDefault,
                    can_reset = true
                };

            case "choice":
            {
                var options = new List<string>();
                if (item["options"] is JArray optArr)
                {
                    foreach (JToken o in optArr)
                        options.Add(o?.ToString() ?? string.Empty);
                }

                return new
                {
                    id = itemId.ToString(CultureInfo.InvariantCulture),
                    node_id = itemId,
                    section_id = secId,
                    section_name = secName,
                    section_desc = secDesc,
                    display_name = name,
                    description = tooltip,
                    kind = "enum",
                    enum_options = options,
                    enum_index = item.Value<int?>("value") ?? 0,
                    use_buttons = item.Value<bool?>("buttons") ?? (options.Count <= 4),
                    is_default = isDefault,
                    can_reset = true
                };
            }

            case "button":
                return new
                {
                    id = itemId.ToString(CultureInfo.InvariantCulture),
                    node_id = itemId,
                    section_id = secId,
                    section_name = secName,
                    section_desc = secDesc,
                    display_name = name,
                    description = tooltip,
                    kind = "button",
                    button_label = item.Value<string>("label") ?? name,
                    is_default = true,
                    can_reset = false
                };

            default:
                return null;
        }
    }

    public void SetBool(string entryId, bool value)
    {
        if (!IsAvailable || !int.TryParse(entryId, NumberStyles.Integer, CultureInfo.InvariantCulture, out int id))
            return;
        InvokeSet(id, value ? "true" : "false");
    }

    public void SetNumber(string entryId, double value)
    {
        if (!IsAvailable || double.IsNaN(value) || double.IsInfinity(value))
            return;
        if (!int.TryParse(entryId, NumberStyles.Integer, CultureInfo.InvariantCulture, out int id))
            return;
        InvokeSet(id, value.ToString("R", CultureInfo.InvariantCulture));
    }

    public void SetString(string entryId, string value)
    {
        if (!IsAvailable || !int.TryParse(entryId, NumberStyles.Integer, CultureInfo.InvariantCulture, out int id))
            return;
        InvokeSet(id, JsonConvert.ToString(value ?? string.Empty));
    }

    public void SetEnum(string entryId, int index)
    {
        if (!IsAvailable || !int.TryParse(entryId, NumberStyles.Integer, CultureInfo.InvariantCulture, out int id))
            return;
        InvokeSet(id, index.ToString(CultureInfo.InvariantCulture));
    }

    public void ResetNode(int nodeId)
    {
        if (!IsAvailable || _resetMethod == null || nodeId < 0)
            return;
        try
        {
            _resetMethod.Invoke(null, new object[] { nodeId });
        }
        catch (Exception ex)
        {
            MelonLogger.Warning($"[LumenicMenu] Mowods Reset({nodeId}) failed: {ex.Message}");
        }
    }

    public void ClickButton(int nodeId)
    {
        if (!IsAvailable || _clickMethod == null || nodeId < 0)
            return;
        try
        {
            _clickMethod.Invoke(null, new object[] { nodeId });
        }
        catch (Exception ex)
        {
            MelonLogger.Warning($"[LumenicMenu] Mowods Click({nodeId}) failed: {ex.Message}");
        }
    }

    public void OpenLink(int pageId)
    {
        if (!IsAvailable || _openLinkMethod == null || pageId < 0)
            return;
        try
        {
            _openLinkMethod.Invoke(null, new object[] { pageId });
        }
        catch (Exception ex)
        {
            MelonLogger.Warning($"[LumenicMenu] Mowods OpenLink({pageId}) failed: {ex.Message}");
        }
    }

    private void InvokeSet(int id, string json)
    {
        try
        {
            _setMethod.Invoke(null, new object[] { id, json });
        }
        catch (Exception ex)
        {
            MelonLogger.Warning($"[LumenicMenu] Mowods Set({id}) failed: {ex.Message}");
        }
    }
}
