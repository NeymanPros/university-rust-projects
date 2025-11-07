use std::fs;
use serde_json::{json, Value};
use teloxide::Bot;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use teloxide::types::ParseMode::{Html};
use crate::UserStates;

/// Выбор модели
pub async fn show_model_selection(bot: &Bot, chat_id: ChatId) -> ResponseResult<()> {
    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback(
            "Meta Llama",
            "model:meta-llama/Meta-Llama-3-8B-Instruct"
        ),
        ],
        vec![InlineKeyboardButton::callback(
            "DeepSeek",
            "model:deepseek-ai/DeepSeek-V3.2-Exp"
        )
        ],
    ]);

    bot.send_message(
        chat_id,
        "🎓 <b>Добро пожаловать в образовательного бота!</b>\n\n\
                Я помогу вам с учёбой: решу задачи, объясню материал, проверю решение.\n\n\
                🤖 <b>Шаг 1/6:</b> Выберите языковую модель для работы:"
    )
        .reply_markup(keyboard)
        .parse_mode(Html)
        .await?;

    Ok(())
}

/// Выбор типа задачи
pub async fn show_question_type (bot: &Bot, chat_id: ChatId) -> ResponseResult<()> {
    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback(
            "Решить задачу",
            "type:решить задачу"
        )],
        vec![InlineKeyboardButton::callback(
            "Объяснить материал",
            "type:объяснить материал"
        )],
        vec![InlineKeyboardButton::callback(
            "Проверить решение",
            "type:проверить решение"
        )],
        vec![InlineKeyboardButton::callback(
            "Протестировать знания",
            "type:проверка знаний"
        )],

    ]);

    bot.send_message(
        chat_id,
        "🤖 <b>Шаг 2/6:</b> Выберите вид запроса:"
    )
        .reply_markup(keyboard)
        .parse_mode(Html)
        .await?;
    
    Ok(())
}

/// Выбор вида вопроса
pub async fn show_question_field (bot: &Bot, chat_id: ChatId) -> ResponseResult<()> {
    bot.send_message(
        chat_id,
        "🤖 <b>Шаг 3/6:</b> Введите область знания для вопроса:"
    )
        .parse_mode(Html)
        .await?;

    Ok(())
}

///Выбор раздела
pub async fn show_question_subfield (bot: &Bot, chat_id: ChatId, user_states: UserStates) -> ResponseResult<()> {
    let variants = generate_subfields(user_states, chat_id).await;
    
    bot.send_message(
        chat_id,
        "🤖 <b>Шаг 4/6:</b> Выберите интересующий вас раздел или введите свой:"
    )
        .reply_markup(variants)
        .parse_mode(Html)
        .await?;
    Ok(())
}

async fn generate_subfields(user_states: UserStates, chat_id: ChatId) -> InlineKeyboardMarkup  {
    let (model, field) = {
        let state = user_states.lock().expect("No lock");
        let data = state.get(&chat_id).expect("No chat id");
        (data.model.clone().unwrap(), data.field.clone().unwrap())
    };
    
    let auth = fs::read_to_string(
        "token.env".to_string()
    )
        .expect("No token file provided!")
        .split('\n')
        .nth(1)
        .expect("No nth")
        .trim()
        .to_string();

    let client = reqwest::Client::new();
    let response = client
        .post("https://router.huggingface.co/v1/chat/completions")
        .header("Authorization", auth)
        .json(&json!({
            "model": model,
            "messages": [{"role": "user", "content": field_prompt(field)}]
    }))
        .send()
        .await
        .expect("No send");
    

    let future_json: Value = response.json().await.expect("No json");
    let generated_text = future_json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("Ошибка получения ответа")
        .to_string();
    
    let options: Vec<Vec<InlineKeyboardButton>> = generated_text.split(',').map(|x| { 
        vec![
            InlineKeyboardButton::callback(
                x.to_lowercase(),
                format!("subfield:{}", x)
            )
        ]
    }).collect();
    
    InlineKeyboardMarkup::new(options)
}

