use lazy_static::lazy_static;
use rand::{thread_rng, distributions::Alphanumeric, Rng};
use teloxide::{prelude::*,utils::command::BotCommands, types::{InputFile, InlineKeyboardButton, InlineKeyboardMarkup, InputMessageContent, InlineQueryResultArticle, InputMessageContentText}};
use std::{error::Error, sync::Mutex, collections::HashMap, time::Duration, thread};
lazy_static! {
    static ref SEARCHING:Mutex<Vec<teloxide::types::ChatId>> = Mutex::new(Vec::new());
    static ref BUSY:Mutex<Vec<teloxide::types::ChatId>> = Mutex::new(Vec::new());
    static ref PARTNERS :Mutex<HashMap<String,Vec<ChatId>>> = Mutex::new(HashMap::new());
}
#[tokio::main]
async fn main()-> Result<(), Box<dyn Error>>{
    println!("\x1b[93mStarting bot...\x1b[0m");
    dotenv::dotenv().ok();
    let bot = Bot::from_env().auto_send();
    let tel = bot.clone();
    tokio::spawn(async{
        finding(tel).await.unwrap();
    });
    let handler = dptree::entry()
    .branch(Update::filter_message().endpoint(chat))
    .branch(Update::filter_callback_query().endpoint(callback_handler))
    .branch(Update::filter_inline_query().endpoint(inline_query_handler));
    Dispatcher::builder(bot, handler).build().setup_ctrlc_handler().dispatch().await;
    Ok(())
}
#[derive(BotCommands)]
#[command(rename = "lowercase", description = "دستورهای قابل استفاده:")]
enum Command {
    #[command(description = "help")]
    Help,
    #[command(description = "پیدا کردن یک شریک جدید")]
    Search,
    #[command(description = "توقف گفتگو")]
    Stop,
    #[command(description = "گفتگو جدید")]
    Next,
    Start,
}
impl From<&str> for Command{
    fn from(s:&str)->Self{
        match s {
            "/help"=>Command::Help,
            "/search"=>Command::Search,
            "/stop" => Command::Stop,
            "/next" => Command::Next,
            "/start" => Command::Start,
            _=>Command::Help
        }
    }
}
async fn finding(bot:AutoSend<Bot>) -> Result<(), Box<dyn Error + Send + Sync>>{
    loop{
        thread::sleep(Duration::from_millis(500));
        let search = SEARCHING.lock().unwrap().clone();
        if search.len() >= 2{
            let random_one = search.len()-1;
            let chat_id_one = search.clone().into_iter().nth(random_one).unwrap();
            SEARCHING.lock().unwrap().remove(search.clone().into_iter().position(|x| x == search[random_one]).unwrap());
            drop(search);
            let search = SEARCHING.lock().unwrap().clone();
            let random_two = search.len()-1;
            let chat_id_two = search.clone().into_iter().nth(random_two).unwrap();
            SEARCHING.lock().unwrap().remove( search.clone().into_iter().position(|x| x == search[random_two]).unwrap());
            let room_id:String = thread_rng().sample_iter(&Alphanumeric).take(35).map(char::from).collect::<String>();
            PARTNERS.lock().unwrap().insert(room_id, vec![chat_id_two,chat_id_one]);
            println!("{:?}",PARTNERS.lock().unwrap());
            BUSY.lock().unwrap().push(chat_id_one);
            BUSY.lock().unwrap().push(chat_id_two);
            bot.send_message(chat_id_one, "🤖 شریک شما پیدا شد!").await?;
            bot.send_message(chat_id_two, "🤖 شریک شما پیدا شد!").await?;
            let keyboard = next_stop_button();
            bot.send_message(chat_id_one, "ℹ️💢 کلیدهای کمکی 💢ℹ️").reply_markup(keyboard.clone()).await?;
            bot.send_message(chat_id_two, "ℹ️💢 کلیدهای کمکی 💢ℹ️").reply_markup(keyboard).await?;
            return Ok(())
        } 
    }
}
fn find_key<'a>(map: &'a HashMap<String,Vec<ChatId>>, value: &Vec<ChatId>) -> Option<&'a String> {
    map.iter().find_map(|(key, val)| if &val == &value { Some(key) } else { None })
}
async fn chat(
    bot: AutoSend<Bot>,
    message: Message,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    match message.text(){
        Some(text)=>{
            if text.starts_with("/"){
                match Command::from(message.text().unwrap()){
                    Command::Help => {
                        bot.send_message(message.chat.id, Command::descriptions().to_string()).await?;
                    }
                    Command::Search =>search(bot, message.chat.id).await?,
                    Command::Next=>next(bot, message).await?,
                    Command::Stop=>stop(bot, message).await?,
                    Command::Start =>{
                        let keyboard = start_button();
                        bot.send_message(message.chat.id, "🔎 پیدا کردن شریک 🔎").reply_markup(keyboard).await?;
                    }
                };
            }else{
                let busy = BUSY.lock().unwrap().clone();
                for i in busy{
                    if i == message.chat.id{
                        let rooms = PARTNERS.lock().unwrap().clone();
                        let values = rooms.values().into_iter().find(|x| x.contains(&message.chat.id)).unwrap();
                        let partner = if &message.chat.id == values.into_iter().nth(0).unwrap(){
                            values.into_iter().nth(1).unwrap()
                        }else{
                            values.into_iter().nth(0).unwrap()
                        };
                        bot.send_message(partner.to_owned(), text).await?;
                    };
                };
            }
        },
        None=>{
            let busy = BUSY.lock().unwrap().clone();
            for i in busy{
                if i == message.chat.id{
                    let rooms = PARTNERS.lock().unwrap().clone();
                    let values = rooms.values().into_iter().find(|x| x.contains(&message.chat.id)).unwrap();
                    let partner = if &message.chat.id == values.into_iter().nth(0).unwrap(){
                        values.into_iter().nth(1).unwrap()
                    }else{
                        values.into_iter().nth(0).unwrap()
                    };
                    if message.photo().is_some(){
                        let photo_id = message.photo().unwrap().iter().last().unwrap().file_id.to_string();
                        bot.send_photo(partner.to_owned(), InputFile::file_id(photo_id)).await?;
                    }else if message.audio().is_some(){
                        let audio_id =  message.audio().unwrap().file_id.to_string();
                        bot.send_audio(partner.to_owned(), InputFile::file_id(audio_id)).await?;
                    }else if message.video().is_some(){
                        let video_id = message.video().unwrap().file_id.to_string();
                        bot.send_video(partner.to_owned(), InputFile::file_id(video_id)).await?;
                    }else if message.voice().is_some(){
                        let voice_id = message.voice().unwrap().file_id.to_string();
                        bot.send_voice(partner.to_owned(), InputFile::file_id(voice_id)).await?;
                    }else if message.document().is_some(){
                        let document_id = message.document().unwrap().file_id.to_string();
                        bot.send_document(partner.to_owned(), InputFile::file_id(document_id)).await?;
                    }else if message.sticker().is_some(){
                        let sticker_id = message.sticker().unwrap().file_id.to_string();
                        bot.send_sticker(partner.to_owned(), InputFile::file_id(sticker_id)).await?;
                    }else if message.voice().is_some(){
                        let voice_id = message.voice().unwrap().file_id.to_string();
                        bot.send_voice(partner.to_owned(), InputFile::file_id(voice_id)).await?;
                    }else if message.video_note().is_some(){
                        let video_note = message.video_note().unwrap().file_id.to_string();
                        bot.send_voice(partner.to_owned(), InputFile::file_id(video_note)).await?;
                    }else if message.url().is_some(){
                        bot.send_message(message.chat.id, "🤖 شما نمیتواند هیچ لینکی ارسال کنید").await?;
                    } else {
                        bot.send_message(message.chat.id, "🤖 پیام مورد نظر قابل ارسال نیست").await?;
                    }
                };
            };
        }
    }
    Ok(())
}
fn start_button() -> InlineKeyboardMarkup {
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = vec![];
    let values = [
        "جست و جو",
    ];
    for buttons in values.chunks(3) {
        let row = buttons
            .iter()
            .map(|&buttons| InlineKeyboardButton::callback(buttons.to_owned(), buttons.to_owned()))
            .collect();
        keyboard.push(row);
    }
    InlineKeyboardMarkup::new(keyboard)
}
fn next_stop_button() -> InlineKeyboardMarkup {
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = vec![];
    let values = [
        "بعدی","توقف"
    ];
    for buttons in values.chunks(3) {
        let row = buttons
            .iter()
            .map(|&buttons| InlineKeyboardButton::callback(buttons.to_owned(), buttons.to_owned()))
            .collect();
        keyboard.push(row);
    }
    InlineKeyboardMarkup::new(keyboard)
}
async fn inline_query_handler(
    q: InlineQuery,
    bot: AutoSend<Bot>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let start = InlineQueryResultArticle::new(
        "0",
        "searching",
        InputMessageContent::Text(InputMessageContentText::new("شروع جست و جو:")),
    )
    .reply_markup(start_button());
    let next_stop = InlineQueryResultArticle::new(
        "1",
        "next and stop",
        InputMessageContent::Text(InputMessageContentText::new("کلیدهای کمکی:")),
    )
    .reply_markup(next_stop_button());
    bot.answer_inline_query(q.id, vec![start.into(),next_stop.into()]).await?;
    Ok(())
}
async fn callback_handler(
    q: CallbackQuery,
    bot: AutoSend<Bot>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(command) = q.data {
        println!("{command}");
        match q.message {
            Some(message) => {
                bot.delete_message(message.chat.id, message.id).await?;
                match command.as_str(){
                    "جست و جو" =>search(bot, message.chat.id).await?,
                    "بعدی" =>next(bot, message).await?,
                    "توقف" =>stop(bot, message).await?,
                    _ =>{},
                };
            }
            None => {
                if let Some(_) = q.inline_message_id {
                }
            }
        }
    }
    Ok(())
}
async fn stop(bot:AutoSend<Bot>,message: Message)-> Result<(), Box<dyn Error + Send + Sync>>{
    let busy = BUSY.lock().unwrap().clone();
    if busy.contains(&message.chat.id){
        let rooms = PARTNERS.lock().unwrap().clone();
        let values = rooms.values().into_iter().find(|x| x.contains(&message.chat.id)).unwrap();
        let partner = if &message.chat.id == values.into_iter().nth(0).unwrap(){
            values.into_iter().nth(1).unwrap()
    }else{
        values.into_iter().nth(0).unwrap()
    };
        let p_index = busy.clone().into_iter().position(|x| &x == partner).unwrap();
        let m_index = busy.into_iter().position(|x| &x == &message.chat.id).unwrap();
        BUSY.lock().unwrap().remove(p_index);
        BUSY.lock().unwrap().remove(m_index);
        match find_key(&rooms,values){
            Some(key)=>{
            PARTNERS.lock().unwrap().remove(key);
            },
            None=>{
                bot.send_message(message.chat.id, "🤖 شما درحال گفتگو نیستید").await?;
            }
        }
        bot.send_message(message.chat.id, "🤖  شما گفتگو را متوقف کردید").await?;
        bot.send_message(partner.to_owned(), "🤖 شریک شما گفتگو را متوقف کرد").await?;
        let keyboard = start_button();
        bot.send_message(message.chat.id, "🔎 پیدا کردن شریک 🔎").reply_markup(keyboard).await?;
    }else{
        bot.send_message(message.chat.id, "🤖 شما درحال گفتگو نیستید").await?;
    };
    Ok(())
}
async fn next(bot:AutoSend<Bot>,message: Message)-> Result<(), Box<dyn Error + Send + Sync>>{
    let busy = BUSY.lock().unwrap().clone();
    if busy.contains(&message.chat.id){
        let rooms = PARTNERS.lock().unwrap().clone();
        let values = rooms.values().into_iter().find(|x| x.contains(&message.chat.id)).unwrap();
        let partner = if &message.chat.id == values.into_iter().nth(0).unwrap(){
            values.into_iter().nth(1).unwrap()
        }else{
        values.into_iter().nth(0).unwrap()
        };
        let p_index = busy.clone().into_iter().position(|x| &x == partner).unwrap();
        let m_index = busy.into_iter().position(|x| &x == &message.chat.id).unwrap();
        BUSY.lock().unwrap().remove(p_index);
        BUSY.lock().unwrap().remove(m_index);
        match find_key(&rooms,values){
            Some(key)=>{
            PARTNERS.lock().unwrap().remove(key);
            },
            None=>{
                bot.send_message(message.chat.id, "🤖 شما درحال گفتگو نیستید").await?;
            }
        }
        SEARCHING.lock().unwrap().push(message.chat.id);
        bot.send_message(message.chat.id, "🤖  شما گفتگو را متوقف کردید").await?;
        bot.send_message(partner.to_owned(), "🤖 شریک شما گفتگو را متوقف کرد").await?;
    bot.send_message(message.chat.id, "🤖 در حال جست و جو برای شریک ...").await?;
    }else{
        bot.send_message(message.chat.id, "🤖 شما درحال گفتگو نیستید").await?;
    }
    Ok(())
}
async fn search(bot:AutoSend<Bot>,chat_id: ChatId)-> Result<(), Box<dyn Error + Send + Sync>>{
    let searching = SEARCHING.lock().unwrap().clone();
    match searching.clone().into_iter().find(|x| x == &chat_id){
        Some(_) => {
            bot.send_message(chat_id, "😒 چند بار میزنی دارم پیدا میکنم").await?;
        },
        None => {
            drop(searching);
            SEARCHING.lock().unwrap().push(chat_id);
            bot.send_message(chat_id, "🤖 در حال جست و جو برای شریک ...").await?;
        }
    }
    Ok(())
}