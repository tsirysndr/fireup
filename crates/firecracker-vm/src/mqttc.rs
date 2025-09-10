use std::time::Duration;

use anyhow::Error;
use rumqttc::{AsyncClient, MqttOptions, QoS};

pub async fn wait_for_mqtt_message(msgtype: &str) -> Result<String, Error> {
    println!("[+] Waiting for MQTT message...");
    let mut mqttoptions = MqttOptions::new("fireup", "localhost", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
    client.subscribe("/dhcp/#", QoS::AtMostOnce).await?;

    while let Ok(notification) = eventloop.poll().await {
        if let rumqttc::Event::Incoming(rumqttc::Packet::Publish(publish)) = notification {
            let payload_str = String::from_utf8_lossy(&publish.payload).to_string();
            println!("[+] Received MQTT message: {}", payload_str);
            if payload_str.starts_with(msgtype) {
                return Ok(payload_str);
            }
        }
    }

    Err(Error::msg("Failed to receive MQTT message"))
}
