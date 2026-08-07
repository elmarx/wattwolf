use crate::config::MqttConfig;
use rumqttc::{Client, Connection, LastWill, MqttOptions, QoS};
use std::time::Duration;

pub fn init_client(config: &MqttConfig) -> (Client, Connection) {
    let mut mqtt_options = MqttOptions::new("wattwolf", &config.host, config.port);
    mqtt_options.set_keep_alive(Duration::from_secs(30));

    if let (Some(username), Some(password)) = (&config.username, &config.password) {
        mqtt_options.set_credentials(username, password.0.as_str());
    }

    // Configure Last Will and Testament to set device offline when connection is lost
    let last_will = LastWill::new(
        &config.availability_topic,
        "offline",
        QoS::AtLeastOnce,
        true,
    );
    mqtt_options.set_last_will(last_will);

    let (client, connection) = Client::new(mqtt_options, 10);

    // subscribe to homeassistant/status. If it publishes "online", we have to re-publish the discovery
    let ha_status_topic = format!("{}/status", config.homeassistant_topic);
    client
        .subscribe(ha_status_topic, QoS::AtMostOnce)
        .expect("subscribing to topics should not fail");

    (client, connection)
}
