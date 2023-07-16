use lambda_flows::{request_received, send_response};
use serde_json::Value;
use std::collections::HashMap;
use store_flows::{get, set};
use tokio;

#[no_mangle]
#[tokio::main(flavor = "current_thread")]
pub async fn run() {
    request_received(handler).await;
}

// set_response_status(status as i32);
// set_response_headers(headers.as_ptr(), headers.len() as i32);
// set_response(body.as_ptr(), body.len() as i32);

async fn handler(qry: HashMap<String, Value>, body: Vec<u8>) {
    let mut val = String::from("no data found");
    match qry.get("key").unwrap().as_str() {
        Some(key_to_get_val) => match get(&key_to_get_val) {
            Some(data) => {
                val = data.to_string();
            }

            None => {}
        },
        None => {}
    }

    if !body.is_empty() {
        let result: Result<HashMap<String, Value>, _> = serde_json::from_slice(&body);

        match result {
            Ok(data) => {
                for (key, val) in data {
                    if val.as_str().map(|s| s.is_empty()).unwrap_or(true) {
                        send_response(
                            400,
                            vec![(String::from("content-type"), String::from("text/html"))],
                            format!("No value provided for key: {key}")
                                .as_bytes()
                                .to_vec(),
                        );
                        return;
                    }
                    set(&key, val.clone(), None);
                    send_response(
                        200,
                        vec![(String::from("content-type"), String::from("text/html"))],
                        format!("key: {key}, val: {val:?} saved")
                            .as_bytes()
                            .to_vec(),
                    );
                    return;
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
