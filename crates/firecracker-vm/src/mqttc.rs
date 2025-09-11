use std::time::Duration;

use anyhow::Error;
use rumqttc::{AsyncClient, MqttOptions, QoS};

use crate::command::run_command;

pub async fn wait_for_mqtt_message(msgtype: &str) -> Result<String, Error> {
    println!("[+] Waiting for MQTT message...");
    let mut mqttoptions = MqttOptions::new("fireup", "localhost", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
    client.subscribe("/dhcp/#", QoS::AtLeastOnce).await?;

    while let Ok(notification) = eventloop.poll().await {
        if let rumqttc::Event::Incoming(rumqttc::Packet::Publish(publish)) = notification {
            let payload_str = String::from_utf8_lossy(&publish.payload).to_string();
            println!("[+] Received MQTT message: {}", payload_str);
            if payload_str.starts_with(msgtype) {
                let ip_addr = payload_str.split_whitespace().nth(2).ok_or_else(|| {
                    anyhow::anyhow!("Failed to extract IP address from MQTT message")
                })?;

                if !ip_addr.is_empty() {
                    let mut attempts = 0;
                    while attempts < 3 {
                        println!("[+] Pinging IP address: {}", ip_addr);
                        if run_command("sh", &["-c", &format!("ping -c 1 {}", ip_addr)], false)
                            .is_ok()
                        {
                            println!("[+] IP address {} is reachable.", ip_addr);
                            return Ok(payload_str);
                        }
                        println!("[-] IP address {} is not reachable yet.", ip_addr);
                        attempts += 1;
                        tokio::time::sleep(Duration::from_millis(500)).await;
                    }
                }
            }
        }
    }

    Err(Error::msg("Failed to receive MQTT message"))
}
