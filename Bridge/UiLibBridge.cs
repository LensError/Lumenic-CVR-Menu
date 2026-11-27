using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.Linq;
using System.Net;
using System.Reflection;
using System.Text.RegularExpressions;
using System.Threading.Tasks;
using ABI_RC.Core.InteractionSystem;
using ABI_RC.Core.Networking.API;
using ABI_RC.Core.Networking.API.Responses;
using ABI_RC.Core.Savior;
using ABI_RC.Systems.UI.UILib;
using ABI_RC.Systems.UI.UILib.Components;
using ABI_RC.Systems.UI.UILib.UIObjects;
using ABI_RC.Systems.UI.UILib.UIObjects.Components;
using ABI_RC.Systems.UI.UILib.UIObjects.Objects;
using HarmonyLib;
using LumenicMenu.Native;
using MelonLoader;
using UnityEngine;

namespace LumenicMenu.Bridge;

/// <summary>
/// Full native C# bridge mirroring CVR's ABI_RC.Systems.UI.UILib object graph (Pages, Categories,
/// Buttons, Toggles, Sliders, CycleButtons, TextBlocks, TextInputs, ContentButtons, ContentDetails,
/// MultiSelection, Confirm/Notice/NumberInput modals, AlertToasts, PlayerList, PropList, ColourPicker)
/// into Lumenic Quick Menu.
/// </summary>
internal sealed class UiLibBridge
{
    private const string TabHomeId = "home";
    private const string TabPlayersId = "players";
    private const string TabPropsId = "props";
    private const string TotallyWholesomeModName = "TotallyWholesome";
    private const string TotallyWholesomeRootPageName = "TWRoot";
    private const string TotallyWholesomeStatusEvent = "twStatusUpdate";

    private static readonly Regex HtmlBreakRegex = new Regex(@"<\s*br\s*/?\s*>", RegexOptions.IgnoreCase | RegexOptions.Compiled);
    private static readonly Regex HtmlParagraphCloseRegex = new Regex(@"</\s*p\s*>", RegexOptions.IgnoreCase | RegexOptions.Compiled);
    private static readonly Regex HtmlTagRegex = new Regex(@"<[^>]+>", RegexOptions.Compiled);
    private static readonly Regex ExcessNewlinesRegex = new Regex(@"\n{3,}", RegexOptions.Compiled);

    // Cached reflection accessors for internal/private UILib members.
    private static readonly FieldInfo RootPagesField = AccessTools.Field(typeof(CVR_MenuManager), "RootPages");
    private static readonly FieldInfo SelectedRootPageField = AccessTools.Field(typeof(CVR_MenuManager), "SelectedRootPage");
    private static readonly FieldInfo IsInPlayerListField = AccessTools.Field(typeof(CVR_MenuManager), "IsInPlayerList");
    private static readonly FieldInfo CurrentPageField = AccessTools.Field(typeof(CVR_MenuManager), "_currentPage");
    private static readonly FieldInfo MiscTabPageField = AccessTools.Field(typeof(QuickMenuAPI), "_miscTabPage");
    private static readonly FieldInfo SubElementsField = AccessTools.Field(typeof(QMUIElement), "SubElements");
    private static readonly FieldInfo ElementDeletedField = AccessTools.Field(typeof(QMUIElement), "Deleted");
    private static readonly FieldInfo ElementProtectedField = AccessTools.Field(typeof(QMUIElement), "Protected");
    private static readonly FieldInfo ElementColumnCountField = AccessTools.Field(typeof(QMUIElement), "_columnCount");
    private static readonly FieldInfo ElementDisabledField = AccessTools.Field(typeof(QMUIElement), "_disabled");
    private static readonly FieldInfo ElementHiddenField = AccessTools.Field(typeof(QMUIElement), "_hidden");
    private static readonly FieldInfo ElementVisibleField = AccessTools.Field(typeof(QMUIElement), "_visible");

    private static readonly FieldInfo PageIsRootField = AccessTools.Field(typeof(Page), "IsRootPage");
    private static readonly FieldInfo PageNameField = AccessTools.Field(typeof(Page), "PageName");
    private static readonly FieldInfo PageModNameField = AccessTools.Field(typeof(Page), "ModName");
    private static readonly FieldInfo PageTabIconField = AccessTools.Field(typeof(Page), "_tabIcon");
    private static readonly FieldInfo PageNoTabField = AccessTools.Field(typeof(Page), "_noTab");
    private static readonly FieldInfo PageIsPageOpenField = AccessTools.Field(typeof(Page), "IsPageOpen");
    private static readonly MethodInfo PageChangeVisibilityMethod =
        AccessTools.Method(typeof(Page), "ChangeVisibility", new[] { typeof(bool), typeof(VisibilityChangeSource) });

    private static readonly FieldInfo CategoryShowHeaderField = AccessTools.Field(typeof(Category), "_showHeader");
    private static readonly FieldInfo CategoryCanCollapseField = AccessTools.Field(typeof(Category), "_canCollapse");
    private static readonly FieldInfo CategoryCollapsedField = AccessTools.Field(typeof(Category), "Collapsed");
    private static readonly PropertyInfo CategoryModNameProp = AccessTools.Property(typeof(Category), "ModName");

    private static readonly FieldInfo ButtonHeaderField = AccessTools.Field(typeof(Button), "_headerButton");
    private static readonly FieldInfo ButtonStyleField = AccessTools.Field(typeof(Button), "_style");
    private static readonly FieldInfo CycleHeaderField = AccessTools.Field(typeof(CycleButton), "_headerButton");
    private static readonly FieldInfo ContentButtonLinesField = AccessTools.Field(typeof(ContentButton), "_buttonDescriptionLines");
    private static readonly FieldInfo ContentDetailsBlocksField = AccessTools.Field(typeof(ContentDetails), "_contentTextBlocks");
    private static readonly MethodInfo InteractableOnClickMethod =
        AccessTools.Method(typeof(QMInteractable), "OnClick", new[] { typeof(bool?) });

    private static readonly Type PlayerListType =
        AccessTools.TypeByName("ABI_RC.Systems.UI.UILib.Components.PlayerList");
    private static readonly FieldInfo PlayerListInstanceField =
        PlayerListType != null ? AccessTools.Field(PlayerListType, "Instance") : null;
    private static readonly FieldInfo PlayerListPageField =
        PlayerListType != null ? AccessTools.Field(PlayerListType, "InternalPlayerListPage") : null;
    private static readonly MethodInfo PlayerListSetupMethod =
        PlayerListType != null ? AccessTools.Method(PlayerListType, "SetupPlayerList") : null;

    private static readonly Type PropListType =
        AccessTools.TypeByName("ABI_RC.Systems.UI.UILib.Components.PropList");
    private static readonly PropertyInfo PropListInstanceProp =
        PropListType != null ? AccessTools.Property(PropListType, "Instance") : null;
    private static readonly FieldInfo PropListPageField =
        PropListType != null ? AccessTools.Field(PropListType, "_page") : null;
    private static readonly MethodInfo PropListSetupMethod =
        PropListType != null ? AccessTools.Method(PropListType, "SetupPropList") : null;

