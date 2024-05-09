use frankenstein::{MethodResponse, Update};

use kinode_process_lib::{
    await_message, call_init, get_blob,
    http::{HttpClientError, HttpClientResponse},
    println, Address, Message, Request, Response,
};

mod structs;
use structs::*;

mod helpers;
use helpers::*;

mod http;
use http::*;

static BASE_API_URL: &str = "https://api.telegram.org/bot";

wit_bindgen::generate!({
    path: "wit",
    world: "process",
});

use telegram_interface::{TgRequest, TgResponse, TgUpdate};

fn handle_request(
    our: &Address,
    state: &mut Option<State>,
    body: &[u8],
    source: &Address,
) -> anyhow::Result<()> {
    match serde_json::from_slice::<TgRequest>(body)? {
        TgRequest::RegisterApiKey(tg_initialize) => {
            match state {
                Some(state) => {
                    state.tg_key = tg_initialize.token.clone();
                    state.api_url = format!("{}{}", BASE_API_URL, tg_initialize.token.clone());
                    state.current_offset = 0;
                    state.api = Some(Api {
                        api_url: format!("{}{}", BASE_API_URL, tg_initialize.token.clone()),
                    });
                    state.save();
                }
                None => {
                    let state_ = State {
                        tg_key: tg_initialize.token.clone(),
                        api_url: format!("{}{}", BASE_API_URL, tg_initialize.token.clone()),
                        current_offset: 0,
                        subscribers: Vec::new(),
                        api: Some(Api {
                            api_url: format!("{}{}", BASE_API_URL, tg_initialize.token.clone()),
                        }),
                    };
                    state_.save();
                    *state = Some(state_);
                }
            }

            if let Some(ref state) = state {
                let updates_params = frankenstein::GetUpdatesParams {
                    offset: Some(state.current_offset as i64),
                    limit: None,
                    timeout: Some(15),
                    allowed_updates: None,
                };
                request_no_wait(&state.api_url, "getUpdates", Some(updates_params))?;
                let _ = Response::new()
                    .body(serde_json::to_vec(&TgResponse::Ok)?)
                    .send();
            }
        }
        TgRequest::Subscribe => {
            if let Some(state) = state {
                if !state.subscribers.contains(source) {
                    state.subscribers.push(source.clone());
                }
            }
            let _ = Response::new()
                .body(serde_json::to_vec(&TgResponse::Ok)?)
                .send();
        }
        TgRequest::Unsubscribe => {
            if let Some(state) = state {
                if let Some(index) = state.subscribers.iter().position(|x| x == source) {
                    state.subscribers.remove(index);
                }
            }
            let _ = Response::new()
                .body(serde_json::to_vec(&TgResponse::Ok)?)
                .send();
        }
        TgRequest::GetFile(get_file_params) => {
            let Some(state) = state else {
                return Err(anyhow::anyhow!("state not initialized"));
            };
            let Some(api) = state.api else {
                return Err(anyhow::anyhow!("api not initialized"));
            };

            let file_path = api.get_file(&get_file_params).ok()?.result.file_path?;
            let download_url = format!(
                "{}{}/{}",
                BASE_API_URL,
                state.tg_key.clone(),
                file_path 
            );

            let outgoing_request = http::OutgoingHttpRequest {
                method: "GET".to_string(),
                version: None,
                url: download_url,
                headers: HashMap::new(),
            };

            let body_bytes = json!(http::HttpClientAction::Http(outgoing_request))
                .to_string()
                .as_bytes()
                .to_vec();

            Request::new()
                .target(Address::new(
                    "our",
                    ProcessId::new(Some("http_client"), "distro", "sys"),
                ))
                .body(body_bytes)
                .context(vec![0])
                .expects_response(30)
                .send()
                .ok();
        }
        TgRequest::SendMessage(send_message_params) => {
            // TODO:
        }
    }

    Ok(())
}

fn handle_inner_message(our: &Address, state: &mut Option<State>) -> anyhow::Result<()> {
    match message {
        Message::Request {
            ref body, source, ..
        } => handle_request(our, state, body, &source),
        Message::Response { ref body, .. } => Ok(()),
    }
}


fn handle_message(our: &Address, state: &mut Option<State>) -> anyhow::Result<()> {
    let message = await_message()?;
    if message.source().node != our.node {
        return Err(anyhow::anyhow!(
            "got request from foreign source {:?}",
            message.source()
        ));
    }

    match message.source().process.to_string().as_str() {
        "http_server:distro:sys" | "http_client:distro:sys" => {
            handle_http_message(&message, state)
        }
        _ => handle_inner_message(our, state)
    }
}

call_init!(init);
fn init(our: Address) {
    println!("tg_bot: booted");
    let mut state = State::fetch();

    loop {
        match handle_message(&our, &mut state) {
            Ok(()) => {}
            Err(e) => {
                println!("tg: error: {:?}", e);
            }
        };
    }
}
