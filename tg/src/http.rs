use frankenstein::{MethodResponse, Update};

use kinode_process_lib::{
    await_message, call_init, get_blob,
    http::{HttpClientError, HttpClientResponse},
    println, Address, Message, Request, Response,
};

pub fn handle_http_message(message: &Message, state: &mut Option<State>) -> anyhow::Result<()> {
    match message {
        Message::Request { ref body, .. } => Ok(()),
        Message::Response {
            ref body,
            ref context,
            ..
        } => handle_http_response(state, body, context),
    }
}

fn handle_tg_update(state: &mut Option<State>, body: &[u8]) -> anyhow::Result<()> {
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

fn receive_downloaded_audio() -> anyhow::Result<()> {
    let bytes = get_blob()?.bytes;
    // TODO: Just send a response with those bytes bro
    Ok(())
}

fn handle_http_response(
    state: &mut Option<State>,
    body: &[u8],
    context: &Option<Vec<u8>>,
) -> anyhow::Result<()> {
    match context {
        Some(context) => {
            match context {
                0 => receive_downloaded_audio(),
                _ => anyhow::anyhow!("unexpected context"),
            }
            let Some(state) = state else {
                return Err(anyhow::anyhow!("state not initialized"));
            };
            Ok(())
        }
        None => handle_tg_update(state, body),
    }
}
