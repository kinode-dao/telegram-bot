use serde::{Deserialize, Serialize};
use std::str::FromStr;

use kinode_process_lib::{
    await_message, call_init, println, Address, Message, ProcessId, Request, Response,
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
enum TgRequest {
    Test,
}

#[derive(Debug, Serialize, Deserialize)]
enum TgResponse {
    Ack,
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
                Response::new()
                    .body(serde_json::to_vec(&TgResponse::Ack).unwrap())
                    .send()
                    .unwrap();
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
