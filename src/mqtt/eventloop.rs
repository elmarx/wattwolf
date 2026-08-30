use crate::mqtt::event_handler::MqttEventHandler;
use rumqttc::v5::{Connection, Event, Incoming};
use std::time::Duration;
use tracing::error;

pub fn spawn(connection: Connection, handler: impl MqttEventHandler + 'static) {
    std::thread::spawn(move || {
        handle_connection(connection, &handler);
    });
}

fn handle_connection<H: MqttEventHandler>(mut connection: Connection, handler: &H) {
    const INITIAL_BACKOFF: Duration = Duration::from_millis(100);
    const MAX_BACKOFF: Duration = Duration::from_mins(1);
    const BACKOFF_MULTIPLIER: u32 = 2;

    let mut current_backoff = INITIAL_BACKOFF;

    for notification in connection.iter() {
        match notification {
            Err(e) => {
                error!(error = %e, backoff_ms = current_backoff.as_millis(), "MQTT connection error, backing off");

                std::thread::sleep(current_backoff);

                // Exponentially increase backoff, capped at MAX_BACKOFF
                current_backoff = std::cmp::min(
                    current_backoff.saturating_mul(BACKOFF_MULTIPLIER),
                    MAX_BACKOFF,
                );
            }
            Ok(Event::Incoming(Incoming::ConnAck(_))) if INITIAL_BACKOFF < current_backoff => {
                // Reset backoff on successful notification
                current_backoff = INITIAL_BACKOFF;

                handler.on_mqtt_connection_online();
            }
            // this should be the initial "connection up"
            Ok(Event::Incoming(Incoming::ConnAck(_))) => {
                handler.on_mqtt_connection_online();
            }
            Ok(_) => {}
        }
    }
}
