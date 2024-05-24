use std::str::FromStr;
use kinode_process_lib::{http::Method, http::send_request, sqlite, Address, LazyLoadBlob};
use kinode_process_lib::http::send_request_await_response;
// use kinode_process_lib::println;
use std::collections::HashMap;
use std::path::PathBuf;
use serde::Serialize;
use serde::Deserialize;
use frankenstein::TelegramApi;
use crate::NewMessageUpdate;

pub fn request_no_wait<T1: serde::ser::Serialize>(
    api_url: &str,
    method: &str,
    params: Option<T1>,
) -> Result<(), anyhow::Error> {
    println!("tg: request no wait");
    let url = format!("{}/{method}", api_url);
    let url = url::Url::from_str(&url)?;

    let headers: HashMap<String, String> =
        HashMap::from_iter([("Content-Type".into(), "application/json".into())]);

    let body = if let Some(ref params) = params {
        serde_json::to_vec(params)?
    } else {
        println!("tg: no params");
        Vec::new()
    };
    println!("tg: method: {:?}", method);
    println!("tg: body: {:?}", body);
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
        println!("tg: request");
        let url = format!("{}/{method}", self.api_url);
        let url = url::Url::from_str(&url)?;

        // content-type application/json
        let headers: HashMap<String, String> =
            HashMap::from_iter([("Content-Type".into(), "application/json".into())]);

        let body = if let Some(ref params) = params {
            serde_json::to_vec(params)?
        } else {
            println!("tg: no params");
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

pub fn data_to_ws_update_blob(
    chat_id: i64,
    message_id: i32,
    date: u64,
    username: String,
    text: String,
) -> LazyLoadBlob {
    LazyLoadBlob {
        mime: Some("application/json".to_string()),
        bytes: serde_json::json!({
            "NewMessageUpdate": NewMessageUpdate {
                chat_id,
                message_id,
                date,
                username,
                text,
            }
        })
        .to_string()
        .as_bytes()
        .to_vec(),
    }
}

// pub fn message_to_params
// pub fn insert_row(our: &Address, chat_id: i64, message_id: i32, date: u64, username: String, text: String) {
//     let db = sqlite::open(our.package_id(), "tg_chat", None).unwrap();
//     let insert_statement = "INSERT INTO tg_chat (chat_id, message_id, date, username, text) VALUES (?, ?, ?, ?, ?);".to_string();
//     let params = vec![
//         serde_json::Value::Number(chat_id.into()),
//         serde_json::Value::Number(message_id.into()),
//         serde_json::Value::Number(date.into()),
//         serde_json::Value::String(username),
//         serde_json::Value::String(text),
//     ];
//     db.write(insert_statement, params, None);
// }


// let select_statement = "SELECT * FROM tg_chat;".to_string();
// let rows = db.read(select_statement, vec![]).unwrap();
// // rows: Vec<HashMap<String, serde_json::Value>>
// println!("rows: {:#?}", rows);

// println!("tg: chatid: {}", &message.chat.id);
// if let Some(from) = &message.from {
//     println!("tg: from: {:?}", from.username); //TODO error handling when None
// }
// println!("tg: chat: {}", &message.message_id);
// println!("tg: chat: {}", &message.date);
// println!("tg: chat: {:?}", &message.text); // TODO error handling when None
// // println!("tg: chat: {}", msg.voice); TODO
