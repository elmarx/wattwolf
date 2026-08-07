pub trait MqttEventHandler: Send + Sync {
    /// to be called when home-assistant signals it's online (topic homeassistant/state by default)
    fn on_mqtt_message(&self, topic: String, payload: bytes::Bytes);

    // to be called when the mqtt connection is available, i.e. initial connection and when connection has been lost
    fn on_mqtt_connection_online(&self);
}
