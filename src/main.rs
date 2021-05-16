use std::{fs};
use std::io::Read;

use serenity::{async_trait, model::{channel::Message, gateway::Ready}, prelude::*};

use csv;
use fastrand;
struct Handler;

#[async_trait]
impl EventHandler for Handler{

    async fn message(&self, ctx: Context, message: Message) {

        if !(message.content.contains("credit score")) || message.author.bot {return;}

        let user_id = message.author.id.as_u64();

        if !(check_csv(user_id).await.unwrap()) {
            let credit_score = initial_credit_score_gen(user_id).await.unwrap();
            
            if let Err(why) = message.reply_ping(ctx.http, format!("your credit score is {}", credit_score)).await  {
                println!("Error sending message: {:?}", why);
            }
        } else {
            let (credit_score, old_credit_score) = get_credit_score(user_id).await.unwrap();

            if let Err(why) = message.reply_ping(ctx.http, format!("your credit score is {}", credit_score)).await  {
                println!("Error sending message: {:?}", why);
            }
        }
    }

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

async fn check_csv(user_id : &u64) -> Result<bool, csv::Error>{
    let mut reader = csv::Reader::from_path("credit_scores.csv").unwrap();
    for record in reader.records() {
        let record = record?;
        let record_id :u64= record[0].parse().unwrap();
        if *user_id == record_id {
            return Ok(true)
        }
    }
    Ok(false)
}

async fn initial_credit_score_gen(user_id: &u64) -> Result<i16, csv::Error>{
    fastrand::seed(*user_id);
    let genned_score = fastrand::i16(0..1000);

    let mut cmd_writer = csv::WriterBuilder::new()
        .from_writer(fs::OpenOptions::new().append(true).open("credit_scores.csv").unwrap());

        cmd_writer.write_record(&[user_id.to_string(),genned_score.to_string(), 0.to_string()])?;

    Ok(genned_score)
}

async fn get_credit_score(user_id: &u64) -> Result<(i16, i16), csv::Error>{
    let mut reader = csv::Reader::from_path("credit_scores.csv").unwrap();
    for record in reader.records() {
        let record = record?;
        let record_id :u64= record[0].parse().unwrap();
        if *user_id == record_id {
            let credit_score: i16 = record[1].parse().unwrap();
            let credit_score_old: i16 = record[2].parse().unwrap();
            return Ok((credit_score,credit_score_old))
        }
    }
    //TODO update old credit score
    Ok((0,0))
}