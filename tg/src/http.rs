use frankenstein::{MethodResponse, Update, UpdateContent};

use kinode_process_lib::{
    get_blob, Message, Request, Address,/* println,*/
    http::{HttpClientError, HttpClientResponse, WsMessageType, send_ws_push, HttpServerRequest},
};

use crate::State;
use crate::TgResponse;
use crate::request_no_wait;
use crate::data_to_ws_update_blob;
use crate::TgUpdate;

pub fn handle_http_server_request(
    _our: &Address,
    _source: &Address,
    body: &[u8],
    state: &mut Option<State>,
) -> anyhow::Result<()> {

    println!("tg: handle http server request");

    let Ok(server_request) = serde_json::from_slice::<HttpServerRequest>(body) else {
        return Ok(());
    };

    match server_request {
        HttpServerRequest::WebSocketOpen { channel_id, .. } => {
            println!("tg: web socket open");
            if let Some(state) = state {
                state.our_channel_id = channel_id;
                state.save();
            }
            Ok(())
        }
        _ => Ok(())
    }
}

pub fn handle_http_message(
    our: &Address,
    message: &Message,
    state: &mut Option<State>,
) -> anyhow::Result<()> {
    println!("tg: handle http message");
    
    match message {
        Message::Request { 
            ref body,
            ref source,
            ..
        } => handle_http_server_request(our, source, body, state),
        Message::Response {
            ref body,
            ref context,
            ..
        } => handle_http_response(state, body, context),
    }
}

fn handle_tg_update(
    state: &mut Option<State>,
    body: &[u8],
) -> anyhow::Result<()> {
    println!("tg: handle_tg_update");
    
    let HttpClientResponse::Http(_) =
        serde_json::from_slice::<Result<HttpClientResponse, HttpClientError>>(&body)??
    else {
        println!("unexpected response 1");
        return Err(anyhow::anyhow!("unexpected Response: "));
    };
    println!("tg: got http client response");
    let Some(state) = state else {
        return Err(anyhow::anyhow!("tg: state not initialized"));
    };
    if let Some(blob) = get_blob() {
        let Ok(response) = serde_json::from_slice::<MethodResponse<Vec<Update>>>(&blob.bytes)
        else {
            println!("unexpected response 2");
            return Err(anyhow::anyhow!("unexpected Response: "));
        };
        
        if let Some(update) = response.result.get(0) {
            match &update.content {
                UpdateContent::Message(msg) => {
                    // TODO more elegant way to get username and text
                    let username: String = match &msg.from {
                        Some(from) => 
                            if let Some(username) = &from.username {
                                username.to_string()
                            } else {"Unknown".to_string()}
                        None => "Unknown".to_string(),
                    };
                    let text: String = match &msg.text {
                        Some(text) => text.to_string(),
                        None => "Unknown".to_string(),
                    };

                    let blob = data_to_ws_update_blob(msg.chat.id, msg.message_id, msg.date, username, text);
                    println!("tg: pushing to WS");
                    send_ws_push(state.our_channel_id, WsMessageType::Text, blob);
                }
                _ => println!("tg: not a message"),
            }
        }
        println!("tg: forwarding to subs");
        for sub in state.subscribers.iter() {
            // println!("  - {:?}", sub);
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
    println!("tg: done");
    Ok(())
}

// fn receive_downloaded_audio() -> anyhow::Result<()> {
//     let bytes = get_blob()?.bytes;
//     // TODO: Just send a response with those bytes bro
//     Ok(())
// }

fn handle_http_response(
    state: &mut Option<State>,
    body: &[u8],
    _context: &Option<Vec<u8>>,
) -> anyhow::Result<()> {
    // match context {
    //     Some(context) => {
    //         // match context {
    //         //     0 => receive_downloaded_audio(),
    //         //     _ => return Err(anyhow::anyhow!("unexpected context")),
    //         // }
    //         let Some(state) = state else {
    //             return Err(anyhow::anyhow!("state not initialized"));
    //         };
    //         Ok(())
    //     }
    //     None => handle_tg_update(state, body),
    // }
    handle_tg_update(state, body)
}
