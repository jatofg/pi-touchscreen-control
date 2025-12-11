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

struct MqttData {
    state: Arc<RwLock<State>>,
    client: Client,
    discovery: serde_json::Value,
    device_config_topic: String,
    home_assistant_status_topic: String,
}

impl MqttData {
    pub fn new(config: &MqttConfig, state: Arc<RwLock<State>>) -> Self {
        Self {
            state,
            client: Client::with_auto_id().unwrap(),
            discovery: json!({
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
                    },
                    "brightness": {
                        "p": "number",
                        "unique_id": config.device_id.clone() + "_brightness",
                        "name": "Current brightness",
                        "state_topic": config.app_topic_prefix.clone() + "/brightness/state",
                        "command_topic": config.app_topic_prefix.clone() + "/brightness/set",
                        "availability_topic": config.app_topic_prefix.clone() + "/brightness/available",
                        "min": 1,
                        "max": 255,
                        "mode": "slider",
                        "step": 1,
                    },
                    "use_dimmer": {
                        "p": "switch",
                        "unique_id": config.device_id.clone() + "_use_dimmer",
                        "name": "Automatic dimming",
                        "state_topic": config.app_topic_prefix.clone() + "/use_dimmer/state",
                        "payload_on": "on",
                        "payload_off": "off",
                        "command_topic": config.app_topic_prefix.clone() + "/use_dimmer/set",
                        "availability_topic": config.app_topic_prefix.clone() + "/use_dimmer/available",
                        "payload_available": "online",
                        "payload_not_available": "offline",
                    },
                    "power_dimmed": {
                        "p": "switch",
                        "unique_id": config.device_id.clone() + "_power_dimmed",
                        "name": "Backlight enabled when inactive",
                        "state_topic": config.app_topic_prefix.clone() + "/power_dimmed/state",
                        "payload_on": "on",
                        "payload_off": "off",
                        "command_topic": config.app_topic_prefix.clone() + "/power_dimmed/set",
                        "availability_topic": config.app_topic_prefix.clone() + "/power_dimmed/available",
                        "payload_available": "online",
                        "payload_not_available": "offline",
                    },
                    "brightness_dimmed": {
                        "p": "number",
                        "unique_id": config.device_id.clone() + "_brightness_dimmed",
                        "name": "Brightness when inactive",
                        "state_topic": config.app_topic_prefix.clone() + "/brightness_dimmed/state",
                        "command_topic": config.app_topic_prefix.clone() + "/brightness_dimmed/set",
                        "availability_topic": config.app_topic_prefix.clone() + "/brightness_dimmed/available",
                        "min": 1,
                        "max": 255,
                        "mode": "slider",
                        "step": 1,
                    },
                    "brightness_full": {
                        "p": "number",
                        "unique_id": config.device_id.clone() + "_brightness_full",
                        "name": "Brightness when active",
                        "state_topic": config.app_topic_prefix.clone() + "/brightness_full/state",
                        "command_topic": config.app_topic_prefix.clone() + "/brightness_full/set",
                        "availability_topic": config.app_topic_prefix.clone() + "/brightness_full/available",
                        "min": 1,
                        "max": 255,
                        "mode": "slider",
                        "step": 1,
                    },
                    "timeout_sec": {
                        "p": "number",
                        "unique_id": config.device_id.clone() + "_timeout_sec",
                        "name": "Activity timeout for dimming",
                        "state_topic": config.app_topic_prefix.clone() + "/timeout_sec/state",
                        "command_topic": config.app_topic_prefix.clone() + "/timeout_sec/set",
                        "availability_topic": config.app_topic_prefix.clone() + "/timeout_sec/available",
                        "min": 1,
                        "max": 86400,
                        "mode": "box",
                        "step": 1,
                        "unit_of_measurement": "seconds",
                    },
                },
            }),
            device_config_topic: config.discovery_topic_prefix.clone()
                + "/device/"
                + config.device_id.as_str()
                + "/config",
            home_assistant_status_topic: config.discovery_topic_prefix.clone() + "/status",
        }
    }
}

