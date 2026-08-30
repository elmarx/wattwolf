pub trait MqttEventHandler: Send + Sync {
    // to be called when the mqtt connection is available, i.e. initial connection and when connection has been lost
    fn on_mqtt_connection_online(&self);
}
