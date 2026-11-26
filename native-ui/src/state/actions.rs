use serde::Serialize;

/// Typed actions emitted by Rust UI and executed by C# on Unity's main thread.
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type")]
pub enum UiAction {
    SwitchToVanilla { target: String },
    TogglePerfLogging,
    ToggleLinearTexture,
    DummyPing { source: String, count: u32 },
    PlayUiSound { sound: String },
    ToggleMute,
    ToggleFlight,
    ToggleSeatedOrRecalibrate,
    ToggleFullBody,
    ToggleCamera,
    ToggleMirror,
    Respawn,
    GoHome,
    OpenTextChat,
    OpenMainMenuPage { page: String },
    OpenVanillaPage { page: String },
    OpenLocalPlayerQmPage,
    OpenPlayerInMainMenu,
    RemoveMyProps,
    TogglePropDeleteMode,
    RemoveAllProps,
    RemoveAllAvatars,
    RecalibrateAvatar,
    SwitchVrMode,
    ToggleCursorLock,
    MediaPrev,
    MediaPlayPause,
    MediaNext,
    QuitApplication,
    SetAudioVolume { key: String, value: i32 },
    NotificationButtonClick { button_id: String },
    NotificationDismiss { message_id: String },
    OpenVirtualKeyboard {
        surface_id: u32,
        field_id: String,
        initial_text: String,
        title: String,
        placeholder: String,
        max_len: i32,
        hidden: bool,
    },

    // UILib Mod Compatibility actions
    UiLibSelectTab { tab_id: String },
    UiLibGoBack,
    UiLibToggleCategory { category_id: String },
    UiLibClick { element_id: String },
    UiLibHold { element_id: String },
    UiLibToggle { element_id: String, value: bool },
    UiLibSlider { element_id: String, value: f32, reset: bool },
    UiLibTextInput { element_id: String },
    UiLibModalConfirm { accepted: bool },
    UiLibModalNoticeOk,
    UiLibModalSelectOption { index: i32 },
    UiLibModalNumberSubmit { value: f32 },
    UiLibModalDismiss,

    // UIExpansionKit Mod Settings actions
    RefreshUixSettings,
    UixSetBool {
        category: String,
        entry: String,
        value: bool,
    },
    UixSetNumber {
        category: String,
        entry: String,
        value: f64,
        save: bool,
    },
    UixSetString {
        category: String,
        entry: String,
        value: String,
    },
    UixSetEnum {
        category: String,
        entry: String,
        index: i32,
    },
    UixCustomButton {
        category: String,
        global_id: i32,
    },
    UixCustomToggle {
        category: String,
        global_id: i32,
        value: bool,
    },

    // Phase 3B Content Browser actions
    ContentLoadList {
        kind: String,
        category_key: String,
        is_system: bool,
        page: i32,
        sort_order: i32,
        ascending: bool,
    },
    ContentLoadMore {
        kind: String,
    },
    ContentLoadDetail {
        kind: String,
        id: String,
    },
    ContentCloseDetail,
    ContentGlobalSearch {
        query: String,
    },
    ContentWearAvatar {
        avatar_id: String,
    },
    ContentSpawnProp {
        prop_id: String,
    },
    ContentJoinInstance {
        instance_id: String,
    },
    ContentDropPortal {
        instance_id: String,
    },
    ContentDropCurrentInstancePortal,
    ContentLoadInstanceDetail {
        instance_id: String,
    },
    ContentCloseInstanceDetail,
    ContentToggleCategory {
        kind: String,
        id: String,
        category_key: String,
    },
    ContentCreateInstance {
        world_id: String,
        privacy: String,
        region: String,
        group_id: String,
    },
    ContentResolveGroupModPrompt {
        accept: bool,
        never_show: bool,
    },
    ContentSetHomeWorld {
        world_id: String,
    },
    ContentTogglePin {
        kind: String,
        id: String,
    },
    ContentUnpin {
        kind: String,
        id: String,
    },
    QmOpenWorldDetails {
        world_id: String,
    },

