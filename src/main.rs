use std::{fs};
use std::io::Read;

use serenity::{async_trait, model::{channel::Message, gateway::Ready}, prelude::*};

use csv;
use fastrand;
use tokio::time;

use std::time::{Duration, Instant};
struct Handler;

#[async_trait]
impl EventHandler for Handler{

    async fn message(&self, ctx: Context, message: Message) {

        if !(message.content.contains("credit score")) || message.author.bot {return;}

        let user_id = message.author.id.as_u64();
        let mut nickname = message.author.name.clone();
        if message.author_nick(&ctx.http).await.is_some(){
            nickname = message.author_nick(&ctx.http).await.unwrap();
        }


        if !(check_csv(user_id).await.unwrap()) {
            let credit_score = initial_credit_score_gen(user_id).await.unwrap();
            
            if let Err(why) = message.channel_id.send_message(&ctx.http, |m|{

                m.reference_message(&message);

                m.embed(|mut e| {
                    e.title(format!("{}'s Credit Score", nickname));
                    e.description(format!("Your Credit Score is {}", credit_score));
                    e.thumbnail("https://cdn.discordapp.com/attachments/194593895647019008/843289961309929492/magik.png");
            
                    e
                });

                m
            }).await {
                println!("Error sending message: {:?}", why);
            }

        } else {
            let mut credit_history = get_credit_score(user_id).await.unwrap();
            let latest_score = credit_history.pop().unwrap(); //unwrap and then get the next pop for previnous score
            let mut prev_score:i16=0;
            if credit_history.len() >= 1 {
                prev_score = credit_history.pop().unwrap();
            }

            // if let Err(why) = message.reply_ping(&ctx.http, format!("your credit score is {}, from {}", latest_score, prev_score)).await  {
            //     println!("Error sending message: {:?}", why);
            // }

            if let Err(why) = message.channel_id.send_message(&ctx.http, |m|{

                m.reference_message(&message);

                m.embed(|mut e| {
                    e.title(format!("{}'s Credit Score", nickname));

                    if latest_score > prev_score {
                        e.description(format!("Your Credit Score is {}, from {} :chart_with_upwards_trend:", latest_score, prev_score));
                    } else if latest_score < prev_score {
                        e.description(format!("Your Credit Score is {}, from {} :chart_with_downwards_trend:", latest_score, prev_score));
                    } else {
                        e.description(format!("Your Credit Score is {}, from {}", latest_score, prev_score));
                    }

                    
                    e.thumbnail("https://cdn.discordapp.com/attachments/194593895647019008/843289961309929492/magik.png");
            
                    e
                });

                m
            }).await {
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

    println!("hewwo");
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
    let mut genned_score = fastrand::i16(0..1000);

    if *user_id % 2 != 0{
        genned_score = genned_score * 2;
    }

    let mut cmd_writer = csv::WriterBuilder::new()
        .from_writer(fs::OpenOptions::new().append(true).open("credit_scores.csv").unwrap());

        cmd_writer.write_record(&[user_id.to_string(),genned_score.to_string()])?;

    Ok(genned_score)
}

async fn get_credit_score(user_id: &u64) -> Result<Vec<i16>, csv::Error>{
    let mut reader = csv::Reader::from_path("credit_scores.csv").unwrap();
    let mut credit_history: Vec<i16> = Vec::new();

    for record in reader.records() {
        let record = record?;
        let record_id :u64= record[0].parse().unwrap();

        if *user_id == record_id {
            credit_history.push(record[1].parse().unwrap());
        }

    }
    credit_history.push(credit_check_hit(&credit_history.last().unwrap(), user_id).await.unwrap());
    //TODO update old credit score
    Ok(credit_history)
}

async fn credit_check_hit(credit_score: &i16, user_id: &u64) -> Result<i16, csv::Error>{
    let mut cmd_writer = csv::WriterBuilder::new()
        .from_writer(fs::OpenOptions::new().append(true).open("credit_scores.csv").unwrap());

    let mut credit_hit = fastrand::i16(0..20);
    let good_luck = fastrand::i8(1..11);
    if good_luck == 7 {
        credit_hit = -credit_hit*5;
    }
    let new_credit_score = *credit_score-credit_hit;

    cmd_writer.write_record(&[user_id.to_string(),new_credit_score.to_string()])?;

    Ok(new_credit_score)
}