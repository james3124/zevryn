//! Rust model of the launcher-side event family used by GameHub Lite v5.1.8.
//!
//! The reference app posts plain Kotlin objects through the `com.drake.channel`
//! broadcast bus (`ChannelKt.sendEvent` / `receiveEvent` / `receiveEventLive`).
//! Each event "topic" in that bus is the concrete class name of the object.
//! This module models a representative set of those classes as a single
//! `LauncherEvent` enum whose variants keep the **exact Java class name** so
//! that a topic is always recoverable with [`LauncherEvent::name`].
//!
//! Payloads mirror only the primitive (`int` / `boolean` / `String`) fields of
//! the decompiled classes. Events whose fields are Android/Kotlin objects
//! (`View`, `CardItemData`, `MyGamesItemData`, `GameStartupParams`, ...) are
//! modeled as unit variants and documented; the Android objects themselves are
//! out of scope for this dependency-free crate.

/// The coarse family a [`LauncherEvent`] belongs to.
///
/// Mirrors the way the reference source groups events by the screen/feature
/// that listens for them (card scrolling, focus management, notifications,
/// ...). The `Channel` bus itself is untyped: families exist only to make the
/// event set navigable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventFamily {
    /// Card clicks, list scrolling and per-item callbacks.
    CardScroll,
    /// Focus requests and focus-loss notifications.
    Focus,
    /// Page navigation and tab switching.
    Navigation,
    /// Device connection / emulator state.
    Device,
    /// In-app notification badge events.
    Notification,
    /// Search box and keyword events.
    Search,
    /// Game launch and download requests.
    GameLaunch,
    /// Background/theme changes for the main pages.
    PageAppearance,
    /// Video upload, playback and like events.
    Video,
    /// Events without a tighter grouping.
    Misc,
}

/// A representative, well-identified launcher event.
///
/// Variant names are the exact decompiled class names (including the two
/// historical spellings `MainRequestFocuse` and `closeNoticePageEvent`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LauncherEvent {
    // ---- card / click / scroll -------------------------------------------
    /// `ClickCardItemEvent(card: HomeInfoEntity.CardInfo, pageId: Int, view: View?)`
    ClickCardItemEvent { page_id: i32 },
    /// `ClickTopCardItemEvent(card: TopRecommendEntity, pageId: Int, view: View?)`
    ClickTopCardItemEvent { page_id: i32 },
    /// `ScrollCardItemEvent(card: HomeInfoEntity.CardInfo, pageId: Int, view: View?)`
    ScrollCardItemEvent { page_id: i32 },
    /// `ScrollCardTopItemEvent(card: TopRecommendEntity, pageId: Int, view: View?)`
    ScrollCardTopItemEvent { page_id: i32 },
    /// `ScrollCardBackPathEvent(path, i3, z3, i4)`
    ScrollCardBackPathEvent {
        path: String,
        page_id: i32,
        success: bool,
        direction: i32,
    },
    /// `ScrollCardToTypeEvent` (marker)
    ScrollCardToTypeEvent,
    /// `OnScrollListenerEvent(i3)` — scroll offset / state.
    OnScrollListenerEvent(i32),
    /// `ScrollToNewFriend(i3)`
    ScrollToNewFriend(i32),
    /// `ScrollToNewVideo(i3)`
    ScrollToNewVideo(i32),

    // ---- focus ------------------------------------------------------------
    /// `MainRequestFocuse(focuse: Boolean)`
    MainRequestFocuse(bool),
    /// `ClearFocusButtonEvent` (marker)
    ClearFocusButtonEvent,
    /// `FocusSearchDisabledEvent(z3: Boolean)`
    FocusSearchDisabledEvent(bool),
    /// `MainListFocusedChange(z3: Boolean, i3: Int)`
    MainListFocusedChange { focused: bool, position: i32 },
    /// `NavDialogVisibleChangedEvent(z3: Boolean)`
    NavDialogVisibleChangedEvent(bool),

    // ---- navigation / pages ----------------------------------------------
    /// `Navigate2ImportPcGameFileEvent` (marker)
    Navigate2ImportPcGameFileEvent,
    /// `ShowMainPage(i3)`
    ShowMainPage(i32),
    /// `SendSwitchMainPage(z3: Boolean)`
    SendSwitchMainPage(bool),
    /// `ToggleTabEvent(i3)`
    ToggleTabEvent(i32),
    /// `QuitOneKeyLoginPageEvent` (marker)
    QuitOneKeyLoginPageEvent,
    /// `MainGuideEvent` (marker)
    MainGuideEvent,

    // ---- device -----------------------------------------------------------
    /// `DeviceStateChangeEvent(z3: Boolean)`
    DeviceStateChangeEvent(bool),
    /// `DeviceMenuSwitchEvent(i3)`
    DeviceMenuSwitchEvent(i32),
    /// `ClearSimulatorDataEvent` (marker)
    ClearSimulatorDataEvent,
    /// `SendDeleteDeviceEvent` (marker)
    SendDeleteDeviceEvent,
    /// `ReSetMappingEvent` (marker)
    ReSetMappingEvent,

    // ---- notifications ----------------------------------------------------
    /// `BlockNotificationEvent(i3)`
    BlockNotificationEvent(i32),
    /// `FriendNotificationEvent(i3)`
    FriendNotificationEvent(i32),
    /// `HightLightNotificationEvent(i3)` (original spelling)
    HightLightNotificationEvent(i32),
    /// `OpenNotificationEvent` (marker)
    OpenNotificationEvent,
    /// `OtherNotificationEvent(i3)`
    OtherNotificationEvent(i32),
    /// `PushNotificationEvent(i3)`
    PushNotificationEvent(i32),
    /// `closeNoticePageEvent` (original lowercase spelling).
    #[allow(non_camel_case_types)]
    closeNoticePageEvent,

    // ---- search -----------------------------------------------------------
    /// `SearchKeyWordEvent(key: String)`
    SearchKeyWordEvent(String),
    /// `RemoveSearchKeyWordEvent(key: String)`
    RemoveSearchKeyWordEvent(String),
    /// `SearchItemSelectEvent(i3)`
    SearchItemSelectEvent(i32),

    // ---- game launch ------------------------------------------------------
    /// `SendStartGameEvent(str, openType)` — package id + launch type.
    SendStartGameEvent {
        package: String,
        open_type: String,
    },
    /// `SendDownLoadGameEvent(str, channel)` — game id + source channel.
    SendDownLoadGameEvent {
        game_id: String,
        channel: String,
    },
    /// `SelectPcGameVersionEvent(gameStartupParams: GameStartupParams)`
    SelectPcGameVersionEvent,
    /// `SendClickStartGameEvent(gameStartupParams: GameStartupParams)`
    SendClickStartGameEvent,

    // ---- page appearance / data ------------------------------------------
    /// `ChangeMyPageDataEvent(cardItemData: MyGamesItemData)`
    ChangeMyPageDataEvent,
    /// `ChangeCategoryTagEvent(id: Int)`
    ChangeCategoryTagEvent(i32),
    /// `ChangeLaunchTypeEvent(validId: String)`
    ChangeLaunchTypeEvent(String),

    // ---- video ------------------------------------------------------------
    /// `UploadVideoEvent` (`Serializable`, fields are nullable strings).
    UploadVideoEvent,
    /// `SendClickVideoItemEvent(video: GameVideo)`
    SendClickVideoItemEvent,
}

