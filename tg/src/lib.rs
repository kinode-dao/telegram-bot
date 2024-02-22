use frankenstein::TelegramApi;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use kinode_process_lib::{await_message, call_init, println, Address, Message};

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
enum TgRequest {
    Test,
    Initialize { token: String },
    Hello,
}

fn handle_message(our: &Address, api: &mut Option<Api>) -> anyhow::Result<()> {
    let message = await_message()?;

    match message {
        Message::Response { .. } => {
            return Err(anyhow::anyhow!("unexpected Response: {:?}", message));
        }
        Message::Request {
            ref source,
            ref body,
            ..
        } => match serde_json::from_slice(body)? {
            TgRequest::Test => {
                println!("test hello");
            }
            TgRequest::Initialize { token } => {
                let new_api = Api::new(&token, our.clone());
                *api = Some(new_api);
            }
            TgRequest::Hello => {
                let updates_params = frankenstein::GetUpdatesParams {
                    offset: None,
                    limit: Some(1),
                    timeout: None,
                    allowed_updates: None,
                };

                let members_params = frankenstein::GetChatMemberCountParams {
                    chat_id: frankenstein::ChatId::Integer(6856598744),
                };
                let res = api
                    .as_mut()
                    .unwrap()
                    .get_chat_member_count(&members_params)?;
                println!("got resss: {:?}", res);
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
