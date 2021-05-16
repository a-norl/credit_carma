use std::fs;
use std::io::Read;

use serenity::{
    async_trait,
    model::{gateway::Ready},
    prelude::*,
};

use csv;
struct Handler;

#[async_trait]
impl EventHandler for Handler{

    async fn ready(&self, _ctx: Context, _data_about_bot: Ready) {
        println!("Connected as {}:{}", _data_about_bot.user.name, _data_about_bot.user.discriminator)
    }

}

#[tokio::main]
async fn main() {
    println!("Program Started.");

    let mut token_file = fs::File::open("token").unwrap();
    let mut token_string = String::new();
    token_file.read_to_string(&mut token_string).unwrap();

    let mut client = Client::builder(&token_string)
        .event_handler(Handler)
        .await
        .expect("err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}