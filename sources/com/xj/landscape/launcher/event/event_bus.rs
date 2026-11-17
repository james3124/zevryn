//! A minimal, dependency-free event bus modeling the `com.drake.channel`
//! broadcast bus used by GameHub Lite v5.1.8.
//!
//! The Android bus is a coroutine `BroadcastChannel` of `ChannelEvent(value,
//! tag)` objects with `sendEvent`, `receiveEvent`, `receiveEventLive` and
//! `receiveEventHandler` helpers. The coroutine / `LiveData` plumbing is out of
//! scope for this crate; what remains is the observable contract:
//!
//! * handlers register interest in a **topic** (here, an exact event class
//!   name) with a stable `handler_id`;
//! * [`EventBus::publish`] fans an event out to every subscriber of its topic;
//! * [`EventBus::publish_sticky`] additionally stores the last event per topic
//!   and replays it to handlers that subscribe later (mirroring the sticky
//!   behavior of `receiveEventLive`);
//! * every delivery is appended to a [`DispatchRecord`] log so tests can make
//!   assertions without real callbacks or threads.
//!
//! Only `std` collections are used. The bus is deliberately single-threaded and
//! deterministic: dispatch order follows subscription order.

use crate::launcher_event::LauncherEvent;
use std::collections::HashMap;

/// A handler's registration on a topic.
///
/// Returned by [`EventBus::subscribe`] and passed back to
/// [`EventBus::unsubscribe`]. `id` is a monotonically increasing token unique
/// within one bus instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subscription {
    pub id: u64,
    pub topic: String,
    pub handler_id: String,
}

/// One observed delivery, recorded in the bus dispatch log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DispatchRecord {
    pub subscription_id: u64,
    pub handler_id: String,
    pub topic: String,
    /// Exact class name of the delivered [`LauncherEvent`].
    pub event: String,
    /// `true` when this delivery is a sticky replay to a late subscriber.
    pub sticky: bool,
}

/// Result of a publish: how many handler deliveries were logged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PublishResult {
    pub deliveries: usize,
}

/// In-memory event bus with sticky storage and a dispatch log.
#[derive(Debug, Default)]
pub struct EventBus {
    next_id: u64,
    subscriptions: Vec<Subscription>,
    sticky: HashMap<String, LauncherEvent>,
    log: Vec<DispatchRecord>,
}

impl EventBus {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register `handler_id` for `topic` and return the subscription token.
    ///
    /// If a sticky event is already stored for `topic`, it is immediately
    /// replayed to the new handler and recorded with `sticky = true`.
    pub fn subscribe(&mut self, topic: &str, handler_id: &str) -> Subscription {
        let sub = Subscription {
            id: self.next_id,
            topic: topic.to_string(),
            handler_id: handler_id.to_string(),
        };
        self.next_id += 1;

        if let Some(event) = self.sticky.get(topic).cloned() {
            self.log_delivery(&sub, &event, true);
        }

        self.subscriptions.push(sub.clone());
        sub
    }

    /// Remove a subscription by token. Returns `true` if one was removed.
    pub fn unsubscribe(&mut self, subscription_id: u64) -> bool {
        let before = self.subscriptions.len();
        self.subscriptions.retain(|s| s.id != subscription_id);
        self.subscriptions.len() != before
    }

    /// Fan `event` out to all current subscribers of its topic.
    ///
    /// Deliveries are logged with `sticky = false` in subscription order.
    pub fn publish(&mut self, event: LauncherEvent) -> PublishResult {
        let topic = event.topic().to_string();
        let matching: Vec<Subscription> = self
            .subscriptions
            .iter()
            .filter(|s| s.topic == topic)
            .cloned()
            .collect();

        for sub in &matching {
            self.log_delivery(sub, &event, false);
        }

        PublishResult {
            deliveries: matching.len(),
        }
    }

    /// Store `event` as the sticky value for its topic, then publish it to
    /// current subscribers as a normal delivery.
    ///
    /// Late subscribers to the same topic receive a `sticky = true` replay.
    pub fn publish_sticky(&mut self, event: LauncherEvent) -> PublishResult {
        self.sticky.insert(event.topic().to_string(), event.clone());
        self.publish(event)
    }

    /// The last sticky event stored for `topic`, if any.
    pub fn sticky_event(&self, topic: &str) -> Option<&LauncherEvent> {
        self.sticky.get(topic)
    }

    /// Remove the sticky value for `topic` (so future subscribers get no replay).
    pub fn clear_sticky(&mut self, topic: &str) -> bool {
        self.sticky.remove(topic).is_some()
    }

    /// All current subscriptions, in registration order.
    pub fn subscriptions(&self) -> &[Subscription] {
        &self.subscriptions
    }

    /// Number of subscribers currently registered for `topic`.
    pub fn subscriber_count(&self, topic: &str) -> usize {
        self.subscriptions.iter().filter(|s| s.topic == topic).count()
    }

    /// The full dispatch log, in delivery order.
    pub fn dispatch_log(&self) -> &[DispatchRecord] {
        &self.log
    }

    /// Clear the dispatch log (sticky state and subscriptions are untouched).
    pub fn clear_log(&mut self) {
        self.log.clear();
    }