    private static readonly Type ColourPickerType =
        AccessTools.TypeByName("ABI_RC.Systems.UI.UILib.Components.ColourPicker");
    private static readonly FieldInfo ColourPickerInstanceField =
        ColourPickerType != null ? AccessTools.Field(ColourPickerType, "Instance") : null;
    private static readonly FieldInfo ColourPickerPageField =
        ColourPickerType != null ? AccessTools.Field(ColourPickerType, "_colourPickerPage") : null;
    private static readonly FieldInfo ColourPickerRField =
        ColourPickerType != null ? AccessTools.Field(ColourPickerType, "_rSlider") : null;
    private static readonly FieldInfo ColourPickerGField =
        ColourPickerType != null ? AccessTools.Field(ColourPickerType, "_gSlider") : null;
    private static readonly FieldInfo ColourPickerBField =
        ColourPickerType != null ? AccessTools.Field(ColourPickerType, "_bSlider") : null;

    private static readonly PropertyInfo CurrentPageIdProp =
        AccessTools.Property(typeof(QuickMenuAPI), nameof(QuickMenuAPI.CurrentPageID));
    private static readonly PropertyInfo SliderValueProp =
        AccessTools.Property(typeof(SliderFloat), nameof(SliderFloat.SliderValue));
    private static readonly PropertyInfo TextInputTextProp =
        AccessTools.Property(typeof(TextInput), nameof(TextInput.Text));
    private static readonly FieldInfo ViewManagerUserDetailsField =
        AccessTools.Field(typeof(ViewManager), "UserDetails");

    private readonly List<Page> _navStack = new List<Page>();
    private readonly Dictionary<string, QMUIElement> _elementsById = new Dictionary<string, QMUIElement>(StringComparer.Ordinal);
    private readonly Dictionary<string, Category> _categoriesById = new Dictionary<string, Category>(StringComparer.Ordinal);
    private readonly ConcurrentDictionary<string, string> _userBioCache = new ConcurrentDictionary<string, string>(StringComparer.Ordinal);
    private readonly ConcurrentDictionary<string, byte> _userBioInFlight = new ConcurrentDictionary<string, byte>(StringComparer.Ordinal);

    public void InvalidateBioCache(string userId)
    {
        if (!string.IsNullOrEmpty(userId))
            _userBioCache.TryRemove(userId, out _);
    }

    private string _activeTabId = TabHomeId;
    private string _toastText = string.Empty;
    private float _toastExpiresAt;

    // Active modal state
    private string _modalKind;
    private string _modalTitle = string.Empty;
    private string _modalBody = string.Empty;
    private string _modalYesText = "Yes";
    private string _modalNoText = "No";
    private string _modalOkText = "OK";
    private Action _modalYesAction;
    private Action _modalNoAction;
    private Action _modalOkAction;
    private MultiSelection _modalMultiSelect;
    private float _modalNumberValue;
    private Action<float> _modalNumberCallback;
    private ulong _modalRevision;

    // Live TotallyWholesome status state captured from CustomEngineOnFunction("twStatusUpdate").
    private string _twBadgeBg = string.Empty;
    private string _twBadgeFg = string.Empty;
    private string _twBadgeText = string.Empty;
    private bool _twDisplayBadge;
    private bool _twPetAutoAccept;
    private bool _twMasterAutoAccept;
    private bool _twVibDevice;
    private bool _twShockDevice;

    public bool IsViewingUiLibPage => _navStack.Count > 0;
    public bool HasActiveToast => !string.IsNullOrEmpty(_toastText) && Time.unscaledTime < _toastExpiresAt;

    public void ReturnHome()
    {
        string prevTab = _activeTabId == TabHomeId ? "CVRMainQM" : _activeTabId;
        CloseAllNavPages(VisibilityChangeSource.TabChange);
        _activeTabId = TabHomeId;
        ClearModal();
        SyncMenuManagerPageState(currentPage: null, selectedRoot: null, inPlayerList: false);
        SetCurrentPageId("CVRMainQM");
        if (prevTab != "CVRMainQM")
        {
            QuickMenuAPI.OnTabChange?.Invoke("CVRMainQM", prevTab);
        }
    }

    public void SelectTab(string tabId)
    {
        if (string.IsNullOrEmpty(tabId) || tabId == TabHomeId)
        {
            ReturnHome();
            return;
        }

        if (tabId == TabPlayersId)
        {
            object pl = PlayerListInstanceField?.GetValue(null);
            if (pl == null)
            {
                PlayerListSetupMethod?.Invoke(null, null);
                pl = PlayerListInstanceField?.GetValue(null);
            }
            Page plPage = pl != null ? PlayerListPageField?.GetValue(pl) as Page : null;
            if (plPage != null)
            {
                _activeTabId = TabPlayersId;
                OpenPageInternal(plPage, resetBreadcrumbs: true);
            }
            return;
        }

        if (tabId == TabPropsId)
        {
            object propList = PropListInstanceProp?.GetValue(null);
            if (propList == null)
            {
                PropListSetupMethod?.Invoke(null, null);
                propList = PropListInstanceProp?.GetValue(null);
            }
            Page propPage = propList != null ? PropListPageField?.GetValue(propList) as Page : null;
            if (propPage != null)
            {
                _activeTabId = TabPropsId;
                OpenPageInternal(propPage, resetBreadcrumbs: true);
            }
            return;
        }

        List<Page> rootPages = GetRootPages();
        Page target = rootPages.FirstOrDefault(p => p != null && GetElementKey(p) == tabId);
        if (target != null)
        {
            _activeTabId = tabId;
            OpenPageInternal(target, resetBreadcrumbs: true);
        }
    }

    public void OpenLocalPlayerPage()
    {
        try
        {
            object pl = PlayerListInstanceField?.GetValue(null);
            if (pl == null)
            {
                PlayerListSetupMethod?.Invoke(null, null);
            }

            string ownerId = ABI_RC.Core.Savior.MetaPort.Instance?.ownerId;
            if (!string.IsNullOrEmpty(ownerId))
            {
                _activeTabId = TabPlayersId;
                QuickMenuAPI.OpenPlayerListByUserID(ownerId);
            }
        }
        catch (Exception ex)
        {
            MelonLogger.Warning($"[LumenicMenu] Failed to open local player QM page: {ex.Message}");
        }
    }

    public void HandleOpenPage(Page page, bool resetBreadcrumbs, bool forceBreadcrumbAdd)
    {
        if (page == null)
            return;

        OpenPageInternal(page, resetBreadcrumbs);
    }

