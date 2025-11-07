mod bot_logic;
mod show_question;
mod handle_text;

use std::collections::HashMap;
use bot_logic::*;
use std::fs;
use std::sync::{Arc, Mutex};
use teloxide::Bot;
use teloxide::prelude::*;

#[derive(Clone, Default)]
struct State {
    depth: u8,
    model: Option<String>,
    q_type: Option<String>, 
    field: Option<String>, 
    subfield: Option<String>, 
    level: Option<u8>
}

/// Состояния диалога
/// 
/// 0-Start, 1-Model, 2-QuestionType, 3-Field, 4-SubField. 5-Level, 6-End
type UserStates = Arc<Mutex<HashMap<ChatId, State>>>;

fn get_unlocked(user_states: UserStates, chat_id: ChatId) -> u8 {
    let mut state = user_states.lock().expect("No lock");
    if let Some(data) = state.get(&chat_id) {
        data.depth.clone()
    }
    else {
        state.insert(chat_id, State::default());
        0u8
    }
}


#[tokio::main]
async fn main() {
    unsafe {
        std::env::set_var("RUST_LOG".to_string(), "info".to_string());
    }
    pretty_env_logger::init();
    let token = fs::read_to_string(
        "token.env".to_string()
    )
        .expect("No token file provided!")
        .split('\n')
        .nth(0)
        .expect("No nth")
        .trim()
        .to_string();
    
    let bot = Bot::new(token);
    log::info!("Запуск образовательного бота...");

    let user_states: UserStates = Arc::new(Mutex::new(HashMap::new()));

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(handle_message))
        .branch(Update::filter_callback_query().endpoint(handle_callback));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![user_states])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
