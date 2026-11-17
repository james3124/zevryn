# Launcher event package — Rust reimplementation report

## Scope

Rust model of the launcher event family used by GameHub Lite v5.1.8. Only
dependency-free, portable logic is implemented: event identity (exact Java class
names), coarse family grouping, and a synchronous broadcast bus with sticky
delivery and a testable dispatch log. No external crates — only `std`
(`HashMap`, `Vec`, `String`).

## Files written (write area: `app/sources/com/xj/landscape/launcher/event/`)

- `mod.rs` — module declarations and re-exports (`LauncherEvent`, `EventFamily`,
  `EventBus`, `Subscription`, `DispatchRecord`, `PublishResult`).
- `launcher_event.rs` — `LauncherEvent` enum (one variant per modeled class,
  name equal to the exact decompiled class name) and `EventFamily` grouping.
  Each variant exposes `name()` / `topic()` and `family()`.
- `event_bus.rs` — `EventBus` with `subscribe(topic, handler_id)`, `unsubscribe`,
  `publish`, `publish_sticky`, sticky storage, and `DispatchRecord` log.
- `REPORT.md` — this file.

## Event names covered (exact decompiled class names, 44 variants)

Card / click / scroll family:
`ClickCardItemEvent`, `ClickTopCardItemEvent`, `ScrollCardItemEvent`,
`ScrollCardTopItemEvent`, `ScrollCardBackPathEvent`, `ScrollCardToTypeEvent`,
`OnScrollListenerEvent`, `ScrollToNewFriend`, `ScrollToNewVideo`.

Focus family:
`MainRequestFocuse`, `ClearFocusButtonEvent`, `FocusSearchDisabledEvent`,
`MainListFocusedChange`, `NavDialogVisibleChangedEvent`.

Navigation / page family:
`Navigate2ImportPcGameFileEvent`, `ShowMainPage`, `SendSwitchMainPage`,
`ToggleTabEvent`, `QuitOneKeyLoginPageEvent`, `MainGuideEvent`.

Device family:
`DeviceStateChangeEvent`, `DeviceMenuSwitchEvent`, `ClearSimulatorDataEvent`,
`SendDeleteDeviceEvent`, `ReSetMappingEvent`.

Notification family:
`BlockNotificationEvent`, `FriendNotificationEvent`,
`HightLightNotificationEvent`, `OpenNotificationEvent`, `OtherNotificationEvent`,
`PushNotificationEvent`, `closeNoticePageEvent`.

Search family:
`SearchKeyWordEvent`, `RemoveSearchKeyWordEvent`, `SearchItemSelectEvent`.

Game launch family:
`SendStartGameEvent`, `SendDownLoadGameEvent`, `SelectPcGameVersionEvent`,
`SendClickStartGameEvent`.

Page appearance / data family:
`ChangeMyPageDataEvent`, `ChangeCategoryTagEvent`, `ChangeLaunchTypeEvent`.

Video family:
`UploadVideoEvent`, `SendClickVideoItemEvent`.

Payload fidelity: variants carry only primitive decompiled fields (`int` →
`i32`, `boolean` → `bool`, `String` → `String`). Events whose fields are
Android/Kotlin objects (`View`, `CardItemData`, `MyGamesItemData`,
`HomeInfoEntity.CardInfo`, `TopRecommendEntity`, `GameStartupParams`,
`GameVideo`, `List`, nullable strings in `UploadVideoEvent`) are unit variants;
the original field signatures are preserved in doc comments on each variant.

## Bus model

The reference app posts events through `com.drake.channel.ChannelKt`
(`sources/com/drake/channel/ChannelKt.java`): a `BroadcastChannel` of
`ChannelEvent(value, tag)` objects with `sendEvent(Object, String tag)` and
`receiveEvent` / `receiveEventLive` / `receiveEventHandler` / `receiveTag`
helpers. Topic == concrete event class name.

Modeled here:
- `subscribe(topic, handler_id)` registers a handler and returns a unique
  `Subscription` token; subscriptions are ordered.
- `publish(event)` fans out to all current subscribers of `event.topic()`;
  each delivery is a `DispatchRecord` (no real callbacks — the log is the
  observable effect).
- `publish_sticky(event)` stores the last event per topic and publishes it;
  a handler subscribing later gets an immediate `sticky = true` replay
  (mirrors the sticky behavior of `receiveEventLive` / `receiveEvent` channels).
- `unsubscribe(token)`, `clear_sticky`, `clear_log`, `sticky_event`,
  `subscriber_count`, `subscriptions` are provided for tests/introspection.

## Sources consulted (read-only reference)

- `reference/gamehub-lite-v518-decompiled/.../sources/com/xj/landscape/launcher/event/*.java`
  — all 80 decompiled event classes (field/constructor inventory and the
  representative payloads listed above).
- `sources/com/drake/channel/ChannelKt.java`, `ChannelEvent.java`,
  `ChannelKt$receiveEvent$1.java`, `ChannelKt$receiveEventLive$1.java` —
  broadcast bus and sticky/LiveData receive helpers.
- `sources/com/xj/common/event/*.java`, `sources/com/xj/module_pcstream/event/*.java`,
  `sources/com/xj/pay/event/*.java` — adjacent event packages surveyed for
  naming conventions; not modeled here (belong to other areas/agents).
- Existing project modules under `app/sources/com/xj/common/` — module layout,
  doc-comment style, and `#[cfg(test)]` conventions.

## Out of scope

- `com.drake.channel` coroutine `BroadcastChannel`, `ChannelScope`, `Job`,
  `sendEvent` and the `receiveEventHandler` suspend lambdas (no coroutine
  runtime in this crate).
- `androidx.lifecycle.LiveData` / `receiveEventLive` lifecycle ownership.
- `View`-bearing and other Android object payloads; `Serializable`
  `UploadVideoEvent` nullable-string fields.
- Adjacent event packages (`com.xj.common.event`, `com.xj.module_pcstream.event`,
  `com.xj.pay.event`, ARIA/wifi/third-party events) and the greenrobot/Guava
  `EventBus` classes present in the APK but unrelated to launcher events.
- Thread safety, priority/ordered broadcasting, tag filtering beyond topic,
  and actual handler callback execution — replaced by the deterministic
  dispatch log.

## Tests

`#[cfg(test)]` tests in `launcher_event.rs` and `event_bus.rs` cover:
class-name/topic correctness, family grouping, unique names, publish/subscribe
fan-out, unsubscribe, sticky store + late-subscriber replay, sticky replacement,
sticky clearing, and log clearing without state loss. Run with `cargo test`.