    public void HandleGoBack()
    {
        if (_navStack.Count == 0)
            return;

        Page leaving = _navStack[_navStack.Count - 1];
        _navStack.RemoveAt(_navStack.Count - 1);
        SetPageVisible(leaving, false, VisibilityChangeSource.Navigation);

        string lastId = GetElementKey(leaving);
        if (_navStack.Count > 0)
        {
            Page target = _navStack[_navStack.Count - 1];
            string targetId = GetElementKey(target);
            bool inPlayerList = targetId == "CVRUI-QMUI-PlayerList";
            Page root = GetBoolField(PageIsRootField, target) ? target : _navStack[0];
            SyncMenuManagerPageState(target, root, inPlayerList);
            SetCurrentPageId(targetId);
            SetPageVisible(target, true, VisibilityChangeSource.Navigation);
            QuickMenuAPI.OnBackAction?.Invoke(targetId, lastId);
        }
        else
        {
            string prevTab = _activeTabId;
            _activeTabId = TabHomeId;
            SyncMenuManagerPageState(null, null, inPlayerList: false);
            SetCurrentPageId("CVRMainQM");
            QuickMenuAPI.OnBackAction?.Invoke("CVRMainQM", lastId);
            if (prevTab != TabHomeId)
            {
                QuickMenuAPI.OnTabChange?.Invoke("CVRMainQM", prevTab);
            }
        }
    }

    public void HandleShowConfirm(string title, string content, Action onYes, Action onNo, string yesText, string noText)
    {
        ClearModal();
        _modalKind = "confirm";
        _modalTitle = title ?? "Confirm";
        _modalBody = SanitizeModalHtml(content);
        _modalYesText = string.IsNullOrEmpty(yesText) ? "Yes" : yesText;
        _modalNoText = string.IsNullOrEmpty(noText) ? "No" : noText;
        _modalYesAction = onYes;
        _modalNoAction = onNo;
        _modalRevision++;
    }

    public void HandleShowNotice(string title, string content, Action onOk, string okText)
    {
        ClearModal();
        _modalKind = "notice";
        _modalTitle = title ?? "Notice";
        _modalBody = SanitizeModalHtml(content);
        _modalOkText = string.IsNullOrEmpty(okText) ? "OK" : okText;
        _modalOkAction = onOk;
        _modalRevision++;
    }

    public bool HandleCustomEngineEvent(string functionName, object[] args)
    {
        if (!string.Equals(functionName, TotallyWholesomeStatusEvent, StringComparison.Ordinal)
            || args == null
            || args.Length < 8)
        {
            return false;
        }

        string badgeBg = args[0]?.ToString()?.Trim() ?? string.Empty;
        string badgeFg = args[1]?.ToString()?.Trim() ?? string.Empty;
        string badgeText = args[2]?.ToString()?.Trim() ?? string.Empty;
        bool displayBadge = args[3] is bool db && db;
        bool petAuto = args[4] is bool pa && pa;
        bool masterAuto = args[5] is bool ma && ma;
        bool vibDev = args[6] is bool vd && vd;
        bool shockDev = args[7] is bool sd && sd;

        bool changed = !string.Equals(_twBadgeBg, badgeBg, StringComparison.Ordinal)
            || !string.Equals(_twBadgeFg, badgeFg, StringComparison.Ordinal)
            || !string.Equals(_twBadgeText, badgeText, StringComparison.Ordinal)
            || _twDisplayBadge != displayBadge
            || _twPetAutoAccept != petAuto
            || _twMasterAutoAccept != masterAuto
            || _twVibDevice != vibDev
            || _twShockDevice != shockDev;

        if (!changed)
            return false;

        _twBadgeBg = badgeBg;
        _twBadgeFg = badgeFg;
        _twBadgeText = badgeText;
        _twDisplayBadge = displayBadge;
        _twPetAutoAccept = petAuto;
        _twMasterAutoAccept = masterAuto;
        _twVibDevice = vibDev;
        _twShockDevice = shockDev;
        return true;
    }

    public void HandleOpenMultiSelect(MultiSelection multiSelection)
    {
        if (multiSelection == null)
            return;

        ClearModal();
        try { multiSelection.OnMultiSelectPreOpen?.Invoke(); } catch (Exception ex) { MelonLogger.Error($"[LumenicMenu] MultiSelect PreOpen error: {ex.Message}"); }
        _modalKind = "multi_select";
        _modalTitle = multiSelection.Name ?? "Select Option";
        _modalMultiSelect = multiSelection;
        _modalRevision++;
        try { multiSelection.OnMultiSelectOpened?.Invoke(); } catch (Exception ex) { MelonLogger.Error($"[LumenicMenu] MultiSelect Opened error: {ex.Message}"); }
    }

    public void HandleOpenNumberInput(string name, float input, Action<float> onCompleted)
    {
        ClearModal();
        _modalKind = "number_input";
        _modalTitle = string.IsNullOrEmpty(name) ? "Number Input" : name;
        _modalNumberValue = input;
        _modalNumberCallback = onCompleted;
        _modalRevision++;
    }

    public void HandleShowAlertToast(string message, int delay)
    {
        _toastText = message ?? string.Empty;
        _toastExpiresAt = Time.unscaledTime + Mathf.Clamp(delay <= 0 ? 4 : delay, 1, 15);
    }

    public void ToggleCategory(string categoryId)
    {
        if (!_categoriesById.TryGetValue(categoryId, out Category cat) || cat == null)
            return;

        bool cur = GetBoolField(CategoryCollapsedField, cat);
        bool next = !cur;
        CategoryCollapsedField?.SetValue(cat, next);
        InvokeModCallback("Category.OnCollapse", () => cat.OnCollapse?.Invoke(next));
    }

    private int _selfActionJokeIndex;

    private static readonly (string Title, string Body, string Ok)[] SelfBlockJokes =
    {
        ("Nice Try", "You can't block yourself. You're stuck with you forever.", "Fine..."),
        ("Paradox Detected", "Blocking yourself would create a paradox and delete the universe. Request denied.", "Fair enough"),
        ("Mirror Check", "Running away from your own reflection isn't supported yet.", "Okay, okay"),
        ("Error 418", "Cannot block your own inner monologue. Drink some water instead.", "Got it")
    };

    private static readonly (string Title, string Body, string Ok)[] SelfUnfriendJokes =
    {
        ("Be Kind to Yourself", "You can't unfriend yourself! Someone has to stay on your friends list.", "I'll be nice"),
        ("Friendship Forever", "Self-unfriending failed: you're already your own best friend.", "Fair enough"),
        ("Breakup Denied", "No breaking up with yourself in ChilloutVR. Take a deep breath instead.", "Okay, okay"),
        ("Nice Try", "You can't unfriend yourself. You two are stuck together.", "Fine...")
    };

