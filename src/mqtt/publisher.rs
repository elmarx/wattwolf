use crate::config::{MqttConfig, Sensor};
use crate::error::WattwolfError;
use crate::mqtt::event_handler::MqttEventHandler;
use rumqttc::{Client, QoS};
use serde_json::json;
use tracing::{error, info};

#[derive(Clone)]
pub struct MqttPublisher {
    client: Client,
    topic_prefix: String,
    ha_topic: String,
    availability_topic: String,
    sensors: Vec<Sensor>,
}

/// implement `MqttEventHandler` for `MqttPublisher` directly, as we need to publish messages in reaction to events
impl MqttEventHandler for MqttPublisher {
    /// publish discovery and the "online" state once wattwolf's own mqtt connection is established.
    ///
    /// Discovery and availability are both published retained, so Home Assistant learns about
    /// this device/its sensors as soon as it (re-)subscribes, regardless of whether it was
    /// online at the time of publishing. We don't need to react to Home Assistant's birth
    /// message (homeassistant/status) for this reason - republishing here, on every successful
    /// (re-)connect, is enough.
    fn on_mqtt_connection_online(&self) {
        if let Err(e) = self.publish_discovery() {
            error!(error = %e, "Publishing discovery message state failed");
        } else {
            info!("Published device discovery message");
        }

        if let Err(e) = self.publish_online() {
            // if the connection is dead there is not much we can/have to do, so just log.
            error!(error = %e, "Publishing \"online\" state failed");
        } else {
            info!("Device status set to online");
        }
    }
}

impl MqttPublisher {
    pub fn new(config: &MqttConfig, client: rumqttc::Client, sensors: &[Sensor]) -> Self {
        Self {
            client,
            topic_prefix: config.topic_prefix.clone(),
            availability_topic: config.availability_topic.clone(),
            ha_topic: config.homeassistant_topic.clone(),
            sensors: sensors.to_vec(),
        }
    }

    /// Publish Home Assistant auto-discovery messages for all sensors
    pub fn publish_discovery(&self) -> Result<(), WattwolfError> {
        for sensor in &self.sensors {
            self.publish_sensor_discovery(sensor)?;
        }
        info!(
            sensor_count = self.sensors.len(),
            "Published auto-discovery messages"
        );
        Ok(())
    }

    /// Publish Home Assistant auto-discovery message for a single sensor
    fn publish_sensor_discovery(&self, sensor: &Sensor) -> Result<(), WattwolfError> {
        // device class as configured by the user
        let device_class = sensor
            .device_class
            .as_ref()
            .map(std::convert::AsRef::as_ref);
        // unit is hardcoded given the obis numbers
        let unit = sensor.obis.unit.as_ref();

        // see discovery payload: https://www.home-assistant.io/integrations/mqtt/#single-component-discovery-payload
        let discovery_payload = json!({
            "name": sensor.friendly_name,
            "unique_id": format!("wattwolf_{}", sensor.name),
            "state_topic": format!("{}/{}/state", self.topic_prefix, sensor.name),
            "unit_of_measurement": unit,
            "device_class": device_class,
            "state_class": sensor.state_class.as_deref(),
            "availability_topic": self.availability_topic,
            "payload_available": "online",
            "payload_not_available": "offline",
            "device": {
                "identifiers": ["wattwolf"],
                "name": "Wattwolf Smart Meter",
                "model": "SML Reader",
                "manufacturer": "Wattwolf"
            }
        });

        // see discovery topic documentation here: https://www.home-assistant.io/integrations/mqtt/#discovery-topic
        let discovery_topic = format!("{}/sensor/wattwolf/{}/config", self.ha_topic, sensor.name);

        // retained, so Home Assistant receives the discovery config from the broker as soon as
        // it (re-)subscribes, even if it wasn't online when we published it.
        self.client
            .publish(
                discovery_topic,
                QoS::AtLeastOnce,
                true,
                discovery_payload.to_string(),
            )
            .map_err(WattwolfError::MqttHaDiscovery)?;

        Ok(())
    }

    /// Publish a measurement value for a sensor
    pub fn publish_measurement(&self, sensor_name: &str, value: u64) -> Result<(), WattwolfError> {
        let state_topic = format!("{}/{}/state", self.topic_prefix, sensor_name);

        self.client
            .publish(state_topic, QoS::AtLeastOnce, true, value.to_string())
            .map_err(WattwolfError::MqttSensorPublish)?;

        Ok(())
    }

    /// Publish "online" to the availability topic
    fn publish_online(&self) -> Result<(), WattwolfError> {
        self.client
            .publish(&self.availability_topic, QoS::AtLeastOnce, true, "online")
            .map_err(WattwolfError::MqttOnlinePublish)
    }
}
