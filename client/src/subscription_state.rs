use std::collections::HashMap;

use opcua_types::service_types::DataChangeNotification;

use crate::subscription::*;

/// Holds the live subscription state
pub struct SubscriptionState {
    /// Subscriptions (key = subscription_id)
    subscriptions: HashMap<u32, Subscription>,
}

impl SubscriptionState {
    pub fn new() -> SubscriptionState {
        SubscriptionState {
            subscriptions: HashMap::new(),
        }
    }

    pub(crate) fn drain_subscriptions(&mut self) -> HashMap<u32, Subscription> {
        self.subscriptions.drain().collect()
    }

    pub fn subscription_ids(&self) -> Option<Vec<u32>> {
        if self.subscriptions.is_empty() {
            None
        } else {
            Some(self.subscriptions.keys().map(|k| *k).collect())
        }
    }

    pub fn subscription_exists(&self, subscription_id: u32) -> bool {
        self.subscriptions.contains_key(&subscription_id)
    }

    pub fn get(&self, subscription_id: u32) -> Option<&Subscription> {
        self.subscriptions.get(&subscription_id)
    }

    pub(crate) fn add_subscription(&mut self, subscription: Subscription) {
        self.subscriptions.insert(subscription.subscription_id(), subscription);
    }

    pub(crate) fn modify_subscription(&mut self, subscription_id: u32, publishing_interval: f64, lifetime_count: u32, max_keep_alive_count: u32, max_notifications_per_publish: u32, priority: u8) {
        if let Some(ref mut subscription) = self.subscriptions.get_mut(&subscription_id) {
            subscription.set_publishing_interval(publishing_interval);
            subscription.set_lifetime_count(lifetime_count);
            subscription.set_max_keep_alive_count(max_keep_alive_count);
            subscription.set_max_notifications_per_publish(max_notifications_per_publish);
            subscription.set_priority(priority);
        }
    }

    pub(crate) fn delete_subscription(&mut self, subscription_id: u32) -> Option<Subscription> {
        self.subscriptions.remove(&subscription_id)
    }

    pub(crate) fn set_publishing_mode(&mut self, subscription_ids: &[u32], publishing_enabled: bool) {
        subscription_ids.iter().for_each(|subscription_id| {
            if let Some(ref mut subscription) = self.subscriptions.get_mut(subscription_id) {
                subscription.set_publishing_enabled(publishing_enabled);
            }
        });
    }

    pub(crate) fn subscription_data_change(&mut self, subscription_id: u32, data_change_notifications: &[DataChangeNotification]) {
        if let Some(ref mut subscription) = self.subscriptions.get_mut(&subscription_id) {
            subscription.data_change(data_change_notifications);
        }
    }

    pub(crate) fn insert_monitored_items(&mut self, subscription_id: u32, items_to_create: &[CreateMonitoredItem]) {
        if let Some(ref mut subscription) = self.subscriptions.get_mut(&subscription_id) {
            subscription.insert_monitored_items(items_to_create);
        }
    }

    pub(crate) fn modify_monitored_items(&mut self, subscription_id: u32, items_to_modify: &[ModifyMonitoredItem]) {
        if let Some(ref mut subscription) = self.subscriptions.get_mut(&subscription_id) {
            subscription.modify_monitored_items(items_to_modify);
        }
    }

    pub(crate) fn delete_monitored_items(&mut self, subscription_id: u32, items_to_delete: &[u32]) {
        if let Some(ref mut subscription) = self.subscriptions.get_mut(&subscription_id) {
            subscription.delete_monitored_items(items_to_delete);
        }
    }
}