    private bool IsSelfModerationButton(QMUIElement el, out bool isBlock, out bool isFriendAction)
    {
        isBlock = false;
        isFriendAction = false;
        if (el is not Button btn || _navStack.Count == 0)
            return false;

        Page activePage = _navStack[_navStack.Count - 1];
        if (activePage == null || GetElementKey(activePage) != "CVRUI-QMUI-PlayerSelectPage")
            return false;

        string myId = MetaPort.Instance?.ownerId ?? ABI_RC.Core.Networking.AuthManager.UserId;
        string selId = QuickMenuAPI.SelectedPlayerID;
        bool isSelf = string.IsNullOrEmpty(selId)
            || (!string.IsNullOrEmpty(myId) && string.Equals(selId, myId, StringComparison.OrdinalIgnoreCase));
        if (!isSelf)
            return false;

        string text = (btn.ButtonText ?? string.Empty).ToLowerInvariant();
        string tip = (btn.ButtonTooltip ?? string.Empty).ToLowerInvariant();
        isBlock = (text.Contains("block") && !text.Contains("unblock"))
            || tip.Contains("blocks the selected user");
        isFriendAction = text.Contains("unfriend")
            || text.Contains("remove friend")
            || text.Contains("add friend")
            || tip.Contains("unfriend")
            || tip.Contains("friend request");
        return isBlock || isFriendAction;
    }

    private bool TryInterceptSelfModerationClick(QMUIElement el)
    {
        if (!IsSelfModerationButton(el, out bool isBlock, out bool isFriendAction))
            return false;

        if (isBlock)
        {
            var joke = SelfBlockJokes[_selfActionJokeIndex++ % SelfBlockJokes.Length];
            HandleShowNotice(joke.Title, joke.Body, null, joke.Ok);
            return true;
        }
        if (isFriendAction)
        {
            var joke = SelfUnfriendJokes[_selfActionJokeIndex++ % SelfUnfriendJokes.Length];
            HandleShowNotice(joke.Title, joke.Body, null, joke.Ok);
            return true;
        }
        return false;
    }

    public void ClickElement(string elementId)
    {
        if (!_elementsById.TryGetValue(elementId, out QMUIElement el) || el == null)
            return;

        if (TryInterceptSelfModerationClick(el))
            return;

        if (GetBoolField(ElementDisabledField, el))
            return;

        if (el is QMInteractable interactable && InteractableOnClickMethod != null)
        {
            InvokeModCallback($"ClickElement({elementId})", () => InteractableOnClickMethod.Invoke(interactable, new object[] { null }));
        }
    }

    public void HoldElement(string elementId)
    {
        if (!_elementsById.TryGetValue(elementId, out QMUIElement el) || el == null)
            return;

        if (GetBoolField(ElementDisabledField, el))
            return;

        if (el is QMInteractable interactable)
        {
            InvokeModCallback($"HoldElement({elementId})", () => interactable.OnHeld?.Invoke());
        }
    }

    public void ToggleElement(string elementId, bool value)
    {
        if (!_elementsById.TryGetValue(elementId, out QMUIElement el) || el == null)
            return;

        if (GetBoolField(ElementDisabledField, el))
            return;

        if (el is ToggleButton tb && InteractableOnClickMethod != null)
        {
            InvokeModCallback($"ToggleElement({elementId})", () => InteractableOnClickMethod.Invoke(tb, new object[] { value }));
        }
    }

    public void SetSlider(string elementId, float value, bool reset)
    {
        if (!_elementsById.TryGetValue(elementId, out QMUIElement el) || el is not SliderFloat slider)
            return;

        if (GetBoolField(ElementDisabledField, el))
            return;

        if (float.IsNaN(value) || float.IsInfinity(value))
            return;

        float clamped = Mathf.Clamp(value, slider.MinValue, slider.MaxValue);
        InvokeModCallback($"SetSlider({elementId})", () =>
        {
            SliderValueProp?.SetValue(slider, clamped);
            if (reset)
                slider.OnSliderReset?.Invoke();
        });
    }

    public void OpenTextInput(string elementId)
    {
        if (!_elementsById.TryGetValue(elementId, out QMUIElement el) || el is not TextInput ti)
            return;

        if (GetBoolField(ElementDisabledField, el))
            return;

        LumenicMenuMod.Instance?.ClearSurfaceFocus(NativeBridge.SurfaceQuick);
        QuickMenuAPI.OpenKeyboard(ti.Text ?? string.Empty, s =>
        {
            InvokeModCallback($"TextInput({elementId})", () => TextInputTextProp?.SetValue(ti, s ?? string.Empty));
            LumenicMenuMod.Instance?.MarkStateDirty();
        });
    }

    public void ModalConfirm(bool accepted)
    {
        Action cb = accepted ? _modalYesAction : _modalNoAction;
        ClearModal();
        InvokeModCallback("ModalConfirm", () => cb?.Invoke());
    }

    public void ModalNoticeOk()
    {
        Action cb = _modalOkAction;
        ClearModal();
        InvokeModCallback("ModalNoticeOk", () => cb?.Invoke());
    }

    public void ModalSelectOption(int index)
    {
        MultiSelection ms = _modalMultiSelect;
        ClearModal();
        if (ms != null && ms.Options != null && index >= 0 && index < ms.Options.Length)
        {
            InvokeModCallback("ModalSelectOption", () => ms.SelectedOption = index);
        }
    }

    public void ModalNumberSubmit(float value)
    {
        Action<float> cb = _modalNumberCallback;
        ClearModal();
        if (!float.IsNaN(value) && !float.IsInfinity(value))
            InvokeModCallback("ModalNumberSubmit", () => cb?.Invoke(value));
    }

    public void ModalDismiss()
    {
        Action cancelCb = _modalKind == "confirm"
            ? _modalNoAction
            : (_modalKind == "notice" ? _modalOkAction : null);
        ClearModal();
        InvokeModCallback("ModalDismiss", () => cancelCb?.Invoke());
    }

    private static void InvokeModCallback(string label, Action action)
    {
        try
        {
            action();
        }
        catch (Exception ex)
        {
            MelonLogger.Warning($"[LumenicMenu] UILib {label} callback threw: {ex.InnerException?.Message ?? ex.Message}");
        }
    }

    public object BuildSnapshot(ImageLoaderService images)
    {
        // Expire toast cleanly.
        string toast = HasActiveToast ? _toastText : string.Empty;
        if (string.IsNullOrEmpty(toast))
            _toastText = string.Empty;

        // Prune deleted pages from nav stack.
        while (_navStack.Count > 0 && (_navStack[_navStack.Count - 1] == null || IsDeleted(_navStack[_navStack.Count - 1])))
        {
            _navStack.RemoveAt(_navStack.Count - 1);
        }
        if (_navStack.Count == 0 && _activeTabId != TabHomeId)
        {
            _activeTabId = TabHomeId;
        }

        List<object> tabs = BuildTabsSnapshot(images);
        object activePageDto = _navStack.Count > 0
            ? BuildActivePageSnapshot(_navStack[_navStack.Count - 1], images)
            : null;
        object modalDto = BuildModalSnapshot();

        return new
        {
            tabs,
            active_page = activePageDto,
            modal = modalDto,
            toast
        };
    }