    fn log_delivery(&mut self, sub: &Subscription, event: &LauncherEvent, sticky: bool) {
        self.log.push(DispatchRecord {
            subscription_id: sub.id,
            handler_id: sub.handler_id.clone(),
            topic: sub.topic.clone(),
            event: event.name().to_string(),
            sticky,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn publish_reaches_only_matching_topic_subscribers() {
        let mut bus = EventBus::new();
        bus.subscribe("ClickCardItemEvent", "card_handler");
        bus.subscribe("DeviceStateChangeEvent", "device_handler");
        bus.subscribe("ClickCardItemEvent", "second_card_handler");

        let result = bus.publish(LauncherEvent::ClickCardItemEvent { page_id: 7 });
        assert_eq!(result.deliveries, 2);

        let log = bus.dispatch_log();
        assert_eq!(log.len(), 2);
        assert_eq!(log[0].handler_id, "card_handler");
        assert_eq!(log[1].handler_id, "second_card_handler");
        for record in log {
            assert_eq!(record.topic, "ClickCardItemEvent");
            assert_eq!(record.event, "ClickCardItemEvent");
            assert!(!record.sticky);
        }
    }

    #[test]
    fn unsubscribed_handler_is_not_notified_and_token_is_removed() {
        let mut bus = EventBus::new();
        let a = bus.subscribe("ClearFocusButtonEvent", "a");
        let b = bus.subscribe("ClearFocusButtonEvent", "b");

        assert!(bus.unsubscribe(a.id));
        assert!(!bus.unsubscribe(a.id)); // idempotent
        assert_eq!(bus.subscriber_count("ClearFocusButtonEvent"), 1);

        let result = bus.publish(LauncherEvent::ClearFocusButtonEvent);
        assert_eq!(result.deliveries, 1);
        assert_eq!(bus.dispatch_log()[0].handler_id, "b");
        assert_eq!(b.handler_id, "b");
    }

    #[test]
    fn sticky_event_is_stored_and_replayed_to_late_subscribers() {
        let mut bus = EventBus::new();

        // Existing handler gets a normal delivery.
        let early = bus.subscribe("DeviceStateChangeEvent", "early");
        let first = bus.publish_sticky(LauncherEvent::DeviceStateChangeEvent(true));
        assert_eq!(first.deliveries, 1);
        assert_eq!(bus.dispatch_log().len(), 1);
        assert!(!bus.dispatch_log()[0].sticky);

        // Late handler gets an immediate sticky replay on subscribe.
        let late = bus.subscribe("DeviceStateChangeEvent", "late");
        assert_eq!(bus.dispatch_log().len(), 2);
        let replay = &bus.dispatch_log()[1];
        assert_eq!(replay.subscription_id, late.id);
        assert_eq!(replay.handler_id, "late");
        assert!(replay.sticky);
        assert_eq!(replay.event, "DeviceStateChangeEvent");

        // Sticky storage is readable and the early handler is unaffected.
        assert_eq!(
            bus.sticky_event("DeviceStateChangeEvent"),
            Some(&LauncherEvent::DeviceStateChangeEvent(true))
        );
        assert_eq!(early.handler_id, "early");
    }

    #[test]
    fn publish_sticky_replaces_previous_value_per_topic() {
        let mut bus = EventBus::new();
        bus.publish_sticky(LauncherEvent::ChangeCategoryTagEvent(1));
        bus.publish_sticky(LauncherEvent::ChangeCategoryTagEvent(9));

        assert_eq!(
            bus.sticky_event("ChangeCategoryTagEvent"),
            Some(&LauncherEvent::ChangeCategoryTagEvent(9))
        );

        bus.subscribe("ChangeCategoryTagEvent", "late");
        assert_eq!(bus.dispatch_log().len(), 1);
        assert!(bus.dispatch_log()[0].sticky);
    }

    #[test]
    fn clear_sticky_stops_future_replays() {
        let mut bus = EventBus::new();
        bus.publish_sticky(LauncherEvent::QuitOneKeyLoginPageEvent);
        assert!(bus.clear_sticky("QuitOneKeyLoginPageEvent"));
        assert!(!bus.clear_sticky("QuitOneKeyLoginPageEvent"));

        bus.subscribe("QuitOneKeyLoginPageEvent", "late");
        assert!(bus.dispatch_log().is_empty());
        assert!(bus.sticky_event("QuitOneKeyLoginPageEvent").is_none());
    }

    #[test]
    fn log_can_be_cleared_without_dropping_state() {
        let mut bus = EventBus::new();
        let sub = bus.subscribe("SearchKeyWordEvent", "search");
        bus.publish(LauncherEvent::SearchKeyWordEvent("abc".into()));
        assert_eq!(bus.dispatch_log().len(), 1);

        bus.clear_log();
        assert!(bus.dispatch_log().is_empty());
        assert_eq!(bus.subscriptions().len(), 1);

        bus.publish(LauncherEvent::SearchKeyWordEvent("def".into()));
        assert_eq!(bus.dispatch_log().len(), 1);
        assert_eq!(bus.dispatch_log()[0].subscription_id, sub.id);
    }
}
