use crate::config::MqttConfig;
use crate::error::WattwolfError;
use rumqttc::{Client, Connection, MqttOptions, QoS};
use std::cell::RefCell;
use std::time::Duration;

pub struct MqttPublisher {
    client: Client,
    topic_prefix: String,
    connection: RefCell<Option<Connection>>,
}

impl MqttPublisher {
    pub fn new(config: &MqttConfig) -> Self {
        let mut mqtt_options = MqttOptions::new("wattwolf", &config.host, config.port);
        mqtt_options.set_keep_alive(Duration::from_secs(30));

        if let (Some(username), Some(password)) = (&config.username, &config.password) {
            mqtt_options.set_credentials(username, password);
        }

        let (client, connection) = Client::new(mqtt_options, 10);

        Self {
            client,
            topic_prefix: config.topic_prefix.clone(),
            connection: RefCell::new(Some(connection)),
        }
    }

    /// poll the eventloop. Only call this method once
    pub fn run(&self) {
        let mut connection = self
            .connection
            .borrow_mut()
            .take()
            .expect("expected connection");

        std::thread::spawn(move || {
            for notification in connection.iter() {
                if let Err(e) = notification {
                    eprintln!("MQTT connection error: {e}");
                }
            }
        });
    }

    /// Publish a measurement value for a sensor
    pub fn publish_measurement(&self, sensor_name: &str, value: u64) -> Result<(), WattwolfError> {
        let state_topic = format!("{}/{}/state", self.topic_prefix, sensor_name);

        self.client
            .publish(state_topic, QoS::AtLeastOnce, false, value.to_string())
            .map_err(WattwolfError::MqttSensorPublish)?;

        Ok(())
    }
}
