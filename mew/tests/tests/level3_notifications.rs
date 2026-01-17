//! Level 3 - Notifications integration tests.
//!
//! These tests run against the notifications ontology for Watch & Subscriptions.
//! Watch features are not yet implemented, so tests expect parse errors.

use mew_tests::prelude::*;

mod crud {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("crud")
            .ontology("level-3/notifications/ontology.mew")
            .operations("level-3/notifications/operations/crud.mew")
            .step("test_create_broadcast", |a| a.created(1))
            .step("test_create_topic", |a| a.created(1))
            .step("test_create_direct", |a| a.created(1))
            .step("test_create_queue", |a| a.created(1))
            .step("test_create_subscriber_web", |a| a.created(1))
            .step("test_create_subscriber_mobile", |a| a.created(1))
            .step("test_create_subscriber_email", |a| a.created(1))
            .step("test_subscribe_web_to_broadcast", |a| a.linked(1))
            .step("test_subscribe_mobile_to_topic", |a| a.linked(1))
            .step("test_subscribe_with_filter", |a| a.linked(1))
            .step("test_create_urgent_message", |a| a.created(1).linked(1))
            .step("test_create_normal_message", |a| a.created(1).linked(1))
            .step("test_create_message_with_expiry", |a| a.created(1).linked(1))
            .step("test_create_consumer_group", |a| a.created(1))
            .step("test_link_group_to_queue", |a| a.linked(1))
            .step("test_query_urgent_messages", |a| a.rows_gte(1))
            .step("test_query_pending_deliveries", |a| a.rows_gte(0))
            .step("test_query_channel_subscribers", |a| a.rows_gte(1))
            .step("test_pause_subscriber", |a| a.modified(1))
            .step("test_activate_subscriber", |a| a.modified(1))
            .step("test_cleanup_messages", |a| a.deleted(3))
            .step("test_cleanup_deliveries", |a| a.deleted(0))
            .step("test_cleanup_subscribers", |a| a.deleted(3))
            .step("test_cleanup_groups", |a| a.deleted(1))
            .step("test_cleanup_channels", |a| a.deleted(4))
    }

    #[test]
    fn test_notifications_crud_operations() {
        scenario().run().unwrap();
    }
}

mod watch {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("watch")
            .ontology("level-3/notifications/ontology.mew")
            .operations("level-3/notifications/operations/watch.mew")
            // Setup operations should work
            .step("test_setup_broadcast_channel", |a| a.created(1))
            .step("test_setup_topic_channel", |a| a.created(1))
            .step("test_setup_subscriber_one", |a| a.created(1))
            .step("test_setup_subscriber_two", |a| a.created(1))
            .step("test_link_subscriptions", |a| a.linked(3))
            // WATCH statements expect parse errors (not yet implemented)
            .step("test_watch_basic", |a| a.error("parse"))
            .step("test_watch_with_filter", |a| a.error("parse"))
            .step("test_watch_with_mode_explicit", |a| a.error("parse"))
            .step("test_watch_ordering_arrival", |a| a.error("parse"))
            .step("test_watch_ordering_attribute", |a| a.error("parse"))
            .step("test_watch_ordering_causal", |a| a.error("parse"))
            .step("test_watch_initial_full", |a| a.error("parse"))
            .step("test_watch_initial_none", |a| a.error("parse"))
            .step("test_watch_initial_since", |a| a.error("parse"))
            .step("test_watch_tumbling_window", |a| a.error("parse"))
            .step("test_watch_sliding_window", |a| a.error("parse"))
            .step("test_watch_buffer_limit", |a| a.error("parse"))
            .step("test_watch_buffer_block", |a| a.error("parse"))
            .step("test_watch_buffer_error", |a| a.error("parse"))
            .step("test_watch_joined_pattern", |a| a.error("parse"))
            .step("test_watch_with_subscriber", |a| a.error("parse"))
            .step("test_cleanup", |a| a.deleted(4))
    }

    #[test]
    fn test_watch_subscription_operations() {
        scenario().run().unwrap();
    }
}

mod consume {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("consume")
            .ontology("level-3/notifications/ontology.mew")
            .operations("level-3/notifications/operations/consume.mew")
            // Setup should work
            .step("test_setup_queue_channel", |a| a.created(1))
            .step("test_setup_consumer_group", |a| a.created(1))
            .step("test_link_group_to_queue", |a| a.linked(1))
            .step("test_setup_worker_one", |a| a.created(1))
            .step("test_setup_worker_two", |a| a.created(1))
            .step("test_link_workers_to_group", |a| a.linked(4))
            // WATCH consume mode expects parse errors
            .step("test_watch_consume_basic", |a| a.error("parse"))
            .step("test_watch_consume_with_group", |a| a.error("parse"))
            .step("test_watch_best_effort", |a| a.error("parse"))
            .step("test_watch_reliable", |a| a.error("parse"))
            .step("test_watch_reliable_with_retries", |a| a.error("parse"))
            // ACK/NACK expect parse errors
            .step("test_publish_message_for_ack", |a| a.created(1).linked(1))
            .step("test_ack_delivery", |a| a.error("parse"))
            .step("test_nack_with_retry", |a| a.error("parse"))
            .step("test_nack_no_retry", |a| a.error("parse"))
            // More WATCH with dead letter
            .step("test_watch_with_dead_letter", |a| a.error("parse"))
            .step("test_query_dead_letters", |a| a.rows_gte(0))
            .step("test_watch_committed_visibility", |a| a.error("parse"))
            .step("test_watch_immediate_visibility", |a| a.error("parse"))
            .step("test_cleanup", |a| a.deleted(5))
    }

    #[test]
    fn test_consume_mode_operations() {
        scenario().run().unwrap();
    }
}

mod management {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("management")
            .ontology("level-3/notifications/ontology.mew")
            .operations("level-3/notifications/operations/management.mew")
            // Setup
            .step("test_setup_channel", |a| a.created(1))
            .step("test_setup_subscriber", |a| a.created(1))
            .step("test_link_subscription", |a| a.linked(1))
            // Create watch expects parse error
            .step("test_create_watch", |a| a.error("parse"))
            // Management operations expect parse errors
            .step("test_pause_watch", |a| a.error("parse"))
            .step("test_verify_paused_subscriber", |a| a.rows(1))
            .step("test_resume_watch", |a| a.error("parse"))
            .step("test_verify_resumed", |a| a.rows(1))
            .step("test_alter_buffer_size", |a| a.error("parse"))
            .step("test_alter_multiple_options", |a| a.error("parse"))
            .step("test_alter_ordering", |a| a.error("parse"))
            .step("test_alter_window", |a| a.error("parse"))
            .step("test_cancel_watch", |a| a.error("parse"))
            .step("test_verify_cancelled", |a| a.rows(1))
            // Error cases
            .step("test_pause_nonexistent", |a| a.error("parse"))
            .step("test_resume_not_paused", |a| a.error("parse"))
            .step("test_alter_cancelled", |a| a.error("parse"))
            .step("test_cleanup", |a| a.deleted(2))
    }

    #[test]
    fn test_watch_management_operations() {
        scenario().run().unwrap();
    }
}
