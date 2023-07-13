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
    if let Some(Value::String(method)) = qry.get("method") {
        match method.as_str() {
            "POST" => handle_post(qry, body),
            _ => handle_get(qry),
        }
    } else {
        handle_get(qry);
    }
}

fn handle_post(_qry: HashMap<String, Value>, body: Vec<u8>) {
    let data: Result<HashMap<String, Value>, _> = serde_json::from_slice(&body);
    send_response(
        200,
        vec![(String::from("content-type"), String::from("text/html"))],
        body,
    );
    match data {
        Ok(parsed_data) => {
            for (key, val) in parsed_data {
                set(&key, val.clone(), None);
                send_response(
                    200,
                    vec![(String::from("content-type"), String::from("text/html"))],
                    format!("key: {key}, val: {val} saved").as_bytes().to_vec(),
                );

                break;
            }
        }
        Err(e) => {
            send_response(
                400,
                vec![(String::from("content-type"), String::from("text/html"))],
                format!("Error parsing POST payload JSON: {:?}", e)
                    .as_bytes()
                    .to_vec(),
            );
        }
    }
}

fn handle_get(_qry: HashMap<String, Value>) {
    let key = _qry.get("key").unwrap().as_str().unwrap();

    let mut val = String::from("no data found");

    match get(&key) {
        Some(data) => val = data.to_string(),

        None => {}
    }

    send_response(
        200,
        vec![(String::from("content-type"), String::from("text/html"))],
        val.as_bytes().to_vec(),
    );
}
