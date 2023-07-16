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
                        let new_values: Vec<String> = match val {
                            Value::Array(arr) => arr
                                .iter()
                                .filter_map(Value::as_str)
                                .map(String::from)
                                .collect(),
                            Value::String(s) => vec![s],
                            _ => vec![],
                        };

                        if new_values.is_empty() {
                            send_response(
                                400,
                                vec![(String::from("content-type"), String::from("text/html"))],
                                format!("No value provided for key: {}", key)
                                    .as_bytes()
                                    .to_vec(),
                            );
                            return;
                        }

                        // Get existing records, if any
                        let mut existing_records = get(&key)
                            .and_then(|val| serde_json::from_value(val).ok())
                            .unwrap_or_else(HashSet::new);

                        // Merge existing and new records
                        existing_records.extend(new_values);

                        // Save updated records
                        set(&key, serde_json::json!(existing_records), None);

                        send_response(
                            200,
                            vec![(String::from("content-type"), String::from("text/html"))],
                            format!("key: {}, values: {:?} saved", key, existing_records)
                                .as_bytes()
                                .to_vec(),
                        );
                        return;
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
