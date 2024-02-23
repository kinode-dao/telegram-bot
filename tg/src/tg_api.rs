/// API for the bot and the parent process.
use crate::TgInitialize;
use frankenstein::{GetUpdatesParams, TelegramApi};
use kinode_process_lib::{
    http::{send_request, send_request_await_response, Method},
    our_capabilities, spawn, Address, OnExit, ProcessId, Request,
};
use std::collections::HashMap;
use std::{path::PathBuf, str::FromStr};

static BASE_API_URL: &str = "https://api.telegram.org/bot";

/// function to spawn and initialize a tg bot.
/// call this from your parent process to receive updates!
#[allow(unused)]
pub fn init_tg_bot(
    our: Address,
    token: &str,
    params: Option<GetUpdatesParams>,
) -> anyhow::Result<(Api, ProcessId)> {
    let tg_bot_wasm_path = format!("{}/pkg/tg.wasm", our.package_id());

    let our_caps = our_capabilities();

    let process_id = spawn(
        None,
        &tg_bot_wasm_path,
        OnExit::None,
        our_caps,
        vec![],
        false,
    )?;

    let api = Api::new(token, our.clone());
    let init = TgInitialize {
        token: token.to_string(),
        params,
    };

    let _ = Request::new()
        .target(Address {
            node: our.node.clone(),
            process: process_id.clone(),
        })
        .body(serde_json::to_vec(&init)?)
        .send();

    Ok((api, process_id))
}

pub struct Api {
    pub api_url: String,
    pub our: Address,
    pub current_offset: u32,
}

impl Api {
    #[must_use]
    pub fn new(api_key: &str, our: Address) -> Self {
        let api_url = format!("{BASE_API_URL}{api_key}");
        Self {
            api_url,
            our,
            current_offset: 0,
        }
    }
}

impl TelegramApi for Api {
    type Error = anyhow::Error;

    fn request<T1: serde::ser::Serialize, T2: serde::de::DeserializeOwned>(
        &self,
        method: &str,
        params: Option<T1>,
    ) -> Result<T2, anyhow::Error> {
        let url = format!("{}/{method}", self.api_url);
        let url = url::Url::from_str(&url)?;

        // content-type application/json
        let headers: HashMap<String, String> =
            HashMap::from_iter([("Content-Type".into(), "application/json".into())]);

        let body = if let Some(ref params) = params {
            serde_json::to_vec(params)?
        } else {
            Vec::new()
        };
        let res = send_request_await_response(Method::GET, url, Some(headers), 30, body)?;

        let deserialized: T2 = serde_json::from_slice(&res.body())
            .map_err(|e| anyhow::anyhow!("Failed to deserialize response body: {}", e))?;

        Ok(deserialized)
    }

    fn request_with_form_data<T1: serde::ser::Serialize, T2: serde::de::DeserializeOwned>(
        &self,
        _method: &str,
        _params: T1,
        _files: Vec<(&str, PathBuf)>,
    ) -> Result<T2, anyhow::Error> {
        return Err(anyhow::anyhow!(
            "tgbot doesn't support multipart uploads (yet!)"
        ));
    }
}

impl Api {
    pub fn request_no_wait<T1: serde::ser::Serialize>(
        &self,
        method: &str,
        params: Option<T1>,
    ) -> Result<(), anyhow::Error> {
        let url = format!("{}/{method}", self.api_url);
        let url = url::Url::from_str(&url)?;

        // content-type application/json
        let headers: HashMap<String, String> =
            HashMap::from_iter([("Content-Type".into(), "application/json".into())]);

        let body = if let Some(ref params) = params {
            serde_json::to_vec(params)?
        } else {
            Vec::new()
        };
        send_request(Method::GET, url, Some(headers), Some(20), body);
        Ok(())
    }
}