async fn publish_device_config(mqtt_data: &MqttData) {
    // Publish discovery
    mqtt_data
        .client
        .publish(
            mqtt_data.device_config_topic.as_str(),
            mqtt_data.discovery.to_string().as_str(),
            QoS::ExactlyOnce,
            true,
        )
        .await
        .expect("Unable to publish device config");

    // Publish availability for all entities
    for entity in mqtt_data.discovery["cmps"].as_object().unwrap().values() {
        mqtt_data
            .client
            .publish(
                entity["availability_topic"].as_str().unwrap(),
                "online",
                QoS::ExactlyOnce,
                true,
            )
            .await
            .expect(
                format!(
                    "Unable to publish availability for entity {}",
                    entity["unique_id"].as_str().unwrap()
                )
                .as_str(),
            );
    }

    info!("Published device config");
}

fn bool_to_mqtt_payload(value: bool) -> &'static str {
    if value { "on" } else { "off" }
}

fn mqtt_payload_to_bool(value: &Vec<u8>) -> bool {
    value == b"on"
}

fn duration_to_mqtt_payload(value: &Duration) -> String {
    value.as_secs().to_string()
}

fn mqtt_payload_to_duration(value: &Vec<u8>) -> Duration {
    Duration::from_secs(
        String::from_utf8(value.clone())
            .unwrap()
            .parse::<u64>()
            .unwrap(),
    )
}

fn mqtt_payload_to_u8(value: &Vec<u8>) -> u8 {
    String::from_utf8(value.clone())
        .unwrap()
        .parse::<u8>()
        .unwrap()
}

async fn publish_state_for_entity(client: &Client, entity: &serde_json::Value, value: &str) {
    client
        .publish(
            entity["state_topic"].as_str().unwrap(),
            value,
            QoS::ExactlyOnce,
            false,
        )
        .await
        .expect(
            format!(
                "Unable to publish state for entity {}",
                entity["unique_id"].as_str().unwrap()
            )
            .as_str(),
        );
    info!(
        "Published state for entity {}",
        entity["unique_id"].as_str().unwrap()
    );
}

async fn publish_state(
    mqtt_data: &MqttData,
    previous_state: Option<&State>,
    current_state: &State,
) {
    if previous_state.is_none()
        || previous_state.unwrap().backlight_active_current
            != current_state.backlight_active_current
    {
        publish_state_for_entity(
            &mqtt_data.client,
            &mqtt_data.discovery["cmps"]["power_state"],
            bool_to_mqtt_payload(current_state.backlight_active_current),
        )
        .await;
    }
    if previous_state.is_none()
        || previous_state.unwrap().backlight_active_dimmed != current_state.backlight_active_dimmed
    {
        publish_state_for_entity(
            &mqtt_data.client,
            &mqtt_data.discovery["cmps"]["power_dimmed"],
            bool_to_mqtt_payload(current_state.backlight_active_dimmed),
        )
        .await;
    }
    if previous_state.is_none()
        || previous_state.unwrap().brightness_current != current_state.brightness_current
    {
        publish_state_for_entity(
            &mqtt_data.client,
            &mqtt_data.discovery["cmps"]["brightness"],
            &current_state.brightness_current.to_string(),
        )
        .await;
    }
    if previous_state.is_none()
        || previous_state.unwrap().brightness_dimmed != current_state.brightness_dimmed
    {
        publish_state_for_entity(
            &mqtt_data.client,
            &mqtt_data.discovery["cmps"]["brightness_dimmed"],
            &current_state.brightness_dimmed.to_string(),
        )
        .await;
    }
    if previous_state.is_none()
        || previous_state.unwrap().brightness_full != current_state.brightness_full
    {
        publish_state_for_entity(
            &mqtt_data.client,
            &mqtt_data.discovery["cmps"]["brightness_full"],
            &current_state.brightness_full.to_string(),
        )
        .await;
    }
    if previous_state.is_none() || previous_state.unwrap().use_dimmer != current_state.use_dimmer {
        publish_state_for_entity(
            &mqtt_data.client,
            &mqtt_data.discovery["cmps"]["use_dimmer"],
            bool_to_mqtt_payload(current_state.use_dimmer),
        )
        .await;
    }
    if previous_state.is_none() || previous_state.unwrap().timeout != current_state.timeout {
        publish_state_for_entity(
            &mqtt_data.client,
            &mqtt_data.discovery["cmps"]["timeout_sec"],
            &duration_to_mqtt_payload(&current_state.timeout),
        )
        .await;
    }
}

