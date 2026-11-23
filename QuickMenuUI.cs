using System;
using System.IO;
using System.Reflection;
using ABI_RC.Systems.UI.UILib;
using ABI_RC.Systems.UI.UILib.UIObjects;
using ABI_RC.Systems.UI.UILib.UIObjects.Components;
using MelonLoader;

namespace LumenicMenu;

/// <summary>
/// Registers LumenicMenu controls under the shared LensErrorsMods Quick Menu page.
/// </summary>
internal static class QuickMenuUI
{
    private const string SharedQuickMenuModName = "LensErrorsMods";
    private const string SharedQuickMenuPageName = "Main";
    private const string SharedQuickMenuIconName = "lumenic_menu";
    private const string EmbeddedIconResource = "LumenicMenu.resources.lumenic_menu.png";

    private static Page _sharedPage;
    private static Category _modCategory;
    private static ToggleButton _mainMenuToggle;
    private static ToggleButton _quickMenuToggle;
    private static ToggleButton _perfLogToggle;
    private static bool _suppressCallbacks;

    public static void PrepareSharedIcon()
    {
        try
        {
            if (QuickMenuAPI.DoesIconExist(SharedQuickMenuModName, SharedQuickMenuIconName))
                return;

            using Stream stream = Assembly.GetExecutingAssembly().GetManifestResourceStream(EmbeddedIconResource);
            if (stream != null)
            {
                QuickMenuAPI.PrepareIcon(SharedQuickMenuModName, SharedQuickMenuIconName, stream);
            }
            else
            {
                MelonLogger.Warning($"[LumenicMenu] Embedded icon resource '{EmbeddedIconResource}' not found.");
            }
        }
        catch (Exception ex)
        {
            MelonLogger.Warning($"[LumenicMenu] Could not prepare shared QM icon: {ex.Message}");
        }
    }

    public static void Build()
    {
        if (_modCategory != null)
            return;

        try
        {
            _sharedPage = Page.GetOrCreatePage(
                SharedQuickMenuModName,
                SharedQuickMenuPageName,
                isRootPage: true,
                tabIcon: SharedQuickMenuIconName);

            _sharedPage.MenuTitle = "LensError's Mods";
            _sharedPage.MenuSubtitle = "Installed mod settings and actions";

            _modCategory = _sharedPage.AddCategory("LumenicMenu");

            _mainMenuToggle = _modCategory.AddToggle(
                "Lumenic Main Menu",
                "Use Lumenic UI for CVR's Main Menu",
                Settings.EnableMainMenu);
            _mainMenuToggle.OnValueUpdated += val =>
            {
                if (_suppressCallbacks) return;
                Settings.SetEnableMainMenu(val);
            };

            _quickMenuToggle = _modCategory.AddToggle(
                "Lumenic Quick Menu",
                "Use Lumenic UI for CVR's Quick Menu",
                Settings.EnableQuickMenu);
            _quickMenuToggle.OnValueUpdated += val =>
            {
                if (_suppressCallbacks) return;
                Settings.SetEnableQuickMenu(val);
            };

            _perfLogToggle = _modCategory.AddToggle(
                "Perf Timing Log",
                "Log Milestone 1 frame timings (step/readback/upload/blit) to MelonLoader console",
                Settings.PerfLogging);
            _perfLogToggle.OnValueUpdated += val =>
            {
                if (_suppressCallbacks) return;
                Settings.SetPerfLogging(val);
            };
        }
        catch (Exception ex)
        {
            MelonLogger.Error($"[LumenicMenu] Failed to build BTKUILib QuickMenu category: {ex.Message}");
        }
    }

    public static void SyncToggles()
    {
        if (_modCategory == null)
            return;

        _suppressCallbacks = true;
        try
        {
            if (_mainMenuToggle != null)
                _mainMenuToggle.ToggleValue = Settings.EnableMainMenu;
            if (_quickMenuToggle != null)
                _quickMenuToggle.ToggleValue = Settings.EnableQuickMenu;
            if (_perfLogToggle != null)
                _perfLogToggle.ToggleValue = Settings.PerfLogging;
        }
        catch
        {
            // Ignore if BTKUILib page is not ready yet.
        }
        finally
        {
            _suppressCallbacks = false;
        }
    }
}
