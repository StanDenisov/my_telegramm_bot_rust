use serde::{Deserialize, Serialize};
use serde_json::Value;

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
#[serde(rename_all = "snake_case")]
#[serde(rename(serialize = "callback_query", deserialize = "callback_query"))]
pub struct WCallbackQuery {
    pub data: String,
    pub message: WMessage,
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
    pub callback_query: Option<WCallbackQuery>,
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
pub struct InlineKeyboardMarkup {
    pub inline_keyboard: Vec<Vec<KeyboardButton>>,
}

impl InlineKeyboardMarkup {
    pub fn new(inline_keyboard: Vec<Vec<KeyboardButton>>) -> Self {
        Self { inline_keyboard }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct KeyboardButton {
    pub text: String,
    pub callback_data: String,
}

impl KeyboardButton {
    pub fn new(text: String, callback_data: String) -> Self {
        Self {
            text,
            callback_data,
        }
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
    pub reply_markup: InlineKeyboardMarkup,
}

impl WButtons {
    pub fn new(chat_id: i64, text: String, reply_markup: InlineKeyboardMarkup) -> Self {
        Self {
            chat_id,
            text,
            reply_markup,
        }
    }
}