    private void OpenPageInternal(Page page, bool resetBreadcrumbs)
    {
        string prevId = _navStack.Count > 0 ? GetElementKey(_navStack[_navStack.Count - 1]) : "CVRMainQM";
        string prevTab = _activeTabId == TabHomeId ? "CVRMainQM" : _activeTabId;

        if (resetBreadcrumbs)
        {
            CloseAllNavPages(VisibilityChangeSource.TabChange);
        }
        else if (_navStack.Count > 0)
        {
            Page top = _navStack[_navStack.Count - 1];
            if (ReferenceEquals(top, page))
                return;
            SetPageVisible(top, false, VisibilityChangeSource.Navigation);
        }

        // Update active tab indicator if opening a root/special page.
        object pl = PlayerListInstanceField?.GetValue(null);
        Page plPage = pl != null ? PlayerListPageField?.GetValue(pl) as Page : null;
        object propList = PropListInstanceProp?.GetValue(null);
        Page propPage = propList != null ? PropListPageField?.GetValue(propList) as Page : null;

        if (ReferenceEquals(page, plPage))
        {
            _activeTabId = TabPlayersId;
        }
        else if (ReferenceEquals(page, propPage))
        {
            _activeTabId = TabPropsId;
        }
        else if (GetBoolField(PageIsRootField, page)
            && (!GetBoolField(PageNoTabField, page) || IsTotallyWholesomeRootPage(page)))
        {
            _activeTabId = GetElementKey(page);
        }

        int existingIdx = _navStack.IndexOf(page);
        if (existingIdx >= 0)
        {
            _navStack.RemoveRange(existingIdx + 1, _navStack.Count - existingIdx - 1);
        }
        else
        {
            _navStack.Add(page);
        }

        string targetId = GetElementKey(page);
        bool inPlayerList = ReferenceEquals(page, plPage) || targetId == "CVRUI-QMUI-PlayerList";
        Page rootPage = GetBoolField(PageIsRootField, page) ? page : _navStack[0];
        SyncMenuManagerPageState(page, rootPage, inPlayerList);

        // Set CurrentPageID BEFORE ChangeVisibility so mod OnPageOpen / OnVisibilityChanged see the new ID.
        SetCurrentPageId(targetId);
        SetPageVisible(page, true, resetBreadcrumbs ? VisibilityChangeSource.TabChange : VisibilityChangeSource.Navigation);

        if (resetBreadcrumbs && prevTab != targetId)
        {
            QuickMenuAPI.OnTabChange?.Invoke(targetId, prevTab);
        }
        QuickMenuAPI.OnOpenedPage?.Invoke(targetId, prevId);
    }

    private static void SyncMenuManagerPageState(Page currentPage, Page selectedRoot, bool inPlayerList)
    {
        try
        {
            SelectedRootPageField?.SetValue(null, selectedRoot);
            IsInPlayerListField?.SetValue(null, inPlayerList);
            if (CVR_MenuManager.Instance != null)
            {
                CurrentPageField?.SetValue(CVR_MenuManager.Instance, currentPage);
            }
        }
        catch
        {
            // Keep UI alive if CVR_MenuManager internals change.
        }
    }

    private static void SetCurrentPageId(string id)
    {
        try
        {
            CurrentPageIdProp?.SetValue(null, id);
        }
        catch
        {
            // Ignore if setter unavailable.
        }
    }

    private void CloseAllNavPages(VisibilityChangeSource source)
    {
        for (int i = _navStack.Count - 1; i >= 0; i--)
        {
            if (_navStack[i] != null)
            {
                SetPageVisible(_navStack[i], false, source);
            }
        }
        _navStack.Clear();
    }

    private static void SetPageVisible(Page page, bool visible, VisibilityChangeSource source)
    {
        if (page == null) return;
        try
        {
            ElementVisibleField?.SetValue(page, visible);
            if (PageChangeVisibilityMethod != null)
            {
                PageChangeVisibilityMethod.Invoke(page, new object[] { visible, source });
            }
        }
        catch (Exception ex)
        {
            MelonLogger.Warning($"[LumenicMenu] Page.ChangeVisibility failed: {ex.Message}");
        }
    }

    private void ClearModal()
    {
        _modalKind = null;
        _modalTitle = string.Empty;
        _modalBody = string.Empty;
        _modalYesAction = null;
        _modalNoAction = null;
        _modalOkAction = null;
        _modalMultiSelect = null;
        _modalNumberCallback = null;
    }

    private List<object> BuildTabsSnapshot(ImageLoaderService images)
    {
        var tabs = new List<object>
        {
            new
            {
                id = TabHomeId,
                label = "Home",
                mod_name = "Lumenic",
                icon_key = string.Empty,
                active = _activeTabId == TabHomeId,
                kind = "home"
            },
            new
            {
                id = TabPlayersId,
                label = "Players",
                mod_name = "CVR",
                icon_key = string.Empty,
                active = _activeTabId == TabPlayersId,
                kind = "players"
            },
            new
            {
                id = TabPropsId,
                label = "Props",
                mod_name = "CVR",
                icon_key = string.Empty,
                active = _activeTabId == TabPropsId,
                kind = "props"
            }
        };

        object pl = PlayerListInstanceField?.GetValue(null);
        Page plPage = pl != null ? PlayerListPageField?.GetValue(pl) as Page : null;
        object propList = PropListInstanceProp?.GetValue(null);
        Page propPage = propList != null ? PropListPageField?.GetValue(propList) as Page : null;

        var seenIds = new HashSet<string>(StringComparer.Ordinal);
        foreach (Page page in GetRootPages())
        {
            if (page == null || IsDeleted(page))
                continue;
            if (ReferenceEquals(page, plPage) || ReferenceEquals(page, propPage))
                continue;

            bool isRoot = GetBoolField(PageIsRootField, page);
            bool noTab = GetBoolField(PageNoTabField, page);
            bool isTwRoot = IsTotallyWholesomeRootPage(page);
            if (!isRoot || (noTab && !isTwRoot) || page.HideTab || page.Hidden)
                continue;

            string id = GetElementKey(page);
            if (string.IsNullOrEmpty(id) || !seenIds.Add(id))
                continue;

            // Hide empty Misc tab to match vanilla CVR behavior without triggering the lazy MiscTabPage getter.
            Page existingMisc = MiscTabPageField?.GetValue(null) as Page;
            if ((existingMisc != null && ReferenceEquals(page, existingMisc) || id == "CVRUI-QMUI-Misc-Misc")
                && GetSubElements(page).Count == 0)
                continue;

            string modName = PageModNameField?.GetValue(page) as string ?? string.Empty;
            string tabIcon = PageTabIconField?.GetValue(page) as string ?? string.Empty;
            if (isTwRoot && string.IsNullOrWhiteSpace(tabIcon))
            {
                tabIcon = QuickMenuAPI.DoesIconExist(TotallyWholesomeModName, "TWLogoPride")
                    ? "TWLogoPride"
                    : "TWTabIcon";
            }

            string label = isTwRoot
                ? "Totally Wholesome"
                : (!string.IsNullOrWhiteSpace(page.MenuTitle)
                    ? page.MenuTitle
                    : (!string.IsNullOrWhiteSpace(page.PageDisplayName)
                        ? page.PageDisplayName
                        : (!string.IsNullOrWhiteSpace(modName) ? modName : "Mod")));

            bool isToolbox = string.Equals(label, "Toolbox", StringComparison.OrdinalIgnoreCase)
                || string.Equals(modName, "Toolbox", StringComparison.OrdinalIgnoreCase)
                || id.IndexOf("Toolbox", StringComparison.OrdinalIgnoreCase) >= 0;

            if (isToolbox)
            {
                // Place CVR's built-in Toolbox tab right after Props with matching default styling.
                tabs.Insert(3, new
                {
                    id,
                    label = "Toolbox",
                    mod_name = "CVR",
                    icon_key = string.Empty,
                    active = _activeTabId == id,
                    kind = "toolbox",
                    status_dots = Array.Empty<string>()
                });
                continue;
            }

            string iconKey = images.RequestUiLibIcon(NativeBridge.SurfaceQuick, modName, tabIcon);

            tabs.Add(new
            {
                id,
                label,
                mod_name = modName,
                icon_key = iconKey,
                active = _activeTabId == id,
                kind = "mod",
                status_dots = isTwRoot ? BuildTotallyWholesomeStatusDots() : Array.Empty<string>()
            });
        }

        return tabs;
    }

