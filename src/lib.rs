use lambda_flows::{request_received, send_response};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use store_flows::{get, set};
use tokio;

#[no_mangle]
#[tokio::main(flavor = "current_thread")]
pub async fn run() {
    request_received(handler).await;
}

async fn handler(qry: HashMap<String, Value>, body: Vec<u8>) {
    let mut val = String::from("no data found");

    match qry.get("key") {
        Some(key_value) => match key_value.as_str() {
            Some(key_to_get_val) => {
                let data = get(key_to_get_val);
                if let Some(data) = data {
                    if let Ok(records) = serde_json::from_value::<HashSet<String>>(data.clone()) {
                        val = format!("{:?}", records);
                    } else {
                        val = data.to_string();
                    }
                }
            },
            None => {}
        },
        None => {}
    }

    if !body.is_empty() {
        let result: Result<serde_json::Value, _> = serde_json::from_slice(&body);

        match result {
            Ok(data) => {
                if let serde_json::Value::Object(map) = data {
                    for (key, val) in map {
                        if val.is_array() {
                            let records: HashSet<String> = val.as_array().unwrap()
                                .iter()
                                .filter_map(Value::as_str)
                                .map(String::from)
                                .collect();
                            set(&key, serde_json::json!(records), None);
                        } else {
                            if val.as_str().map(|s| s.is_empty()).unwrap_or(true) {
                                send_response(
                                    400,
                                    vec![(String::from("content-type"), String::from("text/html"))],
                                    format!("No value provided for key: {}", key)
                                        .as_bytes()
                                        .to_vec(),
                                );
                                return;
                            }
                            set(&key, val.clone(), None);
                            send_response(
                                200,
                                vec![(String::from("content-type"), String::from("text/html"))],
                                format!("key: {}, val: {:?} saved", key, val)
                                    .as_bytes()
                                    .to_vec(),
                            );
                            return;
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to parse JSON: {:?}", e);
            }
        }
    }

    send_response(
        200,
        vec![(String::from("content-type"), String::from("text/html"))],
        val.as_bytes().to_vec(),
    );
}
