use frankenstein::TelegramApi;
use kinode_process_lib::get_blob;
use std::path::PathBuf;

use kinode_process_lib::{
    http::{
        send_request_await_response, HeaderMap, HttpClientAction, HttpClientError,
        HttpClientRequest, HttpClientResponse, HttpServerError, OutgoingHttpRequest,
    },
    Address, LazyLoadBlob, Message, Request,
};
use std::collections::HashMap;

static BASE_API_URL: &str = "https://api.telegram.org/bot";

pub struct Api {
    pub api_url: String,
    pub our: Address,
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
        Self { api_url, our }
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

        let mut req = Request::new()
            .target(("our", "http_client", "distro", "sys"))
            .body(serde_json::to_vec(&HttpClientAction::Http(
                OutgoingHttpRequest {
                    method: "GET".to_string(),
                    version: None,
                    url,
                    headers: HashMap::from_iter([(
                        "Content-Type".into(),
                        "application/json".into(),
                    )]),
                },
            ))?);

        // if some params to add, add them to blob.
        if let Some(ref params) = params {
            req = req.blob_bytes(serde_json::to_vec(params)?);
        };
        let msg = req.send_and_await_response(5)??;

        let response_body = match msg {
            Message::Response { body, .. } => {
                let response = serde_json::from_slice::<HttpClientResponse>(&body)?;

                if let HttpClientResponse::Http(res) = response {
                    let blob = get_blob();
                    if let Some(blob) = blob {
                        blob.bytes
                    } else {
                        return Err(anyhow::anyhow!("no blob found for http client response!"));
                    }
                } else {
                    return Err(anyhow::anyhow!("got unexpected response from http_client!"));
                }
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "got unexpected non-response from http_client!"
                ))
            }
        };

        serde_json::from_slice(&response_body).map_err(Into::into)
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
