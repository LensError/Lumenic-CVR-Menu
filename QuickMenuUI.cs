using System;
using System.IO;
using System.Reflection;
using ABI_RC.Systems.UI.UILib;
using ABI_RC.Systems.UI.UILib.UIObjects;
using ABI_RC.Systems.UI.UILib.UIObjects.Components;
using MelonLoader;

namespace LumenicMenu;

/// <summary>
/// Registers LumenicMenu controls under its own UILib Quick Menu tab.
/// </summary>
internal static class QuickMenuUI
{
    private const string QuickMenuModName = "LumenicMenu";
    private const string QuickMenuPageName = "Main";
    private const string QuickMenuIconName = "lumenic_menu";
    private const string EmbeddedIconResource = "LumenicMenu.resources.lumenic_menu.png";

    private static Page _page;
    private static Category _modCategory;
    private static ToggleButton _mainMenuToggle;
    private static ToggleButton _quickMenuToggle;
    private static ToggleButton _hudToggle;
    private static bool _suppressCallbacks;

    public static void PrepareSharedIcon()
    {
        try
        {
            if (QuickMenuAPI.DoesIconExist(QuickMenuModName, QuickMenuIconName))
                return;

            using Stream stream = Assembly.GetExecutingAssembly().GetManifestResourceStream(EmbeddedIconResource);
            if (stream != null)
            {
                QuickMenuAPI.PrepareIcon(QuickMenuModName, QuickMenuIconName, stream);
            }
            else
            {
                MelonLogger.Warning($"[LumenicMenu] Embedded icon resource '{EmbeddedIconResource}' not found.");
            }
        }
        catch (Exception ex)
        {
            MelonLogger.Warning($"[LumenicMenu] Could not prepare QM icon: {ex.Message}");
        }
    }

    public static void Build()
    {
        if (_modCategory != null)
            return;

        try
        {
            _page = Page.GetOrCreatePage(
                QuickMenuModName,
                QuickMenuPageName,
                isRootPage: true,
                tabIcon: QuickMenuIconName);

            _page.MenuTitle = "LumenicMenu";
            _page.MenuSubtitle = "Menu replacement toggles";

            _modCategory = _page.AddCategory("Menus");

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

            _hudToggle = _modCategory.AddToggle(
                "Lumenic HUD",
                "Use unified Lumenic UGUI HUD for notifications, mic, ChatBox, and portals",
                Settings.EnableHud);
            _hudToggle.OnValueUpdated += val =>
            {
                if (_suppressCallbacks) return;
                Settings.SetEnableHud(val);
            };

            BuildHudTestCategory();
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
            if (_hudToggle != null)
                _hudToggle.ToggleValue = Settings.EnableHud;
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

    private static void BuildHudTestCategory()
    {
        Category tests = _page.AddCategory("HUD Tests");

        tests.AddButton(
            "Toast: Info",
            string.Empty,
            "Triggers a compact HUD info toast")
            .OnPress += () =>
            {
                ABI_RC.Core.UI.CohtmlHud.Instance?.ViewDropText(
                    "Lumenic",
                    "Info Notification",
                    "Compact HUD toast message.",
                    "lumenic-test-info",
                    false);
            };

        tests.AddButton(
            "Toast: Server",
            string.Empty,
            "Triggers a long-duration server HUD toast")
            .OnPress += () =>
            {
                ABI_RC.Core.UI.CohtmlHud.Instance?.ViewDropTextLong(
                    "Server",
                    "Instance Notice",
                    "Prop limit changed to 50.",
                    "lumenic-test-long",
                    false);
            };

        tests.AddButton(
            "Feed: Join",
            string.Empty,
            "Adds a join entry to the lower-left feed")
            .OnPress += () =>
            {
                ABI_RC.Core.UI.CohtmlHud.Instance?.ViewDropText(
                    "(Global) Client",
                    "TestPlayer joined",
                    string.Empty,
                    "userJoinLeftTestPlayer",
                    false);
            };

        tests.AddButton(
            "Feed: Leave",
            string.Empty,
            "Adds a leave entry to the lower-left feed")
            .OnPress += () =>
            {
                ABI_RC.Core.UI.CohtmlHud.Instance?.ViewDropText(
                    "(Global) Client",
                    "TestPlayer left",
                    string.Empty,
                    "userJoinLeftTestPlayer",
                    false);
            };

        tests.AddButton(
            "Badge: Friends",
            string.Empty,
            "Toggles the Friends icon chip")
            .OnPress += () => LumenicMenuMod.Instance?.Hud?.TriggerTestBadge("friends");

        tests.AddButton(
            "Badge: Invite",
            string.Empty,
            "Toggles the Invite icon chip")
            .OnPress += () => LumenicMenuMod.Instance?.Hud?.TriggerTestBadge("invites");

        tests.AddButton(
            "Badge: Vote",
            string.Empty,
            "Toggles the Vote icon chip")
            .OnPress += () => LumenicMenuMod.Instance?.Hud?.TriggerTestBadge("votes");

        tests.AddButton(
            "Badge: Alert",
            string.Empty,
            "Toggles the Alert icon chip")
            .OnPress += () => LumenicMenuMod.Instance?.Hud?.TriggerTestBadge("alerts");

        tests.AddButton(
            "Badge: Info",
            string.Empty,
            "Toggles the Info icon chip")
            .OnPress += () => LumenicMenuMod.Instance?.Hud?.TriggerTestBadge("info");

        tests.AddButton(
            "Prop Card",
            string.Empty,
            "Shows the petite Prop Spawn chip")
            .OnPress += () =>
            {
                ABI_RC.Core.UI.CohtmlHud.Instance?.SelectPropToSpawn(
                    string.Empty,
                    "Lumenic Test Prop",
                    "Spawning Prop");
            };

        tests.AddButton(
            "Portal",
            string.Empty,
            "Toggles the Portal HUD indicator")
            .OnPress += () => LumenicMenuMod.Instance?.Hud?.ToggleTestPortal();

        tests.AddButton(
            "ChatBox",
            string.Empty,
            "Displays a sample ChatBox message")
            .OnPress += () => LumenicMenuMod.Instance?.Hud?.TriggerTestChatBox();

        tests.AddButton(
            "Clear Tests",
            string.Empty,
            "Clears all test HUD elements")
            .OnPress += () => LumenicMenuMod.Instance?.Hud?.ClearAllTests();
    }
}
