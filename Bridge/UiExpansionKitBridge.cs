using System;
using System.Collections;
using System.Collections.Generic;
using System.ComponentModel;
using System.Globalization;
using System.Linq;
using System.Reflection;
using HarmonyLib;
using MelonLoader;

namespace LumenicMenu.Bridge;

/// <summary>
/// Zero-hard-dependency reflection bridge for UIExpansionKit ("UI Expansion Kit.dll").
/// Mirrors MelonPreferences categories, UIX ranges/visibilities, and GetSettingsCategory custom controls
/// into LumenicMenu's native Main Menu Mod Settings screen.
/// </summary>
internal sealed class UiExpansionKitBridge : IDisposable
{
    private readonly Action _onDirty;

    private bool _probed;
    private bool _available;
    private Type _apiType;
    private FieldInfo _settingPageExtensionsField;
    private FieldInfo _settingsVisibilitiesField;
    private FieldInfo _settingsRangesField;
    private FieldInfo _settingsVisibilityUpdatedField;
    private Delegate _visibilityHook;

    private readonly Dictionary<int, bool> _customToggleFallbackState = new Dictionary<int, bool>();

    public UiExpansionKitBridge(Action onDirty)
    {
        _onDirty = onDirty;
    }

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
            foreach (Assembly asm in AppDomain.CurrentDomain.GetAssemblies())
            {
                Type api = asm.GetType("UIExpansionKit.API.ExpansionKitApi", throwOnError: false);
                if (api == null)
                    continue;

                _apiType = api;
                _settingPageExtensionsField = AccessTools.Field(api, "SettingPageExtensions");
                _settingsVisibilitiesField = AccessTools.Field(api, "SettingsVisibilities");
                _settingsRangesField = AccessTools.Field(api, "SettingsRanges");
                _settingsVisibilityUpdatedField = AccessTools.Field(api, "SettingsVisibilityUpdated");
                _available = true;
                HookVisibilityUpdates();
                MelonLogger.Msg("[LumenicMenu] Detected UIExpansionKit - native Mod Settings bridge enabled.");
                break;
            }
        }
        catch (Exception ex)
        {
            MelonLogger.Warning($"[LumenicMenu] UIExpansionKit probe failed: {ex.Message}");
            _available = false;
        }
    }

    private void HookVisibilityUpdates()
    {
        if (_settingsVisibilityUpdatedField == null)
            return;

        try
        {
            Type regType = _apiType.GetNestedType("SettingVisibilityRegistrationValue", BindingFlags.NonPublic | BindingFlags.Public);
            if (regType == null)
                return;

            MethodInfo cb = typeof(UiExpansionKitBridge).GetMethod(
                nameof(OnVisibilityUpdatedCallback),
                BindingFlags.Instance | BindingFlags.NonPublic);
            if (cb == null)
                return;

            Type delegateType = typeof(Action<,,>).MakeGenericType(typeof(string), typeof(string), regType);
            _visibilityHook = Delegate.CreateDelegate(delegateType, this, cb.MakeGenericMethod(regType));
            Delegate existing = _settingsVisibilityUpdatedField.GetValue(null) as Delegate;
            _settingsVisibilityUpdatedField.SetValue(null, Delegate.Combine(existing, _visibilityHook));
        }
        catch
        {
            // Visibility callbacks still evaluate fresh on every snapshot push.
        }
    }

    public void Dispose()
    {
        if (_visibilityHook != null && _settingsVisibilityUpdatedField != null)
        {
            try
            {
                Delegate existing = _settingsVisibilityUpdatedField.GetValue(null) as Delegate;
                _settingsVisibilityUpdatedField.SetValue(null, Delegate.Remove(existing, _visibilityHook));
            }
            catch
            {
                // Ignore if UIX already tore down.
            }
            _visibilityHook = null;
        }
        _customToggleFallbackState.Clear();
        _pendingSaveCategories.Clear();
    }

    private void OnVisibilityUpdatedCallback<TReg>(string category, string entry, TReg visibility)
    {
        _onDirty?.Invoke();
    }

    public List<object> BuildCategoriesSnapshot()
    {
        var result = new List<object>();
        if (!IsAvailable)
            return result;

        try
        {
            IDictionary extensions = _settingPageExtensionsField?.GetValue(null) as IDictionary;
            IDictionary visibilities = _settingsVisibilitiesField?.GetValue(null) as IDictionary;
            IDictionary ranges = _settingsRangesField?.GetValue(null) as IDictionary;

            var categories = MelonPreferences.Categories
                .Where(c => c != null && !c.IsHidden &&
                    (c.Entries.Any(EntryCanBeShown) || (extensions != null && extensions.Contains(c.Identifier))))
                .OrderBy(c => string.IsNullOrWhiteSpace(c.DisplayName) ? c.Identifier : c.DisplayName, StringComparer.OrdinalIgnoreCase)
                .ToList();

            foreach (MelonPreferences_Category cat in categories)
            {
                var entriesList = new List<object>();
                foreach (MelonPreferences_Entry entry in cat.Entries)
                {
                    if (!EntryCanBeShown(entry))
                        continue;

                    if (!EvaluateVisibility(visibilities, cat.Identifier, entry.Identifier))
                        continue;

                    object entryDto = BuildEntryDto(cat.Identifier, entry, ranges);
                    if (entryDto != null)
                        entriesList.Add(entryDto);
                }

                var customList = BuildCustomControls(extensions, cat.Identifier);
                if (entriesList.Count == 0 && customList.Count == 0)
                    continue;

                result.Add(new
                {
                    id = cat.Identifier ?? string.Empty,
                    display_name = string.IsNullOrWhiteSpace(cat.DisplayName) ? (cat.Identifier ?? string.Empty) : cat.DisplayName,
                    entries = entriesList,
                    custom_controls = customList
                });
            }
        }
        catch (Exception ex)
        {
            MelonLogger.Warning($"[LumenicMenu] Failed to build UIX snapshot: {ex.Message}");
        }

        return result;
    }

    public void SetBool(string categoryId, string entryId, bool value)
    {
        if (FindEntry(categoryId, entryId) is MelonPreferences_Entry<bool> entry)
        {
            entry.Value = value;
            entry.Category.SaveToFile(false);
        }
    }

    public void SetString(string categoryId, string entryId, string value)
    {
        if (FindEntry(categoryId, entryId) is MelonPreferences_Entry<string> entry)
        {
            entry.Value = value ?? string.Empty;
            entry.Category.SaveToFile(false);
        }
    }

    public void SetEnum(string categoryId, string entryId, int index)
    {
        MelonPreferences_Entry entry = FindEntry(categoryId, entryId);
        if (entry == null)
            return;

        Type t = entry.GetReflectedType();
        if (t == null || !t.IsEnum)
            return;

        FieldInfo[] fields = t.GetFields(BindingFlags.DeclaredOnly | BindingFlags.Static | BindingFlags.Public);
        if (index < 0 || index >= fields.Length)
            return;

        object enumVal = fields[index].GetValue(null);
        entry.BoxedValue = enumVal;
        entry.Category.SaveToFile(false);
    }

    private readonly HashSet<MelonPreferences_Category> _pendingSaveCategories = new HashSet<MelonPreferences_Category>();
    private float _pendingSaveFlushAt = -1f;

    public void FlushPendingSaves(float unscaledNow, bool force = false)
    {
        if (_pendingSaveCategories.Count == 0)
            return;
        if (!force && unscaledNow < _pendingSaveFlushAt)
            return;

        foreach (MelonPreferences_Category cat in _pendingSaveCategories)
        {
            try
            {
                cat?.SaveToFile(false);
            }
            catch (Exception ex)
            {
                MelonLogger.Warning($"[LumenicMenu] UIX SaveToFile({cat?.Identifier}) failed: {ex.Message}");
            }
        }
        _pendingSaveCategories.Clear();
        _pendingSaveFlushAt = -1f;
    }

    public void SetNumber(string categoryId, string entryId, double value, bool save)
    {
        if (double.IsNaN(value) || double.IsInfinity(value))
            return;

        MelonPreferences_Entry entry = FindEntry(categoryId, entryId);
        if (entry == null)
            return;

        if (entry is MelonPreferences_Entry<byte> eByte) eByte.Value = (byte)Math.Round(Clamp(value, byte.MinValue, byte.MaxValue));
        else if (entry is MelonPreferences_Entry<sbyte> eSByte) eSByte.Value = (sbyte)Math.Round(Clamp(value, sbyte.MinValue, sbyte.MaxValue));
        else if (entry is MelonPreferences_Entry<short> eShort) eShort.Value = (short)Math.Round(Clamp(value, short.MinValue, short.MaxValue));
        else if (entry is MelonPreferences_Entry<ushort> eUShort) eUShort.Value = (ushort)Math.Round(Clamp(value, ushort.MinValue, ushort.MaxValue));
        else if (entry is MelonPreferences_Entry<int> eInt) eInt.Value = (int)Math.Round(Clamp(value, int.MinValue, int.MaxValue));
        else if (entry is MelonPreferences_Entry<uint> eUInt) eUInt.Value = (uint)Math.Round(Clamp(value, uint.MinValue, uint.MaxValue));
        else if (entry is MelonPreferences_Entry<long> eLong) eLong.Value = (long)Math.Round(Clamp(value, long.MinValue, long.MaxValue));
        else if (entry is MelonPreferences_Entry<ulong> eULong) eULong.Value = (ulong)Math.Round(Clamp(value, ulong.MinValue, ulong.MaxValue));
        else if (entry is MelonPreferences_Entry<float> eFloat) eFloat.Value = (float)value;
        else if (entry is MelonPreferences_Entry<double> eDouble) eDouble.Value = value;
        else return;

        if (save)
        {
            entry.Category.SaveToFile(false);
            _pendingSaveCategories.Remove(entry.Category);
        }
        else if (entry.Category != null)
        {
            _pendingSaveCategories.Add(entry.Category);
            _pendingSaveFlushAt = UnityEngine.Time.unscaledTime + 0.4f;
        }
    }

    public void ClickCustomButton(string categoryId, int globalId)
    {
        if (!IsAvailable)
            return;

        object page = GetExtensionPage(categoryId);
        if (page == null)
            return;

        try
        {
            MethodInfo click = AccessTools.Method(page.GetType(), "ButtonClick", new[] { typeof(int) });
            click?.Invoke(page, new object[] { globalId });
        }
        catch (Exception ex)
        {
            MelonLogger.Warning($"[LumenicMenu] UIX ClickCustomButton({categoryId}, {globalId}) threw: {ex.InnerException?.Message ?? ex.Message}");
        }
    }

    public void ToggleCustomToggle(string categoryId, int globalId, bool state)
    {
        if (!IsAvailable)
            return;

        object page = GetExtensionPage(categoryId);
        if (page == null)
            return;

        _customToggleFallbackState[globalId] = state;

        // Also update ToggleImpl.mySelected if present so snapshot reflects new state immediately.
        try
        {
            if (TryFindControlImpl(page, globalId, out object impl) && impl != null)
            {
                FieldInfo selField = AccessTools.Field(impl.GetType(), "mySelected");
                selField?.SetValue(impl, state);
            }
        }
        catch
        {
            // Fallback map still tracks state.
        }

        try
        {
            MethodInfo toggle = AccessTools.Method(page.GetType(), "ToggleClick", new[] { typeof(int), typeof(bool) });
            toggle?.Invoke(page, new object[] { globalId, state });
        }
        catch (Exception ex)
        {
            MelonLogger.Warning($"[LumenicMenu] UIX ToggleCustomToggle({categoryId}, {globalId}) threw: {ex.InnerException?.Message ?? ex.Message}");
        }
    }

    private object GetExtensionPage(string categoryId)
    {
        if (string.IsNullOrEmpty(categoryId))
            return null;
        IDictionary extensions = _settingPageExtensionsField?.GetValue(null) as IDictionary;
        if (extensions != null && extensions.Contains(categoryId))
            return extensions[categoryId];
        return null;
    }

    private static bool TryFindControlImpl(object page, int globalId, out object impl)
    {
        impl = null;
        FieldInfo regsField = AccessTools.Field(page.GetType(), "myControlRegistrations");
        if (regsField?.GetValue(page) is not IEnumerable list)
            return false;

        foreach (object item in list)
        {
            if (item == null) continue;
            FieldInfo item2Field = item.GetType().GetField("Item2");
            object ctrl = item2Field?.GetValue(item);
            if (ctrl == null) continue;
            FieldInfo idField = AccessTools.Field(ctrl.GetType(), "GlobalId");
            if (idField != null && idField.GetValue(ctrl) is int id && id == globalId)
            {
                impl = ctrl;
                return true;
            }
        }
        return false;
    }

    private List<object> BuildCustomControls(IDictionary extensions, string categoryId)
    {
        var list = new List<object>();
        if (extensions == null || !extensions.Contains(categoryId))
            return list;

        object page = extensions[categoryId];
        if (page == null)
            return list;

        FieldInfo regsField = AccessTools.Field(page.GetType(), "myControlRegistrations");
        if (regsField?.GetValue(page) is not IEnumerable regs)
            return list;

        foreach (object tuple in regs)
        {
            if (tuple == null) continue;
            Type tt = tuple.GetType();
            object reg = tt.GetField("Item1")?.GetValue(tuple);
            object impl = tt.GetField("Item2")?.GetValue(tuple);
            if (reg == null || impl == null) continue;

            int globalId = 0;
            FieldInfo idField = AccessTools.Field(impl.GetType(), "GlobalId");
            if (idField?.GetValue(impl) is int idVal)
                globalId = idVal;

            string text = AccessTools.Field(impl.GetType(), "myText")?.GetValue(impl) as string;
            if (string.IsNullOrEmpty(text))
                text = AccessTools.Field(reg.GetType(), "Text")?.GetValue(reg) as string ?? string.Empty;

            bool isButton = AccessTools.Field(reg.GetType(), "Action")?.GetValue(reg) != null;
            bool isToggle = AccessTools.Field(reg.GetType(), "ToggleAction")?.GetValue(reg) != null;

            string kind;
            bool toggleVal = false;
            if (isButton)
            {
                kind = "button";
            }
            else if (isToggle)
            {
                kind = "toggle";
                if (_customToggleFallbackState.TryGetValue(globalId, out bool cached))
                {
                    toggleVal = cached;
                }
                else
                {
                    FieldInfo selField = AccessTools.Field(impl.GetType(), "mySelected");
                    bool fromImpl = selField?.GetValue(impl) is bool b && b;
                    if (fromImpl)
                    {
                        toggleVal = true;
                    }
                    else if (AccessTools.Field(reg.GetType(), "InitialState")?.GetValue(reg) is Func<bool> initFn)
                    {
                        try { toggleVal = initFn(); } catch { toggleVal = false; }
                    }
                }
            }
            else if (!string.IsNullOrEmpty(text))
            {
                kind = "label";
            }
            else
            {
                kind = "spacer";
            }

            list.Add(new
            {
                global_id = globalId,
                kind,
                text,
                bool_val = toggleVal
            });
        }

        return list;
    }

    private static readonly Dictionary<Type, (object[] Values, string[] Labels)> EnumMetaCache =
        new Dictionary<Type, (object[], string[])>();
    private static readonly Dictionary<Type, FieldInfo> VisibilityFieldCache =
        new Dictionary<Type, FieldInfo>();

    private static (object[] Values, string[] Labels) GetEnumMeta(Type t)
    {
        if (EnumMetaCache.TryGetValue(t, out var cached))
            return cached;

        FieldInfo[] fields = t.GetFields(BindingFlags.DeclaredOnly | BindingFlags.Static | BindingFlags.Public);
        var values = new object[fields.Length];
        var labels = new string[fields.Length];
        for (int i = 0; i < fields.Length; i++)
        {
            DescriptionAttribute attr = fields[i].GetCustomAttribute<DescriptionAttribute>();
            labels[i] = attr != null && !string.IsNullOrEmpty(attr.Description) ? attr.Description : fields[i].Name;
            values[i] = fields[i].GetValue(null);
        }
        var meta = (values, labels);
        EnumMetaCache[t] = meta;
        return meta;
    }

    private static object BuildEntryDto(string categoryId, MelonPreferences_Entry entry, IDictionary ranges)
    {
        string display = string.IsNullOrWhiteSpace(entry.DisplayName) ? entry.Identifier : entry.DisplayName;
        string desc = entry.Description ?? string.Empty;
        Type t = entry.GetReflectedType();

        if (entry is MelonPreferences_Entry<bool> boolEntry)
        {
            return new
            {
                id = entry.Identifier,
                display_name = display,
                description = desc,
                kind = "bool",
                bool_val = boolEntry.Value,
                num_val = 0.0,
                min_val = 0.0,
                max_val = 1.0,
                snap_val = 0.0,
                is_int = false,
                string_val = string.Empty,
                enum_options = Array.Empty<string>(),
                enum_index = 0
            };
        }

        if (entry is MelonPreferences_Entry<string> strEntry)
        {
            return new
            {
                id = entry.Identifier,
                display_name = display,
                description = desc,
                kind = "string",
                bool_val = false,
                num_val = 0.0,
                min_val = 0.0,
                max_val = 1.0,
                snap_val = 0.0,
                is_int = false,
                string_val = strEntry.Value ?? string.Empty,
                enum_options = Array.Empty<string>(),
                enum_index = 0
            };
        }

        if (t != null && t.IsEnum)
        {
            var (values, options) = GetEnumMeta(t);
            int currentIdx = 0;
            object curVal = entry.BoxedValue;

            for (int i = 0; i < values.Length; i++)
            {
                if (Equals(values[i], curVal))
                {
                    currentIdx = i;
                    break;
                }
            }

            return new
            {
                id = entry.Identifier,
                display_name = display,
                description = desc,
                kind = "enum",
                bool_val = false,
                num_val = 0.0,
                min_val = 0.0,
                max_val = 1.0,
                snap_val = 0.0,
                is_int = false,
                string_val = string.Empty,
                enum_options = options,
                enum_index = currentIdx
            };
        }

        if (t != null && IsNumericType(t))
        {
            double cur = Convert.ToDouble(entry.BoxedValue, CultureInfo.InvariantCulture);
            bool isInt = t != typeof(float) && t != typeof(double);
            bool hasRange = TryGetRange(ranges, categoryId, entry.Identifier, out double min, out double max, out double snap);

            return new
            {
                id = entry.Identifier,
                display_name = display,
                description = desc,
                kind = hasRange ? "slider" : "number",
                bool_val = false,
                num_val = cur,
                min_val = hasRange ? min : 0.0,
                max_val = hasRange ? max : 100.0,
                snap_val = hasRange ? snap : (isInt ? 1.0 : 0.0),
                is_int = isInt,
                string_val = string.Empty,
                enum_options = Array.Empty<string>(),
                enum_index = 0
            };
        }

        return null;
    }

    private static bool EvaluateVisibility(IDictionary visibilities, string categoryId, string entryId)
    {
        if (visibilities == null)
            return true;

        var key = (categoryId, entryId);
        if (!visibilities.Contains(key))
            return true;

        object reg = visibilities[key];
        if (reg == null)
            return true;

        Type regType = reg.GetType();
        if (!VisibilityFieldCache.TryGetValue(regType, out FieldInfo visField))
        {
            visField = AccessTools.Field(regType, "IsVisible");
            VisibilityFieldCache[regType] = visField;
        }
        if (visField?.GetValue(reg) is Func<bool> fn)
        {
            try
            {
                return fn();
            }
            catch
            {
                return true;
            }
        }
        return true;
    }

    private static bool TryGetRange(IDictionary ranges, string categoryId, string entryId, out double min, out double max, out double snap)
    {
        min = 0.0;
        max = 1.0;
        snap = 0.0;
        if (ranges == null)
            return false;

        var key = (categoryId, entryId);
        if (!ranges.Contains(key))
            return false;

        object val = ranges[key];
        if (val == null)
            return false;

        if (val is ValueTuple<double, double, double> tup)
        {
            min = tup.Item1;
            max = tup.Item2;
            snap = tup.Item3;
            return max > min;
        }
        return false;
    }

    private static MelonPreferences_Entry FindEntry(string categoryId, string entryId)
    {
        if (string.IsNullOrEmpty(categoryId) || string.IsNullOrEmpty(entryId))
            return null;

        MelonPreferences_Category cat = MelonPreferences.Categories.FirstOrDefault(c => c.Identifier == categoryId);
        return cat?.Entries.FirstOrDefault(e => e.Identifier == entryId);
    }

    private static bool EntryCanBeShown(MelonPreferences_Entry entry)
    {
        if (entry == null || entry.IsHidden)
            return false;

        Type t = entry.GetReflectedType();
        if (t == null)
            return false;

        return t == typeof(bool) || t == typeof(string) || t.IsEnum || IsNumericType(t);
    }

    private static bool IsNumericType(Type t)
    {
        return t == typeof(byte) || t == typeof(sbyte)
            || t == typeof(short) || t == typeof(ushort)
            || t == typeof(int) || t == typeof(uint)
            || t == typeof(long) || t == typeof(ulong)
            || t == typeof(float) || t == typeof(double);
    }

    private static double Clamp(double v, double min, double max)
    {
        if (v < min) return min;
        if (v > max) return max;
        return v;
    }
}
