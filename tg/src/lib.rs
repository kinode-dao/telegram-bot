use frankenstein::{GetUpdatesParams, MethodResponse, Update};

use kinode_process_lib::{
    await_message, call_init, get_blob,
    http::{HttpClientError, HttpClientResponse},
    println, Address, Message, Request,
};

wit_bindgen::generate!({
    path: "wit",
    world: "process",
    exports: {
        world: Component,
    },
});

use telegram_interface::{Api, TgInitialize, TgResponse, TgUpdate};

// use telegram_interface::api::{Api, TgInitialize, TgResponse, TgUpdate};

fn handle_message(
    our: &Address,
    api: &mut Option<Api>,
    parent: &mut Option<Address>,
) -> anyhow::Result<()> {
    let message = await_message()?;

    match message {
        Message::Response { body, .. } => {
            let response =
                serde_json::from_slice::<Result<HttpClientResponse, HttpClientError>>(&body)??;

            if let HttpClientResponse::Http(_) = response {
                if let Some(blob) = get_blob() {
                    if let Ok(response) =
                        serde_json::from_slice::<MethodResponse<Vec<Update>>>(&blob.bytes)
                    {
                        // forward to parent
                        if let Some(parent) = parent {
                            let request = TgUpdate {
                                updates: response.result.clone(),
                            };

                            let tg_response = TgResponse::Update(request);
                            let _ = Request::new()
                                .target(parent.clone())
                                .body(serde_json::to_vec(&tg_response)?)
                                .send();
                        }

                        if let Some(api) = api {
                            // set api.current_offset based on the response, keep same if no updates
                            let next_offset = response
                                .result
                                .last()
                                .map(|u| u.update_id + 1)
                                .unwrap_or(api.current_offset);
                            api.current_offset = next_offset;

                            let updates_params = frankenstein::GetUpdatesParams {
                                offset: Some(api.current_offset as i64),
                                limit: None,
                                timeout: Some(15),
                                allowed_updates: None,
                            };

                            api.request_no_wait("getUpdates", Some(updates_params))?;
                        }
                    }
                } else {
                    if let Some(ref parent_address) = parent {
                        let error_message = format!(
                            "tg_bot, failed to serialize response: {:?}",
                            std::str::from_utf8(&body).unwrap_or("[Invalid UTF-8]")
                        );
                        let tg_response = TgResponse::Error(error_message);
                        let _ = Request::new()
                            .target(parent_address.clone())
                            .body(serde_json::to_vec(&tg_response)?)
                            .send();
                    }
                }
            } else {
                return Err(anyhow::anyhow!("unexpected Response: "));
            }
        }
        Message::Request {
            ref body, source, ..
        } => match serde_json::from_slice(body)? {
            TgInitialize { token, params } => {
                if source.node != our.node {
                    return Err(anyhow::anyhow!(
                        "got initialize request from foreign source {:?}",
                        source
                    ));
                }
                let new_api = Api::new(&token, our.clone());

                let updates_params = params.unwrap_or(GetUpdatesParams {
                    offset: Some(new_api.current_offset as i64),
                    limit: None,
                    timeout: Some(15),
                    allowed_updates: None,
                });

                new_api.request_no_wait("getUpdates", Some(updates_params))?;

                *parent = Some(source);
                *api = Some(new_api);
            }
        },
    }
    Ok(())
}

call_init!(init);

fn init(our: Address) {
    println!("tg_bot: booted");
    // boot uninitialized, wait for initialize command.
    let mut api: Option<Api> = None;
    let mut parent: Option<Address> = None;

    loop {
        match handle_message(&our, &mut api, &mut parent) {
            Ok(()) => {}
            Err(e) => {
                println!("tg: error: {:?}", e);
            }
        };
    }
}