    // Phase 3C Social & User Details actions
    SocialRefreshFriends,
    SocialLoadUserDetail {
        user_id: String,
    },
    SocialSaveUserNote {
        user_id: String,
        note: String,
    },
    SocialUpdateOwnProfile {
        pronouns: String,
        bio: String,
        update_pronouns: bool,
        update_bio: bool,
    },
    SetColorizeMenu {
        enabled: bool,
    },
    SetPlayerColorPreset {
        preset_id: i32,
    },
    SetPlayerColorPrimary {
        swatch_id: i32,
    },
    SetPlayerColorSecondary {
        swatch_id: i32,
    },
    SetPlayerColorEmission {
        enabled: bool,
    },
    SocialCloseUserDetail,
    SocialInvitePlayer {
        user_id: String,
    },
    SocialRequestInvite {
        user_id: String,
    },
    SocialSendFriendRequest {
        user_id: String,
    },
    SocialUnfriend {
        user_id: String,
    },
    SocialSetBlocked {
        user_id: String,
        blocked: bool,
    },
    SocialSetPlayerMute {
        user_id: String,
        muted: bool,
    },
    SocialSetPlayerVolume {
        user_id: String,
        volume: i32,
    },
    SocialSetAvatarVisibility {
        user_id: String,
        mode: i32,
    },
    SocialSetPropVisibility {
        user_id: String,
        mode: i32,
    },
    SocialReloadPlayerAvatar {
        user_id: String,
    },
    SocialKickUser {
        user_id: String,
    },
    SocialSetFeaturedBadge {
        badge_id: String,
        featured: bool,
    },
    SocialSetFeaturedGroup {
        group_id: String,
    },
    SocialLoadGroupDetail {
        group_id: String,
    },
    SocialCloseGroupDetail,
    SocialUpdateGroupName {
        group_id: String,
        name: String,
    },
    SocialUpdateGroupDesc {
        group_id: String,
        description: String,
    },
    SocialKickGroupMember {
        group_id: String,
        target_id: String,
    },
    SocialLeaveGroup {
        group_id: String,
    },
    SocialDeleteGroup {
        group_id: String,
    },
    SocialLoadMoreGroupMembers {
        group_id: String,
    },
    SocialSetGroupMemberRole {
        group_id: String,
        user_id: String,
        role: String,
    },
    SocialMassSetGroupMembersToMember {
        group_id: String,
        user_ids: Vec<String>,
    },
    SocialCancelMassGroupRole,
    SocialUpdateGroupSettings {
        group_id: String,
        listed: bool,
        member_publicity: i32,
        event_publicity: i32,
        privacy_join: i32,
    },

    // Phase 3D Settings & Discover actions
    ContentLoadDiscover,
    ContentRefreshIfStale {
        kind: String,
    },
    SocialEnsureHomeFeeds,
    CvrSetBool {
        key: String,
        value: bool,
    },
    CvrSetInt {
        key: String,
        value: i32,
    },
    CvrSetFloat {
        key: String,
        value: f32,
    },
    CvrSetString {
        key: String,
        value: String,
    },
    ToggleUiClickSounds,
    SetUiSoundPack {
        pack: String,
    },
    SetEnableRecentHistory {
        enabled: bool,
    },
    SetQmModTabsIconOnly {
        enabled: bool,
    },
    SetDesktopScrollMultiplier {
        value: f32,
    },
    SetVrScrollMultiplier {
        value: f32,
    },
    ClearRecentHistory,
    ClearPinnedItems,
    ResetTileLayouts,
    OpenHardcodedUrl {
        key: String,
    },
    OpenExternalUrl {
        url: String,
    },
    CopyText {
        text: String,
    },
    AttemptEatBread,
    OpenLogsFolder,
    AcceptDisclaimer,
    CheckForUpdates,
    RemindUpdateLater,
    SkipUpdateVersion {
        version: String,
    },
    DismissWhatsNew,
    DismissSystemAlert {
        id: i64,
    },
    RespondSystemConfirm {
        id: i64,
        confirmed: bool,
    },
    SetMaxRefreshHz {
        hz: i32,
    },
    SetFontScale {
        scale: f32,
    },
    SetTileOrder {
        target: String,
        order: Vec<String>,
    },

    // Extension API actions
    ExtSelectTab {
        surface: String,
        tab_id: String,
    },
    ExtGoBack {
        surface: String,
    },
    ExtCloseModal {
        surface: String,
    },
    ExtEvent {
        ext_id: String,
        target_id: String,
        node_id: String,
        kind: String,
        string_val: String,
        float_val: f32,
        bool_val: bool,
    },

    // Advanced Avatar Settings & Preview actions
    AvatarSetParam {
        param: String,
        value: f32,
    },
    AvatarSetAutoApply {
        enabled: bool,
    },
    AvatarApplyStaged,
    AvatarDiscardStaged,
    AvatarLoadProfile {
        profile: String,
    },
    AvatarSaveProfile {
        profile: String,
    },
    AvatarDeleteProfile {
        profile: String,
    },
    AvatarReorderProfiles {
        profiles: Vec<String>,
    },
    AvatarResetDefaults,
    AvatarSetHeight {
        height: f32,
    },
    AvatarResetHeight,
    AvatarPreviewRotate {
        delta: f32,
        is_dragging: bool,
    },
    WorldPanoLook {
        delta_x: f32,
        delta_y: f32,
        is_dragging: bool,
    },
    SyncAvatarPreview {
        surface_id: u32,
        active: bool,
        target: String,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    },

    SyncStudioStage {
        target: String,
        local_x: f32,
        local_y: f32,
        height_units: f32,
        min_x: f32,
        max_x: f32,
        min_y: f32,
        max_y: f32,
    },
}