    private object BuildActivePageSnapshot(Page page, ImageLoaderService images)
    {
        _elementsById.Clear();
        _categoriesById.Clear();

        string pageMod = PageModNameField?.GetValue(page) as string ?? string.Empty;
        string title = !string.IsNullOrWhiteSpace(page.MenuTitle)
            ? page.MenuTitle
            : (!string.IsNullOrWhiteSpace(page.PageDisplayName)
                ? page.PageDisplayName
                : (PageNameField?.GetValue(page) as string ?? "Page"));
        string subtitle = page.MenuSubtitle ?? string.Empty;

        var breadcrumbs = new List<string>();
        foreach (Page p in _navStack)
        {
            if (p == null) continue;
            string crumb = !string.IsNullOrWhiteSpace(p.PageDisplayName)
                ? p.PageDisplayName
                : (!string.IsNullOrWhiteSpace(p.MenuTitle) ? p.MenuTitle : "Page");
            breadcrumbs.Add(crumb);
        }

        var headerButtons = new List<object>();
        var topSliders = new List<object>();
        var categories = new List<object>();

        foreach (QMUIElement child in GetSubElements(page))
        {
            if (child == null || IsDeleted(child) || IsHidden(child))
                continue;

            if (child is Category cat)
            {
                string catId = GetElementKey(cat);
                _categoriesById[catId] = cat;

                string catMod = CategoryModNameProp?.GetValue(cat) as string ?? pageMod;
                bool showHeader = GetBoolField(CategoryShowHeaderField, cat, defaultVal: true);
                bool canCollapse = GetBoolField(CategoryCanCollapseField, cat, defaultVal: true);
                bool collapsed = GetBoolField(CategoryCollapsedField, cat);

                var catHeaderButtons = new List<object>();
                var catElements = new List<object>();

                foreach (QMUIElement item in GetSubElements(cat))
                {
                    if (item == null || IsDeleted(item) || IsHidden(item))
                        continue;

                    bool isHeaderBtn = (item is Button && GetBoolField(ButtonHeaderField, item))
                        || (item is CycleButton && GetBoolField(CycleHeaderField, item));

                    object elDto = BuildElementDto(item, catMod, images);
                    if (elDto == null)
                        continue;

                    if (isHeaderBtn)
                        catHeaderButtons.Add(elDto);
                    else
                        catElements.Add(elDto);
                }

                categories.Add(new
                {
                    id = catId,
                    name = cat.CategoryName ?? string.Empty,
                    show_header = showHeader,
                    can_collapse = canCollapse,
                    collapsed,
                    header_buttons = catHeaderButtons,
                    elements = catElements
                });
            }
            else if (child is SliderFloat topSlider)
            {
                object dto = BuildElementDto(topSlider, pageMod, images);
                if (dto != null)
                    topSliders.Add(dto);
            }
            else if (child is Button || child is CycleButton)
            {
                object dto = BuildElementDto(child, pageMod, images);
                if (dto != null)
                    headerButtons.Add(dto);
            }
        }

        string pageKey = GetElementKey(page);
        string colourPreviewHex = TryGetColourPickerPreviewHex(page);
        string playerPreviewStatus = pageKey == "CVRUI-QMUI-PlayerSelectPage"
            ? (LumenicMenuMod.Instance?.AvatarStage?.QuerySelectedPlayerPreviewStatus() ?? string.Empty)
            : string.Empty;
        string selectedPlayerId = pageKey == "CVRUI-QMUI-PlayerSelectPage"
            ? (QuickMenuAPI.SelectedPlayerID ?? string.Empty)
            : string.Empty;
        string playerBio = pageKey == "CVRUI-QMUI-PlayerSelectPage"
            ? ResolveSelectedPlayerBio()
            : string.Empty;

        bool isRootWithTab = GetBoolField(PageIsRootField, page)
            && (!GetBoolField(PageNoTabField, page) || IsTotallyWholesomeRootPage(page));
        bool canGoBack = _navStack.Count > 1 || !isRootWithTab;
        object[] statusBadges = string.Equals(pageMod, TotallyWholesomeModName, StringComparison.OrdinalIgnoreCase)
            ? BuildTotallyWholesomeStatusBadges()
            : Array.Empty<object>();

        return new
        {
            id = pageKey,
            title,
            subtitle,
            mod_name = pageMod,
            can_go_back = canGoBack,
            breadcrumbs,
            colour_preview_hex = colourPreviewHex,
            player_preview_status = playerPreviewStatus,
            selected_player_id = selectedPlayerId,
            player_bio = playerBio,
            status_badges = statusBadges,
            header_buttons = headerButtons,
            top_sliders = topSliders,
            categories
        };
    }

