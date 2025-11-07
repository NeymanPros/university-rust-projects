use teloxide::Bot;
use teloxide::prelude::*;
use teloxide::types::ParseMode::Html;
use crate::UserStates;
use crate::show_question::*;

pub async fn handle_message_field(bot: &Bot, msg: Message, user_states: UserStates) -> ResponseResult<()> {
    if let Some(field) = msg.text() {
        bot.send_message(
            msg.chat.id,
            format!("✅ Область знаний выбрана: <b>{}</b>", field)
        )
            .parse_mode(Html)
            .await?;
        
        bot.send_message(
            msg.chat.id,
            "Подождите, ответ генерируется..."
        ).await?;

        let handle = show_question_subfield(bot, msg.chat.id, user_states.clone());
        
        {
            let mut state = user_states.lock().expect("No lock");
            let changing = state.get_mut(&msg.chat.id).unwrap();
            changing.depth = 3;
            changing.field = Some(field.to_string());
        }
        
        handle.await?;
    }
    Ok(())
}

pub async fn handle_message_subfield(bot: &Bot, msg: Message, user_states: UserStates) -> ResponseResult<()> {
    if let Some(subfield) = msg.text() {
        bot.send_message(
            msg.chat.id,
            format!("✅ Выбранный раздел: <b>{}</b>", subfield)
        )
            .parse_mode(Html)
            .await?;
        
        let handle = show_question_level(bot, msg.chat.id);

        {
            let mut stage = user_states.lock().expect("No lock");
            let changing = stage.get_mut(&msg.chat.id).unwrap();
            changing.depth = 4;
            changing.subfield = Some(subfield.to_string());
        }
        
        handle.await?
    }
    Ok(())
}

pub async fn handle_message_level(bot: &Bot, msg: Message, user_states: UserStates) -> ResponseResult<()> {
    if let Some(level) = msg.text() {
        if let Ok(level_u8) = level.parse::<u8>() && level_u8 <= 10 {
            bot.send_message(
                msg.chat.id,
                format!("✅ Выбранный уровень: <b>{}</b>", level)
            )
                .parse_mode(Html)
                .await?;

            {
                let mut stage = user_states.lock().expect("No lock");
                let changing = stage.get_mut(&msg.chat.id).unwrap();
                changing.depth = 5;
                changing.level = Some(level_u8);
            }

            show_question_ask(bot, msg.chat.id, user_states.clone()).await?
        }
    }
    Ok(())
}

pub async fn handle_message_ask_model(bot: &Bot, msg: Message, user_states: UserStates) -> ResponseResult<()> {
    if let Some(user_text) = msg.text() {
        bot.send_message(
            msg.chat.id,
            "✅ Запрос отправлен! Подождите, ответ генерируется..."
        )
            .await?;

        show_model_answer(bot, msg.chat.id, user_states.clone(), user_text.to_string()).await?;
        let mut states = user_states.lock().expect("No lock");
        let state = states.get_mut(&msg.chat.id).expect("No mut");
        state.depth = 6;
    }
    Ok(())
}