fn field_prompt (field: String) -> String {
    format!("Ты - помощник для образовательного бота. 
    Пользователь выбрал область: {}
    Перечисли от 2 до 5 основных подразделов этой области.
    Каждое название должно иметь не больше 20 символов.
    Формат ответа: только список через запятую, без нумерации, без дополнительных слов/символов.", field)
}

pub async fn show_question_level(bot: &Bot, chat_id: ChatId) -> ResponseResult<()> {
    let options = InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback(
            "0 - уровень чайника",
                "level:0"
        )],
        vec![InlineKeyboardButton::callback(
            "5 - есть хорошее понимание",
            "level:5"
        )],
        vec![InlineKeyboardButton::callback(
            "10 - уровень эксперта",
            "level:10"
        )]
    ]);
    
    bot.send_message(
        chat_id,
        "🤖 <b>Шаг 5/6:</b> Выберите, насколько хорошо разбираетесь в теме, или введите своё число от 1 до 10:"
    )
        .reply_markup(options)
        .parse_mode(Html)
        .await?;
    
    Ok(())
}

pub async fn show_question_ask (bot: &Bot, chat_id: ChatId, user_states: UserStates) -> ResponseResult<()> {
    let ans = {
        let state = user_states.lock().expect("No lock");
        let current = state.get(&chat_id).expect("No answers");
        current.clone()
    };
    bot.send_message(
        chat_id,
        format!("🤖 <b>Последний шаг!</b>\n\
        Выбранные параметры:\nМодель: <i>{}</i>\nРаздел: <i>{} -> {}</i>\n\
        Уровень: <i>{}/10</i>\nЗапрос: <i>{}</i>\n<b>Введите сам вопрос:</b>
        ", ans.model.as_ref().unwrap(), ans.field.as_ref().unwrap(), ans.subfield.as_ref().unwrap(), ans.level.as_ref().unwrap(), ans.q_type.as_ref().unwrap())
    )
        .parse_mode(Html)
        .await?;
    
    Ok(())
}

pub async fn show_model_answer(bot: &Bot, chat_id: ChatId, user_states: UserStates, user_text: String) -> ResponseResult<()> {
    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback(
            "Вернуться в начало",
            "return:0"
        )],
        vec![InlineKeyboardButton::callback(
            "Задать другой вопрос",
            "return:5"
        )]
    ]);
    
    let prom = generate_prompt(user_states.clone(), &chat_id, &user_text);
    let model_answer = send_model_request(user_states, &chat_id, &prom).await;
    bot.send_message(
        chat_id,
        model_answer
    )
        .reply_markup(keyboard)
        .await?;
    
    Ok(())
}

async fn send_model_request(user_states: UserStates, chat_id: &ChatId, prom: &String) -> String {
    let model = {
        let state = user_states.lock().expect("No lock");
        let data = state.get(&chat_id).expect("No chat id");
        data.model.clone().unwrap()
    };

    let auth = fs::read_to_string(
        "token.env".to_string()
    )
        .expect("No token file provided!")
        .split('\n')
        .nth(1)
        .expect("No nth")
        .trim()
        .to_string();

    let client = reqwest::Client::new();
    let response = client
        .post("https://router.huggingface.co/v1/chat/completions")
        .header("Authorization", auth)
        .json(&json!({
            "model": model,
            "messages": [{"role": "user", "content": &prom}]
    }))
        .send()
        .await
        .expect("No send");


    let future_json: Value = response.json().await.expect("No json");
    println!("{}", future_json);
    let generated_text = future_json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("Ошибка получения ответа")
        .to_string();
    
    generated_text
}

fn generate_prompt(user_states: UserStates, chat_id: &ChatId, user_text: &String) -> String {
    let state = user_states.lock().expect("No lock");
    let ans = state.get(&chat_id).unwrap();
    format!("Ты - образовательный бот.\n\
        Твои параметры: Задача: {}.\n\
        Раздел науки: {} -> {}.\n\
        Уровень понимания пользователя: {}/10, где 10/10 - эксперт.\n\
        Вопрос пользователя: {}
        ", ans.q_type.as_ref().unwrap(), ans.field.as_ref().unwrap(), ans.subfield.as_ref().unwrap(), ans.level.as_ref().unwrap(), user_text)
}
