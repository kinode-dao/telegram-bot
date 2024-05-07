/// API for the bot and the parent process.
use frankenstein::{GetUpdatesParams, TelegramApi, Update};
use kinode_process_lib::{
    http::{send_request, send_request_await_response, Method},
    our_capabilities, spawn, Address, OnExit, ProcessId, Request,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{path::PathBuf, str::FromStr};


#[derive(Debug, Serialize, Deserialize)]
pub enum TgRequest {
    RegisterApiKey(TgInitialize),
    Subscribe,
    Unsubscribe,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TgInitialize {
    pub token: String,
    pub params: Option<GetUpdatesParams>,
}

/// Enum Request received by parent process for long-polling updates.
#[derive(Debug, Serialize, Deserialize)]
pub enum TgResponse {
    Ok, 
    Update(TgUpdate),
    Error(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TgUpdate {
    pub updates: Vec<Update>,
}


