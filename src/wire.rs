use crate::config::Config;
use crate::mqtt;
use crate::mqtt::{MqttPublisher, init_client};
use rumqttc::v5::Connection;

pub fn wire(
    config: &Config,
) -> (
    MqttPublisher,
    impl mqtt::MqttEventHandler + use<>,
    Connection,
) {
    let (client, connection) = init_client(&config.mqtt);

    let publisher = MqttPublisher::new(&config.mqtt, client, &config.sensors);
    // publisher implement MqttEventHandler
    let handler = publisher.clone();

    (publisher, handler, connection)
}
