use frankenstein::{MethodResponse, TelegramApi, Update};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use kinode_process_lib::{
    await_message, call_init, get_blob, http::HttpClientResponse, println, timer, Address, Message,
};

wit_bindgen::generate!({
    path: "wit",
    world: "process",
    exports: {
        world: Component,
    },
});

mod api;
use api::Api;

#[derive(Debug, Serialize, Deserialize)]
// #[serde_untagged]
enum TgRequest {
    Initialize { token: String },
    Hello,
}

// start process -> maintain loop -> increment offset ->
//   get_updates() -> forward -> offset increment -> get_updates() -> get_updates
//   expires?
//   get_chat_members() -> http_call() ->
fn handle_message(our: &Address, api: &mut Option<Api>) -> anyhow::Result<()> {
    let message = await_message()?;

    match message {
        Message::Response { body, context, .. } => {
            let response = serde_json::from_slice::<HttpClientResponse>(&body)?;
            if let HttpClientResponse::Http(response) = response {
                println!("got respose with status: {:?}", response.status);
                // give this back to the dawg that requested this? the parent?
                let blob = get_blob();
                if let Some(blob) = blob {
                    let response =
                        serde_json::from_slice::<MethodResponse<Vec<Update>>>(&blob.bytes)?;
                    println!("got response !: {:?}", response);
                }
            } else {
                return Err(anyhow::anyhow!("unexpected Response: "));
            }
        }
        Message::Request { ref body, .. } => match serde_json::from_slice(body)? {
            TgRequest::Initialize { token } => {
                let mut new_api = Api::new(&token, our.clone());

                let updates_params = frankenstein::GetUpdatesParams {
                    offset: Some(new_api.current_offset),
                    limit: None,
                    timeout: Some(30),
                    allowed_updates: None,
                };

                new_api.request_no_wait("GET", Some(updates_params))?;
                new_api.current_offset += 1;

                *api = Some(new_api);
            }
            TgRequest::Hello => {
                let members_params = frankenstein::GetChatMemberCountParams {
                    chat_id: frankenstein::ChatId::Integer(6856598744),
                };
                let res = api
                    .as_mut()
                    .unwrap()
                    .get_chat_member_count(&members_params)?;
                println!("got response when sending: {:?}", res);
            }
        },
    }
    Ok(())
}

call_init!(init);

fn init(our: Address) {
    println!("tg: begin");

    // boot uninitialized, wait for initialize command.
    let mut api: Option<Api> = None;
    loop {
        match handle_message(&our, &mut api) {
            Ok(()) => {}
            Err(e) => {
                println!("tg: error: {:?}", e);
            }
        };
    }
}