impl LauncherEvent {
    /// The exact Java class name — used as the bus topic.
    pub fn name(&self) -> &'static str {
        match self {
            Self::ClickCardItemEvent { .. } => "ClickCardItemEvent",
            Self::ClickTopCardItemEvent { .. } => "ClickTopCardItemEvent",
            Self::ScrollCardItemEvent { .. } => "ScrollCardItemEvent",
            Self::ScrollCardTopItemEvent { .. } => "ScrollCardTopItemEvent",
            Self::ScrollCardBackPathEvent { .. } => "ScrollCardBackPathEvent",
            Self::ScrollCardToTypeEvent => "ScrollCardToTypeEvent",
            Self::OnScrollListenerEvent(_) => "OnScrollListenerEvent",
            Self::ScrollToNewFriend(_) => "ScrollToNewFriend",
            Self::ScrollToNewVideo(_) => "ScrollToNewVideo",
            Self::MainRequestFocuse(_) => "MainRequestFocuse",
            Self::ClearFocusButtonEvent => "ClearFocusButtonEvent",
            Self::FocusSearchDisabledEvent(_) => "FocusSearchDisabledEvent",
            Self::MainListFocusedChange { .. } => "MainListFocusedChange",
            Self::NavDialogVisibleChangedEvent(_) => "NavDialogVisibleChangedEvent",
            Self::Navigate2ImportPcGameFileEvent => "Navigate2ImportPcGameFileEvent",
            Self::ShowMainPage(_) => "ShowMainPage",
            Self::SendSwitchMainPage(_) => "SendSwitchMainPage",
            Self::ToggleTabEvent(_) => "ToggleTabEvent",
            Self::QuitOneKeyLoginPageEvent => "QuitOneKeyLoginPageEvent",
            Self::MainGuideEvent => "MainGuideEvent",
            Self::DeviceStateChangeEvent(_) => "DeviceStateChangeEvent",
            Self::DeviceMenuSwitchEvent(_) => "DeviceMenuSwitchEvent",
            Self::ClearSimulatorDataEvent => "ClearSimulatorDataEvent",
            Self::SendDeleteDeviceEvent => "SendDeleteDeviceEvent",
            Self::ReSetMappingEvent => "ReSetMappingEvent",
            Self::BlockNotificationEvent(_) => "BlockNotificationEvent",
            Self::FriendNotificationEvent(_) => "FriendNotificationEvent",
            Self::HightLightNotificationEvent(_) => "HightLightNotificationEvent",
            Self::OpenNotificationEvent => "OpenNotificationEvent",
            Self::OtherNotificationEvent(_) => "OtherNotificationEvent",
            Self::PushNotificationEvent(_) => "PushNotificationEvent",
            Self::closeNoticePageEvent => "closeNoticePageEvent",
            Self::SearchKeyWordEvent(_) => "SearchKeyWordEvent",
            Self::RemoveSearchKeyWordEvent(_) => "RemoveSearchKeyWordEvent",
            Self::SearchItemSelectEvent(_) => "SearchItemSelectEvent",
            Self::SendStartGameEvent { .. } => "SendStartGameEvent",
            Self::SendDownLoadGameEvent { .. } => "SendDownLoadGameEvent",
            Self::SelectPcGameVersionEvent => "SelectPcGameVersionEvent",
            Self::SendClickStartGameEvent => "SendClickStartGameEvent",
            Self::ChangeMyPageDataEvent => "ChangeMyPageDataEvent",
            Self::ChangeCategoryTagEvent(_) => "ChangeCategoryTagEvent",
            Self::ChangeLaunchTypeEvent(_) => "ChangeLaunchTypeEvent",
            Self::UploadVideoEvent => "UploadVideoEvent",
            Self::SendClickVideoItemEvent => "SendClickVideoItemEvent",
        }
    }

    /// Alias for [`LauncherEvent::name`] to make bus-topic intent explicit.
    pub fn topic(&self) -> &'static str {
        self.name()
    }

    /// The family this event belongs to.
    pub fn family(&self) -> EventFamily {
        match self {
            Self::ClickCardItemEvent { .. }
            | Self::ClickTopCardItemEvent { .. }
            | Self::ScrollCardItemEvent { .. }
            | Self::ScrollCardTopItemEvent { .. }
            | Self::ScrollCardBackPathEvent { .. }
            | Self::ScrollCardToTypeEvent
            | Self::OnScrollListenerEvent(_)
            | Self::ScrollToNewFriend(_)
            | Self::ScrollToNewVideo(_) => EventFamily::CardScroll,

            Self::MainRequestFocuse(_)
            | Self::ClearFocusButtonEvent
            | Self::FocusSearchDisabledEvent(_)
            | Self::MainListFocusedChange { .. }
            | Self::NavDialogVisibleChangedEvent(_) => EventFamily::Focus,

            Self::Navigate2ImportPcGameFileEvent
            | Self::ShowMainPage(_)
            | Self::SendSwitchMainPage(_)
            | Self::ToggleTabEvent(_)
            | Self::QuitOneKeyLoginPageEvent
            | Self::MainGuideEvent => EventFamily::Navigation,

            Self::DeviceStateChangeEvent(_)
            | Self::DeviceMenuSwitchEvent(_)
            | Self::ClearSimulatorDataEvent
            | Self::SendDeleteDeviceEvent
            | Self::ReSetMappingEvent => EventFamily::Device,

            Self::BlockNotificationEvent(_)
            | Self::FriendNotificationEvent(_)
            | Self::HightLightNotificationEvent(_)
            | Self::OpenNotificationEvent
            | Self::OtherNotificationEvent(_)
            | Self::PushNotificationEvent(_)
            | Self::closeNoticePageEvent => EventFamily::Notification,

            Self::SearchKeyWordEvent(_)
            | Self::RemoveSearchKeyWordEvent(_)
            | Self::SearchItemSelectEvent(_) => EventFamily::Search,

            Self::SendStartGameEvent { .. }
            | Self::SendDownLoadGameEvent { .. }
            | Self::SelectPcGameVersionEvent
            | Self::SendClickStartGameEvent => EventFamily::GameLaunch,

            Self::ChangeMyPageDataEvent
            | Self::ChangeCategoryTagEvent(_)
            | Self::ChangeLaunchTypeEvent(_) => EventFamily::PageAppearance,

            Self::UploadVideoEvent | Self::SendClickVideoItemEvent => EventFamily::Video,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_the_exact_decompiled_class_name() {
        assert_eq!(
            LauncherEvent::ChangeMyPageDataEvent.name(),
            "ChangeMyPageDataEvent"
        );
        assert_eq!(
            LauncherEvent::Navigate2ImportPcGameFileEvent.name(),
            "Navigate2ImportPcGameFileEvent"
        );
        assert_eq!(
            LauncherEvent::MainRequestFocuse(false).name(),
            "MainRequestFocuse"
        );
        assert_eq!(
            LauncherEvent::closeNoticePageEvent.name(),
            "closeNoticePageEvent"
        );
        assert_eq!(
            LauncherEvent::HightLightNotificationEvent(1).name(),
            "HightLightNotificationEvent"
        );
    }

    #[test]
    fn topic_and_name_agree() {
        let event = LauncherEvent::ClickCardItemEvent { page_id: 3 };
        assert_eq!(event.topic(), event.name());
    }

    #[test]
    fn family_groups_related_events() {
        assert_eq!(
            LauncherEvent::ScrollCardItemEvent { page_id: 0 }.family(),
            EventFamily::CardScroll
        );
        assert_eq!(
            LauncherEvent::ClearFocusButtonEvent.family(),
            EventFamily::Focus
        );
        assert_eq!(
            LauncherEvent::DeviceStateChangeEvent(true).family(),
            EventFamily::Device
        );
    }

    #[test]
    fn every_variant_has_a_unique_name() {
        use std::collections::HashSet;
        let events = sample_events();
        let names: HashSet<&str> = events.iter().map(|e| e.name()).collect();
        assert_eq!(names.len(), events.len());
    }

    /// One instance of each variant, used by bus tests too.
    pub fn sample_events() -> Vec<LauncherEvent> {
        vec![
            LauncherEvent::ClickCardItemEvent { page_id: 0 },
            LauncherEvent::ClickTopCardItemEvent { page_id: 0 },
            LauncherEvent::ScrollCardItemEvent { page_id: 0 },
            LauncherEvent::ScrollCardTopItemEvent { page_id: 0 },
            LauncherEvent::ScrollCardBackPathEvent {
                path: "p".into(),
                page_id: 0,
                success: true,
                direction: 1,
            },
            LauncherEvent::ScrollCardToTypeEvent,
            LauncherEvent::OnScrollListenerEvent(0),
            LauncherEvent::ScrollToNewFriend(0),
            LauncherEvent::ScrollToNewVideo(0),
            LauncherEvent::MainRequestFocuse(false),
            LauncherEvent::ClearFocusButtonEvent,
            LauncherEvent::FocusSearchDisabledEvent(false),
            LauncherEvent::MainListFocusedChange {
                focused: false,
                position: 0,
            },
            LauncherEvent::NavDialogVisibleChangedEvent(false),
            LauncherEvent::Navigate2ImportPcGameFileEvent,
            LauncherEvent::ShowMainPage(0),
            LauncherEvent::SendSwitchMainPage(false),
            LauncherEvent::ToggleTabEvent(0),
            LauncherEvent::QuitOneKeyLoginPageEvent,
            LauncherEvent::MainGuideEvent,
            LauncherEvent::DeviceStateChangeEvent(false),
            LauncherEvent::DeviceMenuSwitchEvent(0),
            LauncherEvent::ClearSimulatorDataEvent,
            LauncherEvent::SendDeleteDeviceEvent,
            LauncherEvent::ReSetMappingEvent,
            LauncherEvent::BlockNotificationEvent(0),
            LauncherEvent::FriendNotificationEvent(0),
            LauncherEvent::HightLightNotificationEvent(0),
            LauncherEvent::OpenNotificationEvent,
            LauncherEvent::OtherNotificationEvent(0),
            LauncherEvent::PushNotificationEvent(0),
            LauncherEvent::closeNoticePageEvent,
            LauncherEvent::SearchKeyWordEvent("k".into()),
            LauncherEvent::RemoveSearchKeyWordEvent("k".into()),
            LauncherEvent::SearchItemSelectEvent(0),
            LauncherEvent::SendStartGameEvent {
                package: "p".into(),
                open_type: "o".into(),
            },
            LauncherEvent::SendDownLoadGameEvent {
                game_id: "g".into(),
                channel: "c".into(),
            },
            LauncherEvent::SelectPcGameVersionEvent,
            LauncherEvent::SendClickStartGameEvent,
            LauncherEvent::ChangeMyPageDataEvent,
            LauncherEvent::ChangeCategoryTagEvent(0),
            LauncherEvent::ChangeLaunchTypeEvent("v".into()),
            LauncherEvent::UploadVideoEvent,
            LauncherEvent::SendClickVideoItemEvent,
        ]
    }
}