    private string ResolveSelectedPlayerBio()
    {
        try
        {
            string uuid = QuickMenuAPI.SelectedPlayerID;
            string ownerId = MetaPort.Instance?.ownerId;
            if (string.IsNullOrEmpty(uuid))
                uuid = ownerId;
            if (string.IsNullOrEmpty(uuid))
                return string.Empty;

            if (!string.IsNullOrEmpty(ownerId) && uuid == ownerId && ViewManager.Instance != null && ViewManagerUserDetailsField != null)
            {
                if (ViewManagerUserDetailsField.GetValue(ViewManager.Instance) is UserDetailsResponse localDetails)
                {
                    string localBio = localDetails.ProfileBio?.Trim() ?? string.Empty;
                    _userBioCache[uuid] = localBio;
                    return localBio;
                }
            }

            if (_userBioCache.TryGetValue(uuid, out string cached))
                return cached;

            if (_userBioInFlight.TryAdd(uuid, 1))
            {
                string targetUuid = uuid;
                Task.Run(async () =>
                {
                    try
                    {
                        var resp = await ApiConnection.MakeRequest<UserDetailsResponse>(
                            ApiConnection.ApiOperation.UserDetails,
                            new { userID = targetUuid });
                        string bio = resp?.Data?.ProfileBio?.Trim() ?? string.Empty;
                        _userBioCache[targetUuid] = bio;
                    }
                    catch
                    {
                        _userBioCache[targetUuid] = string.Empty;
                    }
                    finally
                    {
                        _userBioInFlight.TryRemove(targetUuid, out _);
                        LumenicMenuMod.Instance?.MarkStateDirty();
                    }
                });
            }

            return "Loading bio...";
        }
        catch
        {
            return string.Empty;
        }
    }

    private string TryGetColourPickerPreviewHex(Page page)
    {
        try
        {
            object cp = ColourPickerInstanceField?.GetValue(null);
            if (cp == null) return string.Empty;
            Page cpPage = ColourPickerPageField?.GetValue(cp) as Page;
            if (!ReferenceEquals(cpPage, page)) return string.Empty;

            var r = ColourPickerRField?.GetValue(cp) as SliderFloat;
            var g = ColourPickerGField?.GetValue(cp) as SliderFloat;
            var b = ColourPickerBField?.GetValue(cp) as SliderFloat;
            if (r == null || g == null || b == null) return string.Empty;

            Color c = new Color(Mathf.Clamp01(r.SliderValue), Mathf.Clamp01(g.SliderValue), Mathf.Clamp01(b.SliderValue), 1f);
            return ColorUtility.ToHtmlStringRGB(c);
        }
        catch
        {
            return string.Empty;
        }
    }

    private static FieldInfo _btkButtonOnHeldField;
    private static bool _btkButtonOnHeldResolved;

    // BTKUILib's Button shim unconditionally attaches a forwarding lambda to QMInteractable.OnHeld,
    // so check the wrapper's own OnHeld delegate to avoid flagging every BTKUILib button as holdable.
    private static bool HasRealHoldHandler(QMInteractable inter)
    {
        Action onHeld = inter?.OnHeld;
        if (onHeld == null)
            return false;

        foreach (Delegate d in onHeld.GetInvocationList())
        {
            object target = d.Target;
            if (target != null && target.GetType().FullName == "BTKUILib.UIObjects.Components.Button")
            {
                if (!_btkButtonOnHeldResolved)
                {
                    _btkButtonOnHeldField = AccessTools.Field(target.GetType(), "OnHeld");
                    _btkButtonOnHeldResolved = true;
                }
                if (_btkButtonOnHeldField?.GetValue(target) is Delegate btkHeld && btkHeld != null)
                    return true;
                continue;
            }

            return true;
        }

        return false;
    }

    private object BuildElementDto(QMUIElement el, string modName, ImageLoaderService images)
    {
        string id = GetElementKey(el);
        _elementsById[id] = el;

        bool disabled = GetBoolField(ElementDisabledField, el);
        int colSpan = GetIntField(ElementColumnCountField, el, -1);
        bool hasHold = el is QMInteractable inter && HasRealHoldHandler(inter);

        switch (el)
        {
            case ToggleButton tb:
                return new
                {
                    id,
                    kind = "toggle",
                    label = tb.ToggleName ?? string.Empty,
                    tooltip = tb.ToggleTooltip ?? string.Empty,
                    disabled,
                    bool_val = tb.ToggleValue,
                    col_span = colSpan
                };

            case Button btn:
                string btnStyle = ButtonStyleField?.GetValue(btn)?.ToString() ?? "TextWithIcon";
                string btnIconKey = images.RequestUiLibIcon(NativeBridge.SurfaceQuick, modName, btn.ButtonIcon);
                return new
                {
                    id,
                    kind = "button",
                    label = btn.ButtonText ?? string.Empty,
                    tooltip = btn.ButtonTooltip ?? string.Empty,
                    icon_key = btnIconKey,
                    disabled,
                    has_hold = hasHold,
                    col_span = colSpan,
                    style = btnStyle
                };

            case CycleButton cyc:
                string cycLabel = string.Empty;
                string cycIconName = string.Empty;
                try
                {
                    KeyValuePair<string, string> sel = cyc.SelectedOption;
                    cycLabel = sel.Key ?? string.Empty;
                    cycIconName = sel.Value ?? string.Empty;
                }
                catch
                {
                    // Guard empty CycleButton options collection.
                }
                string cycIconKey = images.RequestUiLibIcon(NativeBridge.SurfaceQuick, modName, cycIconName);
                return new
                {
                    id,
                    kind = "cycle",
                    label = cycLabel,
                    tooltip = cyc.ButtonTooltip ?? string.Empty,
                    icon_key = cycIconKey,
                    disabled,
                    col_span = colSpan
                };

            case SliderFloat sl:
                return new
                {
                    id,
                    kind = "slider",
                    label = sl.SliderName ?? string.Empty,
                    tooltip = sl.SliderTooltip ?? string.Empty,
                    disabled,
                    float_val = sl.SliderValue,
                    min_val = sl.MinValue,
                    max_val = sl.MaxValue,
                    default_val = sl.DefaultValue,
                    allow_reset = sl.AllowDefaultReset,
                    decimals = sl.DecimalPlaces
                };

            case TextBlock txt:
                return new
                {
                    id,
                    kind = "text_block",
                    label = txt.Text ?? string.Empty
                };

            case TextInput ti:
                return new
                {
                    id,
                    kind = "text_input",
                    text_val = ti.Text ?? string.Empty,
                    placeholder = ti.Placeholder ?? string.Empty,
                    disabled
                };

            case ContentButton cb:
                var lines = ContentButtonLinesField?.GetValue(cb) as List<string>;
                string cbIconKey = images.RequestUiLibIcon(NativeBridge.SurfaceQuick, modName, cb.ButtonIcon);
                string cbSubIconKey = images.RequestUiLibIcon(NativeBridge.SurfaceQuick, modName, cb.ButtonSubIcon);
                return new
                {
                    id,
                    kind = "content_button",
                    label = cb.ButtonText ?? string.Empty,
                    tooltip = cb.ButtonTooltip ?? string.Empty,
                    icon_key = cbIconKey,
                    sub_icon_key = cbSubIconKey,
                    disabled,
                    lines = lines != null ? new List<string>(lines) : new List<string>()
                };

            case ContentDetails cd:
                var blocksRaw = ContentDetailsBlocksField?.GetValue(cd) as List<ContentTextBlock>;
                var detailBlocks = new List<object>();
                if (blocksRaw != null)
                {
                    foreach (ContentTextBlock b in blocksRaw)
                    {
                        detailBlocks.Add(new
                        {
                            header = b.Header ?? string.Empty,
                            text = b.Text ?? string.Empty,
                            badge = b.Badgeify
                        });
                    }
                }
                var extraBtns = new List<object>();
                foreach (QMUIElement sub in GetSubElements(cd))
                {
                    if (sub is Button subBtn && !IsDeleted(subBtn) && !IsHidden(subBtn))
                    {
                        string subId = GetElementKey(subBtn);
                        _elementsById[subId] = subBtn;
                        bool subDisabled = GetBoolField(ElementDisabledField, subBtn)
                            && !IsSelfModerationButton(subBtn, out _, out _);
                        extraBtns.Add(new
                        {
                            id = subId,
                            label = subBtn.ButtonText ?? string.Empty,
                            icon_key = images.RequestUiLibIcon(NativeBridge.SurfaceQuick, modName, subBtn.ButtonIcon),
                            tooltip = subBtn.ButtonTooltip ?? string.Empty,
                            disabled = subDisabled
                        });
                    }
                }
                string cdIconKey = images.RequestUiLibIcon(NativeBridge.SurfaceQuick, modName, cd.ButtonIcon);
                return new
                {
                    id,
                    kind = "content_details",
                    tooltip = cd.ButtonTooltip ?? string.Empty,
                    icon_key = cdIconKey,
                    disabled = disabled || !cd.ButtonClickable,
                    detail_blocks = detailBlocks,
                    extra_buttons = extraBtns
                };

            case Spacer:
                return new { id, kind = "spacer" };

            case CustomElement:
                // Suppress raw CustomElement notice on CVR's Colour Picker since we render its preview natively.
                if (_navStack.Count > 0 && !string.IsNullOrEmpty(TryGetColourPickerPreviewHex(_navStack[_navStack.Count - 1])))
                    return null;
                return new { id, kind = "custom" };

            default:
                return null;
        }
    }

