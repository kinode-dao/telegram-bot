use frankenstein::{GetUpdatesParams, MethodResponse, Update};

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
    exports: {
        world: Component,
    },
});

use telegram_interface::{TgInitialize, TgRequest, TgResponse, TgUpdate};

fn handle_request(
    our: &Address,
    subs: &mut Vec<Address>,
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
            }
        }
        TgRequest::Subscribe => {
            if !subs.contains(source) {
                subs.push(source.clone());
            }
        }
        TgRequest::Unsubscribe => {
            if let Some(index) = subs.iter().position(|x| x == source) {
                subs.remove(index);
            }
        }
    }

    Ok(())
}

fn handle_response(
    our: &Address,
    subs: &mut Vec<Address>,
    state: &mut Option<State>,
    body: &[u8],
) -> anyhow::Result<()> {
    // let HttpClientResponse::Http(response) =
    //     serde_json::from_slice::<Result<HttpClientResponse, HttpClientError>>(&body)??
    // else {
    //     return Err(anyhow::anyhow!("unexpected Response: "));
    // };

    // if let Some(blob) = get_blob() {
    //     let Ok(response) = serde_json::from_slice::<MethodResponse<Vec<Update>>>(&blob.bytes)
    //     else {
    //         return Err(anyhow::anyhow!("unexpected Response: "));
    //     };
    //     // forward to subs
    //     for sub in subs.iter() {
    //         let request = TgUpdate {
    //             updates: response.result.clone(),
    //         };

    //         let tg_response = TgResponse::Update(request);
    //         let _ = Request::new()
    //             .target(sub.clone())
    //             .body(serde_json::to_vec(&tg_response)?)
    //             .send();
    //     }

    //     // set current_offset based on the response, keep same if no updates
    //     let next_offset = response
    //         .result
    //         .last()
    //         .map(|u| u.update_id + 1)
    //         .unwrap_or(state.current_offset);
    //     state.current_offset = next_offset;

    //     let updates_params = frankenstein::GetUpdatesParams {
    //         offset: Some(state.current_offset as i64),
    //         limit: None,
    //         timeout: Some(15),
    //         allowed_updates: None,
    //     };

    //     request_no_wait(&state.api_url, "getUpdates", Some(updates_params))?;
    // } else {
    //     if let Some(ref parent_address) = subs {
    //         let error_message = format!(
    //             "tg_bot, failed to serialize response: {:?}",
    //             std::str::from_utf8(&body).unwrap_or("[Invalid UTF-8]")
    //         );
    //         let tg_response = TgResponse::Error(error_message);
    //         let _ = Request::new()
    //             .target(parent_address.clone())
    //             .body(serde_json::to_vec(&tg_response)?)
    //             .send();
    //     }
    // }
    Ok(())
}

fn handle_message(
    our: &Address,
    subs: &mut Vec<Address>,
    state: &mut Option<State>,
) -> anyhow::Result<()> {
    let message = await_message()?;
    match message {
        Message::Request {
            ref body, source, ..
        } => {
            let _ = handle_request(our, subs, state, body, &source);
        }
        Message::Response { ref body, .. } => {
            let _ = handle_response(our, subs, state, body);
        }
    }
    Ok(())
}

call_init!(init);

fn init(our: Address) {
    println!("tg_bot: booted");
    let mut state = State::fetch();
    // TODO: Zena: Merge state and subs
    let mut subs: Vec<Address> = Vec::new();

    loop {
        match handle_message(&our, &mut subs, &mut state) {
            Ok(()) => {}
            Err(e) => {
                println!("tg: error: {:?}", e);
            }
        };
    }
}
