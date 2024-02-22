use frankenstein::TelegramApi;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use std::thread::sleep;
use std::time::{Duration, Instant};

use kinode_process_lib::{await_message, call_init, println, timer, Address, Message};

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
                    offset: Some(21),
                    limit: Some(10),
                    timeout: Some(30),
                    allowed_updates: None,
                };

                let members_params = frankenstein::GetChatMemberCountParams {
                    chat_id: frankenstein::ChatId::Integer(6856598744),
                };
                let res = api.as_mut().unwrap().get_updates(&updates_params)?;
                println!("got resss: {:?}", res);

                let reply_params = frankenstein::ReplyParameters {
                    chat_id: Some(frankenstein::ChatId::Integer(6856598744)),
                    message_id: res.result[0].update_id as i32,
                    allow_sending_without_reply: Some(true),
                    quote: None,
                    quote_entities: None,
                    quote_parse_mode: None,
                    quote_position: None,
                };

                let send_params = frankenstein::SendMessageParams {
                    chat_id: frankenstein::ChatId::Integer(6856598744),
                    text: "yes I see you!".to_string(),
                    disable_notification: None,
                    entities: None,
                    message_thread_id: None,
                    link_preview_options: None,
                    parse_mode: None,
                    protect_content: None,
                    reply_markup: None,
                    reply_parameters: Some(reply_params),
                };

                let res = api.as_mut().unwrap().send_message(&send_params);
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
