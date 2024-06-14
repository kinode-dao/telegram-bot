/// API for the bot and the parent process.
// TODO: Zena: We will have to make some kind of crawler that recurses through the imported types and reliably converts them to wit. Another solution might have to work too. 
use frankenstein::{GetFileParams, GetUpdatesParams, Message, SendMessageParams, Update, SendPhotoParams};
use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize)]
pub enum TgRequest {
    RegisterApiKey(TgInitialize),
    Subscribe,
    Unsubscribe,
    /// Download a file from telegram.
    GetFile(GetFileParams),
    /// Send a message to a chat.
    SendMessage(SendMessageParams),
    /// Send a photo to a chat.
    SendPhoto(SendPhotoParams),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TgResponse {
    Ok, 
    Update(TgUpdate),
    Error(String),
    /// Download a file from telegram. Blobs included
    GetFile,
    /// Send a message to a chat.
    SendMessage(Message),
    /// Send a photo to a chat.
    SendPhoto(Message),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TgInitialize {
    pub token: String,
    pub params: Option<GetUpdatesParams>,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct TgUpdate {
    pub updates: Vec<Update>,
}

