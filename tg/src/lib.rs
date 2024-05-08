use frankenstein::{MethodResponse, Update};

use kinode_process_lib::{
    await_message, call_init, get_blob,
    http::{HttpClientError, HttpClientResponse},
    println, Address, Message, Request,
};

mod structs;
use structs::*;

mod helpers;
use helpers::*;

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
    if source.node != our.node {
        return Err(anyhow::anyhow!(
            "got initialize request from foreign source {:?}",
            source
        ));
    }
    match serde_json::from_slice::<TgRequest>(body)? {
        TgRequest::RegisterApiKey(tg_initialize) => {
            match state {
                Some(state) => {
                    state.tg_key = tg_initialize.token.clone();
                    state.api_url = format!("{}{}", BASE_API_URL, tg_initialize.token);
                    state.current_offset = 0;
                    state.save();
                }
                None => {
                    let state_ = State {
                        tg_key: tg_initialize.token.clone(),
                        api_url: format!("{}{}", BASE_API_URL, tg_initialize.token),
                        current_offset: 0,
                        subscribers: Vec::new(),
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
                let _ = Response::new().body(serde_json::to_vec(&TgResponse::Ok)?).send();
            }
        }
        TgRequest::Subscribe => {
            if let Some(state) = state {
                if !state.subscribers.contains(source) {
                    state.subscribers.push(source.clone());
                }
            }
            let _ = Response::new().body(serde_json::to_vec(&TgResponse::Ok)?).send();
        }
        TgRequest::Unsubscribe => {
            if let Some(state) = state {
                if let Some(index) = state.subscribers.iter().position(|x| x == source) {
                    state.subscribers.remove(index);
                }
            }
            let _ = Response::new().body(serde_json::to_vec(&TgResponse::Ok)?).send();
        }
    }

    Ok(())
}

fn handle_response(
    state: &mut Option<State>,
    body: &[u8],
) -> anyhow::Result<()> {
    let HttpClientResponse::Http(_) =
        serde_json::from_slice::<Result<HttpClientResponse, HttpClientError>>(&body)??
    else {
        return Err(anyhow::anyhow!("unexpected Response: "));
    };
    let Some(state) = state else {
        return Err(anyhow::anyhow!("state not initialized"));
    };

    if let Some(blob) = get_blob() {
        let Ok(response) = serde_json::from_slice::<MethodResponse<Vec<Update>>>(&blob.bytes)
        else {
            return Err(anyhow::anyhow!("unexpected Response: "));
        };
        // forward to subs
        for sub in state.subscribers.iter() {
            let request = TgUpdate {
                updates: response.result.clone(),
            };

            let tg_response = TgResponse::Update(request);
            let _ = Request::new()
                .target(sub.clone())
                .body(serde_json::to_vec(&tg_response)?)
                .send();
        }

        // set current_offset based on the response, keep same if no updates
        let next_offset = response
            .result
            .last()
            .map(|u| u.update_id + 1)
            .unwrap_or(state.current_offset);
        state.current_offset = next_offset;

        let updates_params = frankenstein::GetUpdatesParams {
            offset: Some(state.current_offset as i64),
            limit: None,
            timeout: Some(15),
            allowed_updates: None,
        };

        request_no_wait(&state.api_url, "getUpdates", Some(updates_params))?;
    } else {
        for sub in state.subscribers.iter() {
            let error_message = format!(
                "tg_bot, failed to serialize response: {:?}",
                std::str::from_utf8(&body).unwrap_or("[Invalid UTF-8]")
            );
            let tg_response = TgResponse::Error(error_message);
            let _ = Request::new()
                .target(sub.clone())
                .body(serde_json::to_vec(&tg_response)?)
                .send();
        }
    }
    Ok(())
}

fn handle_message(
    our: &Address,
    state: &mut Option<State>,
) -> anyhow::Result<()> {
    let message = await_message()?;
    match message {
        Message::Request {
            ref body, source, ..
        } => {
            handle_request(our, state, body, &source)
        }
        Message::Response { ref body, .. } => {
            handle_response(state, body)
        }
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
