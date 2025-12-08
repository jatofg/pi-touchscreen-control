extern crate env_logger;

use crate::config::MqttConfig;
use crate::state::State;
use async_std::task;
use futures::join;
use log::info;
use mosquitto_rs::*;
use serde_json::json;
use std::sync::{Arc, RwLock};
use std::time::Duration;

async fn publish_device_config(client: &Client, config: &MqttConfig) {
    let discovery = json!({
        "dev": {
            "ids": config.device_id.clone(),
            "name": config.device_name.clone(),
            "sw": "0.1"
        },
        "o": {
            "name": "pi_touchscreen_control",
            "sw": "0.1",
            "url": "https://github.com/jatofg/pi-touchscreen-control"
        },
        "cmps": {
            "power_state": {
                "p": "switch",
                "device_class": "switch",
                "unique_id": config.device_id.clone() + "_power",
                "name": "Backlight enabled",
                "state_topic": config.app_topic_prefix.clone() + "/power_state/state",
                "payload_on": "on",
                "payload_off": "off",
                "command_topic": config.app_topic_prefix.clone() + "/power_state/set",
                "availability_topic": config.app_topic_prefix.clone() + "/power_state/available",
                "payload_available": "online",
                "payload_not_available": "offline",
            }
        }
    })
    .to_string();

    let device_config_topic =
        config.discovery_topic_prefix.clone() + "/device/" + config.device_id.as_str() + "/config";
    client
        .publish(
            device_config_topic.as_str(),
            discovery.as_str(),
            QoS::ExactlyOnce,
            true,
        )
        .await
        .expect("Unable to publish device config");

    let power_state_available_topic = config.app_topic_prefix.clone() + "/power_state/available";
    client
        .publish(
            power_state_available_topic.as_str(),
            "online",
            QoS::ExactlyOnce,
            true,
        )
        .await
        .expect("Unable to publish availability");

    info!("Published device config");
}

async fn publish_state(client: &Client, config: &MqttConfig, state: &Arc<RwLock<State>>) {
    // TODO add stuff from state
    let power_state_state_topic = config.app_topic_prefix.clone() + "/power_state/state";
    let current_state = state
        .read()
        .expect("Unable to acquire read lock on state")
        .clone();
    client
        .publish(
            power_state_state_topic.as_str(),
            if current_state.backlight_active_current {
                "on"
            } else {
                "off"
            },
            QoS::ExactlyOnce,
            false,
        )
        .await
        .expect("Unable to publish power state");
    info!("Published power state");
}

fn get_home_assistant_status_topic(config: &MqttConfig) -> String {
    config.discovery_topic_prefix.clone() + "/status"
}

async fn run_state_publisher(client: &Client, config: &MqttConfig, state: &Arc<RwLock<State>>) {
    let mut previous_state = state
        .read()
        .expect("Unable to acquire read lock on state")
        .clone();
    loop {
        let current_state = state
            .read()
            .expect("Unable to acquire read lock on state")
            .clone();
        if current_state != previous_state {
            publish_state(client, config, state).await;
        }
        previous_state = current_state;
        task::sleep(Duration::from_millis(100)).await;
    }
}

async fn run_mqtt_listener(client: &Client, config: &MqttConfig, state: &Arc<RwLock<State>>) {
    let subscriptions = client.subscriber().unwrap();
    let power_state_set_topic = config.app_topic_prefix.clone() + "/power_state/set";
    let home_assistant_status_topic = get_home_assistant_status_topic(config);
    loop {
        if let Ok(event) = subscriptions.recv().await {
            if let Event::Message(msg) = event {
                if msg.topic == power_state_set_topic {
                    if msg.payload == b"on" {
                        info!("Turning backlight on");
                        {
                            let mut state = state
                                .write()
                                .expect("Unable to acquire write lock on state");
                            state.backlight_active_current = true;
                        }
                    } else {
                        info!("Turning backlight off");
                        {
                            let mut state = state
                                .write()
                                .expect("Unable to acquire write lock on state");
                            state.backlight_active_current = false;
                        }
                    }
                } else if msg.topic == home_assistant_status_topic && msg.payload == b"online" {
                    info!("Publishing state again on Home Assistant restart");
                    publish_state(&client, config, &state).await;
                }
            }
        }
        task::sleep(Duration::from_millis(100)).await;
    }
}

pub fn run_mqtt(config: &MqttConfig, state: Arc<RwLock<State>>) {
    env_logger::init();

    let user_name = if config.auth_username.is_empty() {
        None
    } else {
        Some(config.auth_username.as_str())
    };
    let password = if config.auth_password.is_empty() {
        None
    } else {
        Some(config.auth_password.as_str())
    };

    smol::block_on(async {
        let client = Client::with_auto_id().unwrap();
        client
            .set_username_and_password(user_name, password)
            .expect("Invalid username and password for MQTT server");
        let rc = client
            .connect(
                config.server_address.as_str(),
                config.server_port as std::os::raw::c_int,
                std::time::Duration::from_secs(5),
                None,
            )
            .await
            .expect("Unable to connect to MQTT server");
        info!("Connection status: {rc}");

        let power_state_topic = config.app_topic_prefix.clone() + "/power_state/set";
        client
            .subscribe(power_state_topic.as_str(), QoS::AtMostOnce)
            .await
            .unwrap();
        let home_assistant_status_topic = get_home_assistant_status_topic(config);
        client
            .subscribe(home_assistant_status_topic.as_str(), QoS::AtMostOnce)
            .await
            .unwrap();
        info!("Subscribed to relevant topics");

        publish_device_config(&client, config).await;
        publish_state(&client, config, &state).await;

        let state_publisher_fut = run_state_publisher(&client, &config, &state);
        let mqtt_listener_fut = run_mqtt_listener(&client, &config, &state);
        join!(state_publisher_fut, mqtt_listener_fut);
    });

    info!("MQTT thread is exiting.");
}
