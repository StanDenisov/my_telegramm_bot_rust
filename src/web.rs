use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct NewLinkMessage {
    pub text: String,
    pub reply_to_message_id: i64,
    pub chat_id: i64,
}

impl NewLinkMessage {
    pub async fn new(text: String, reply_to_message_id: i64, chat_id: i64) -> Self {
        Self {
            text,
            reply_to_message_id,
            chat_id,
        }
    }
}

/// https://core.telegram.org/bots/api#message
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct WMessage {
    pub message_id: i64,
    pub text: String,
    pub chat: WChat,
}

/// https://core.telegram.org/bots/api#message
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
#[serde(rename(serialize = "edited_message", deserialize = "edited_message"))]
pub struct WEditedMessage {
    pub message_id: i64,
    pub text: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename(serialize = "chat", deserialize = "chat"))]
pub struct WChat {
    pub id: i64,
}

/// https://core.telegram.org/bots/api#update
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
#[serde(rename(serialize = "update", deserialize = "update"))]
pub struct WUpdate {
    pub update_id: i64,
    pub message: Option<WMessage>,
    pub edited_message: Option<WEditedMessage>,
}

/// https://core.telegram.org/bots/api#making-requests
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct Wrapper<T> {
    pub ok: bool,
    pub result: Option<T>,
    pub error_code: Option<i64>,
    pub description: Option<String>,
    pub parameters: Option<Value>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct ReplyKeyboardMarkup {
    pub keyboard: Vec<Vec<KeyboardButton>>,
}

impl ReplyKeyboardMarkup {
    pub fn new(keyboard: Vec<Vec<KeyboardButton>>) -> Self {
        Self { keyboard }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct KeyboardButton {
    pub text: String,
}

impl KeyboardButton {
    pub fn new(text: String) -> Self {
        Self { text }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct DeleteMessage {
    pub chat_id: i64,
    pub message_id: i64,
}

impl DeleteMessage {
    pub async fn new(chat_id: i64, message_id: i64) -> Self {
        Self {
            chat_id,
            message_id,
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct WButtons {
    pub chat_id: i64,
    pub text: String,
    pub reply_to_message_id: i64,
    pub reply_markup: ReplyKeyboardMarkup,
}

impl WButtons {
    pub fn new(
        chat_id: i64,
        text: String,
        reply_to_message_id: i64,
        reply_markup: ReplyKeyboardMarkup,
    ) -> Self {
        Self {
            chat_id,
            text,
            reply_to_message_id,
            reply_markup,
        }
    }
}
