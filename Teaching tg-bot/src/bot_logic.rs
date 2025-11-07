use crate::show_question::*;
use crate::handle_text::*;
use crate::{State, UserStates};
use teloxide::Bot;
use teloxide::prelude::*;
use teloxide::types::MaybeInaccessibleMessage;
use teloxide::types::ParseMode::Html;

/// Обработчик текстовых сообщений
pub async fn handle_message(bot: Bot, msg: Message, user_states: UserStates) -> ResponseResult<()> {
    if let Some(text) = msg.text() {
        if text == "/start" || text == "/restart" {
            show_model_selection(&bot, msg.chat.id).await?;
            let mut state = user_states.lock().expect("No lock");
            state.insert(msg.chat.id, State::default());
            return Ok(())
        }
        match crate::get_unlocked(user_states.clone(), msg.chat.id) {
            0 => {
                angry_bot(&bot, msg.chat.id).await?;
                show_model_selection(&bot, msg.chat.id).await?
            }
            1 => {
                angry_bot(&bot, msg.chat.id).await?;
                show_question_type(&bot, msg.chat.id).await?
            }
            2 => {
                handle_message_field(&bot, msg, user_states).await?;
            }
            3 => {
                handle_message_subfield(&bot, msg, user_states).await?;
            }
            4 => {
                handle_message_level(&bot, msg, user_states).await?;
            }
            5 => {
                handle_message_ask_model(&bot, msg, user_states).await?;
            }
            _ => {}
        }
    }
    Ok(())
}

async fn angry_bot(bot: &Bot, chat_id: ChatId) -> ResponseResult<()>{
    bot.send_message(
        chat_id,
        "Выберите один из предложенных вариантов!"
    ).await?;
    Ok(())
}

/// Обработчик нажатий на кнопки
pub async fn handle_callback(bot: Bot, q: CallbackQuery, user_states: UserStates) -> ResponseResult<()> {
    if let Some(data) = &q.data {
        if let Some(msg) = &q.message {
            match crate::get_unlocked(user_states.clone(), msg.chat().id) {
                0 => {handle_model(data, msg, bot, &q, user_states).await?}
                1 => {handle_question_type(data, msg, bot, &q, user_states).await?}
                3 => {handle_subfield(data, msg, bot, &q, user_states).await?}
                4 => {handle_level(data, msg, bot, &q, user_states).await?}
                6 => {handle_ending(data, msg, bot, &q, user_states).await?}
                _ => {}
            };
        }
    }
    Ok(())
}

async fn handle_model(data: &String, msg: &MaybeInaccessibleMessage, bot: Bot, q: &CallbackQuery, user_states: UserStates) -> ResponseResult<()> {
    if data.starts_with("model:") {
        let model = data.strip_prefix("model:").unwrap();
        bot.answer_callback_query(q.id.clone()).await?;

        bot.send_message(
            msg.chat().id,
            format!("✅ Выбрана модель: <b>{}</b>", model)
        )
            .parse_mode(Html)
            .await?;
        show_question_type(&bot, msg.chat().id).await?;

        
        let mut state = user_states.lock().expect("No lock");
        let changing = state.get_mut(&msg.chat().id).unwrap();
        changing.depth = 1;
        changing.model = Some(model.to_string());
    }
    Ok(())
}

async fn handle_question_type(data: &String, msg: &MaybeInaccessibleMessage, bot: Bot, q: &CallbackQuery, user_states: UserStates) -> ResponseResult<()> {
    if data.starts_with("type:") {
        let q_type = data.strip_prefix("type:").unwrap();
        bot.answer_callback_query(q.id.clone()).await?;
        
        bot.send_message(
            msg.chat().id,
            format!("✅ Вид вопроса выбран: <b>{}</b>", q_type)
        )
            .parse_mode(Html)
            .await?;
        show_question_field(&bot, msg.chat().id).await?;
        
        let mut state = user_states.lock().expect("No lock");
        let changing = state.get_mut(&msg.chat().id).unwrap();
        changing.depth = 2;
        changing.q_type = Some(q_type.to_string());
    }
    Ok(())
}

async fn handle_subfield(data: &String, msg: &MaybeInaccessibleMessage, bot: Bot, q: &CallbackQuery, user_states: UserStates) -> ResponseResult<()> {
    if data.starts_with("subfield:"){
        let subfield = data.strip_prefix("subfield:").unwrap();
        bot.answer_callback_query(q.id.clone()).await?;
        
        bot.send_message(
            msg.chat().id,
            format!("✅ Вид вопроса выбран: <b>{}</b>", subfield)
        )
            .parse_mode(Html)
            .await?;
        
        show_question_level(&bot, msg.chat().id).await.unwrap_or_default();
        
        let mut stage = user_states.lock().expect("No lock");
        let changing = stage.get_mut(&msg.chat().id).unwrap();
        changing.depth = 4;
        changing.subfield = Some(subfield.to_string());
    }
    Ok(())
}

async fn handle_level(data: &String, msg: &MaybeInaccessibleMessage, bot: Bot, q: &CallbackQuery, user_states: UserStates) -> ResponseResult<()> {
    if data.starts_with("level:"){
        if let Ok(level) = data.strip_prefix("level:").unwrap().parse::<u8>() && level <= 10 {
            bot.answer_callback_query(q.id.clone()).await?;
            
            bot.send_message(
                msg.chat().id,
                format!("✅ Выбранный уровень: <b>{}</b>", level)
            )
                .parse_mode(Html)
                .await?;

            {
                let mut stage = user_states.lock().expect("No lock");
                let changing = stage.get_mut(&msg.chat().id).unwrap();
                changing.depth = 5;
                changing.level = Some(level);
            }
            show_question_ask(&bot, msg.chat().id, user_states.clone()).await?;
        }
    }
    
    Ok(())
}

async fn handle_ending(data: &String, msg: &MaybeInaccessibleMessage, bot: Bot, q: &CallbackQuery, user_states: UserStates) -> ResponseResult<()> {
    if data.starts_with("return:") {
        if let Ok(new_depth) = data.strip_prefix("return:").unwrap().parse::<u8>() {
            bot.answer_callback_query(q.id.clone()).await?;
            if new_depth == 0 {
                bot.send_message(
                    msg.chat().id,
                    "Вы перешли в начало!"
                )
                    .await?;
                {
                    let mut states = user_states.lock().expect("No lock");
                    states.remove(&msg.chat().id);
                }
                show_model_selection(&bot, msg.chat().id).await?;
            }
            else if new_depth == 5 {
                bot.send_message(
                    msg.chat().id,
                    "Вы можете задавать следующий вопрос:"
                )
                    .await?;
                {
                    let mut states = user_states.lock().expect("No lock");
                    let state = states.get_mut(&msg.chat().id).expect("No id");
                    state.depth = 5;
                }
                show_question_ask(&bot, msg.chat().id, user_states).await?;
            }
        }
    }
    Ok(())
}
