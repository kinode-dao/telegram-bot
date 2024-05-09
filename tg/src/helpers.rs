use std::str::FromStr;
use kinode_process_lib::http::Method;
use kinode_process_lib::http::send_request;
use std::collections::HashMap;
use std::path::PathBuf;
use kinode_process_lib::http::send_request_await_response;
use serde::Serialize;
use serde::Deserialize;
use frankenstein::TelegramApi;

pub fn request_no_wait<T1: serde::ser::Serialize>(
    api_url: &str,
    method: &str,
    params: Option<T1>,
) -> Result<(), anyhow::Error> {
    let url = format!("{}/{method}", api_url);
    let url = url::Url::from_str(&url)?;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Api {
    pub api_url: String,
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
        // TODO: Zena: This should never happen. We're serving multiple people, this is dangerous 
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