async fn run_state_publisher(mqtt_data: &MqttData) {
    let mut previous_state = mqtt_data
        .state
        .read()
        .expect("Unable to acquire read lock on state")
        .clone();
    loop {
        let current_state = mqtt_data
            .state
            .read()
            .expect("Unable to acquire read lock on state")
            .clone();
        if current_state != previous_state {
            publish_state(&mqtt_data, Some(&previous_state), &current_state).await;
        }
        previous_state = current_state;
        task::sleep(Duration::from_millis(100)).await;
    }
}

async fn run_mqtt_listener(mqtt_data: &MqttData) {
    let subscriptions = mqtt_data.client.subscriber().unwrap();

    loop {
        if let Ok(event) = subscriptions.recv().await {
            if let Event::Message(msg) = event {
                if msg.topic == mqtt_data.home_assistant_status_topic && msg.payload == b"online" {
                    info!("Publishing state again on Home Assistant restart");
                    publish_state(&mqtt_data, None, &mqtt_data.state.read().unwrap()).await;
                } else {
                    let mut state = mqtt_data
                        .state
                        .write()
                        .expect("Unable to acquire write lock on state");

                    if msg.topic
                        == mqtt_data.discovery["cmps"]["power_state"]["command_topic"]
                            .as_str()
                            .unwrap()
                    {
                        state.backlight_active_current = mqtt_payload_to_bool(&msg.payload);
                    } else if msg.topic
                        == mqtt_data.discovery["cmps"]["power_dimmed"]["command_topic"]
                            .as_str()
                            .unwrap()
                    {
                        state.backlight_active_dimmed = mqtt_payload_to_bool(&msg.payload);
                    } else if msg.topic
                        == mqtt_data.discovery["cmps"]["brightness"]["command_topic"]
                            .as_str()
                            .unwrap()
                    {
                        state.brightness_current = mqtt_payload_to_u8(&msg.payload);
                    } else if msg.topic
                        == mqtt_data.discovery["cmps"]["brightness_dimmed"]["command_topic"]
                            .as_str()
                            .unwrap()
                    {
                        state.brightness_dimmed = mqtt_payload_to_u8(&msg.payload);
                    } else if msg.topic
                        == mqtt_data.discovery["cmps"]["brightness_full"]["command_topic"]
                            .as_str()
                            .unwrap()
                    {
                        state.brightness_full = mqtt_payload_to_u8(&msg.payload);
                    } else if msg.topic
                        == mqtt_data.discovery["cmps"]["use_dimmer"]["command_topic"]
                            .as_str()
                            .unwrap()
                    {
                        state.use_dimmer = mqtt_payload_to_bool(&msg.payload);
                    } else if msg.topic
                        == mqtt_data.discovery["cmps"]["timeout_sec"]["command_topic"]
                            .as_str()
                            .unwrap()
                    {
                        state.timeout = mqtt_payload_to_duration(&msg.payload);
                    }
                }
            }
        }
        task::sleep(Duration::from_millis(100)).await;
    }
}

async fn subscribe_to_topics(mqtt_data: &MqttData) {
    mqtt_data
        .client
        .subscribe(
            mqtt_data.home_assistant_status_topic.as_str(),
            QoS::AtMostOnce,
        )
        .await
        .unwrap();

    for entity in mqtt_data.discovery["cmps"].as_object().unwrap().values() {
        mqtt_data
            .client
            .subscribe(entity["command_topic"].as_str().unwrap(), QoS::AtMostOnce)
            .await
            .unwrap();
    }

    info!("Subscribed to relevant topics");
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
        let mqtt_data = MqttData::new(config, state);

        mqtt_data
            .client
            .set_username_and_password(user_name, password)
            .expect("Invalid username and password for MQTT server");
        let rc = mqtt_data
            .client
            .connect(
                config.server_address.as_str(),
                config.server_port as std::os::raw::c_int,
                Duration::from_secs(5),
                None,
            )
            .await
            .expect("Unable to connect to MQTT server");
        info!("Connection status: {rc}");

        subscribe_to_topics(&mqtt_data).await;
        publish_device_config(&mqtt_data).await;
        publish_state(&mqtt_data, None, &mqtt_data.state.read().unwrap()).await;

        let state_publisher_fut = run_state_publisher(&mqtt_data);
        let mqtt_listener_fut = run_mqtt_listener(&mqtt_data);
        join!(state_publisher_fut, mqtt_listener_fut);
    });

    info!("MQTT thread is exiting.");
}
