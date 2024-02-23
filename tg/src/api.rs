use frankenstein::TelegramApi;
use std::{path::PathBuf, str::FromStr};

use kinode_process_lib::{
    http::{send_request, send_request_await_response, Method},
    Address,
};
use std::collections::HashMap;

static BASE_API_URL: &str = "https://api.telegram.org/bot";

pub struct Api {
    pub api_url: String,
    pub our: Address,
    pub current_offset: u32,
}

// #[derive(Debug)] define custom errors!
// pub enum Error {
//     HttpError(HttpError),
//     ApiError(ErrorResponse),
// }

#[derive(Debug)]
pub struct HttpError {
    pub code: u16,
    pub message: String,
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