    private object BuildModalSnapshot()
    {
        if (string.IsNullOrEmpty(_modalKind))
            return null;

        return new
        {
            kind = _modalKind,
            title = _modalTitle,
            body = _modalBody,
            yes_text = _modalYesText,
            no_text = _modalNoText,
            ok_text = _modalOkText,
            options = _modalMultiSelect?.Options != null
                ? new List<string>(_modalMultiSelect.Options)
                : new List<string>(),
            selected_index = _modalMultiSelect?.SelectedOption ?? -1,
            number_value = _modalNumberValue,
            revision = _modalRevision
        };
    }

    private static bool IsTotallyWholesomeRootPage(Page page)
    {
        if (page == null || !GetBoolField(PageIsRootField, page))
            return false;

        string modName = PageModNameField?.GetValue(page) as string;
        string pageName = PageNameField?.GetValue(page) as string;
        return string.Equals(modName, TotallyWholesomeModName, StringComparison.OrdinalIgnoreCase)
            && string.Equals(pageName, TotallyWholesomeRootPageName, StringComparison.OrdinalIgnoreCase);
    }

    private string[] BuildTotallyWholesomeStatusDots()
    {
        var dots = new List<string>(4);
        if (_twPetAutoAccept) dots.Add("#00A801");
        if (_twMasterAutoAccept) dots.Add("#008FFF");
        if (_twVibDevice) dots.Add("#A855F7");
        if (_twShockDevice) dots.Add("#F59E0B");
        return dots.ToArray();
    }

    private object[] BuildTotallyWholesomeStatusBadges()
    {
        var badges = new List<object>(5);
        if (_twDisplayBadge && !string.IsNullOrWhiteSpace(_twBadgeText))
        {
            badges.Add(new
            {
                text = _twBadgeText,
                bg_hex = string.IsNullOrWhiteSpace(_twBadgeBg) ? "#4E00AF" : _twBadgeBg,
                fg_hex = string.IsNullOrWhiteSpace(_twBadgeFg) ? "#FFFFFF" : _twBadgeFg,
                icon = string.Empty
            });
        }
        if (_twPetAutoAccept)
        {
            badges.Add(new { text = "Pet Auto", bg_hex = "#00A801", fg_hex = "#FFFFFF", icon = string.Empty });
        }
        if (_twMasterAutoAccept)
        {
            badges.Add(new { text = "Master Auto", bg_hex = "#008FFF", fg_hex = "#FFFFFF", icon = string.Empty });
        }
        if (_twVibDevice)
        {
            badges.Add(new { text = "Vibe", bg_hex = "#7C3AED", fg_hex = "#FFFFFF", icon = "vibe" });
        }
        if (_twShockDevice)
        {
            badges.Add(new { text = "Shock", bg_hex = "#D97706", fg_hex = "#FFFFFF", icon = "bolt" });
        }
        return badges.ToArray();
    }

    private static string SanitizeModalHtml(string raw)
    {
        if (string.IsNullOrEmpty(raw))
            return string.Empty;

        if (raw.IndexOf('<') < 0 && raw.IndexOf('&') < 0)
            return raw;

        string text = HtmlBreakRegex.Replace(raw, "\n");
        text = HtmlParagraphCloseRegex.Replace(text, "\n\n");
        text = HtmlTagRegex.Replace(text, string.Empty);
        text = WebUtility.HtmlDecode(text);
        text = text.Replace("\r\n", "\n").Replace('\r', '\n');
        text = ExcessNewlinesRegex.Replace(text, "\n\n");
        return text.Trim();
    }

    private static List<Page> GetRootPages()
    {
        if (RootPagesField?.GetValue(null) is List<Page> list)
            return list;
        return new List<Page>();
    }

    private static List<QMUIElement> GetSubElements(QMUIElement el)
    {
        if (el != null && SubElementsField?.GetValue(el) is List<QMUIElement> list)
            return list;
        return new List<QMUIElement>();
    }

    private static string GetElementKey(QMUIElement el)
    {
        if (el == null) return string.Empty;
        if (!string.IsNullOrEmpty(el.ElementID)) return el.ElementID;
        if (!string.IsNullOrEmpty(el.UUID)) return el.UUID;
        return el.GetHashCode().ToString();
    }

    private static bool IsDeleted(QMUIElement el) => GetBoolField(ElementDeletedField, el);
    private static bool IsHidden(QMUIElement el) => el.Hidden || GetBoolField(ElementHiddenField, el);

    private static bool GetBoolField(FieldInfo f, object obj, bool defaultVal = false)
    {
        if (f == null || obj == null) return defaultVal;
        object v = f.GetValue(obj);
        return v is bool b ? b : defaultVal;
    }

    private static int GetIntField(FieldInfo f, object obj, int defaultVal = 0)
    {
        if (f == null || obj == null) return defaultVal;
        object v = f.GetValue(obj);
        return v is int i ? i : defaultVal;
    }
}
