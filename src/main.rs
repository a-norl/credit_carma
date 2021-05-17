use std::{convert::TryInto, fs, mem};
use std::io::Read;

use serenity::{async_trait, model::{channel::Message, gateway::Ready}, prelude::*};

use csv;
use fastrand;

struct Handler;

#[async_trait]
impl EventHandler for Handler{

    async fn message(&self, ctx: Context, message: Message) {
        let user_id = message.author.id.as_u64();

        if check_csv(user_id).await.unwrap() && !(message.content.to_ascii_lowercase().contains("credit score")) {
            credit_increase_with_msg(&user_id, message.content.len()).await.unwrap();
            let surprise_credit_score = get_credit_hist_no_hit(user_id).await.unwrap().pop().unwrap();

            //give The Role
            let mut member = message.member(&ctx.http).await.unwrap();
            let has_role = message.author.has_role(&ctx.http, 194593895647019008, 843927305260761228).await.unwrap();
            if surprise_credit_score >= 1000 && !(has_role) {
                member.add_role(&ctx.http, 843927305260761228).await.unwrap();
            } else if has_role && surprise_credit_score < 1000 {
                member.remove_role(&ctx.http, 843927305260761228).await.unwrap();
            }

            let surprise_check = fastrand::i8(1..101);
            if surprise_check <= 5 && message.channel_id.0 != 843258808389074965 {
                

                if let Err(why) = message.channel_id.send_message(&ctx.http, |m|{

                    m.reference_message(&message);
    
                    m.embed(|e| {
                        e.title("Surprise Credit Check!");
                        e.description(format!("This user's credit score is {}", surprise_credit_score));
                        e.thumbnail("https://cdn.discordapp.com/attachments/194593895647019008/843289961309929492/magik.png");
                
                        e
                    });
    
                    m
                }).await {
                    println!("Error sending message: {:?}", why);
                }
            }
        }

        if message.channel_id.0 == 843258808389074965 && message.content.to_ascii_lowercase().contains("loan"){

        }

        if !(message.content.to_ascii_lowercase().contains("credit score")) || message.author.bot || message.channel_id.0 != 843258808389074965 {return;}


        
        let mut nickname = message.author.name.clone();
        if message.author_nick(&ctx.http).await.is_some(){
            nickname = message.author_nick(&ctx.http).await.unwrap();
        }


        if !(check_csv(user_id).await.unwrap()) {
            let credit_score = initial_credit_score_gen(user_id).await.unwrap();
            
            if let Err(why) = message.channel_id.send_message(&ctx.http, |m|{

                m.reference_message(&message);

                m.embed(|e| {
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
            let mut credit_history = get_credit_hist_with_hit(user_id).await.unwrap();
            let latest_score = credit_history.pop().unwrap();
            let mut prev_score:i16=0;
            if credit_history.len() >= 1 {
                prev_score = credit_history.pop().unwrap();
            }

            if let Err(why) = message.channel_id.send_message(&ctx.http, |m|{

                m.reference_message(&message);

                m.embed(|e| {
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

    let bad_luck = fastrand::i8(1..20);

    if *user_id % 2 != 0{
        genned_score = genned_score * 2;
    }

    if bad_luck == 10 {
        genned_score = -genned_score;
    }

    let mut cmd_writer = csv::WriterBuilder::new()
        .from_writer(fs::OpenOptions::new().append(true).open("credit_scores.csv").unwrap());

        cmd_writer.write_record(&[user_id.to_string(),genned_score.to_string()])?;

    Ok(genned_score)
}

async fn get_credit_hist_with_hit(user_id: &u64) -> Result<Vec<i16>, csv::Error>{
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

    Ok(credit_history)
}

async fn get_credit_hist_no_hit(user_id: &u64) -> Result<Vec<i16>, csv::Error>{
    let mut reader = csv::Reader::from_path("credit_scores.csv").unwrap();
    let mut credit_history: Vec<i16> = Vec::new();

    for record in reader.records() {
        let record = record?;
        let record_id :u64= record[0].parse().unwrap();

        if *user_id == record_id {
            credit_history.push(record[1].parse().unwrap());
        }

    }

    Ok(credit_history)
}

async fn credit_check_hit(credit_score: &i16, user_id: &u64) -> Result<i16, csv::Error>{
    let mut cmd_writer = csv::WriterBuilder::new()
        .from_writer(fs::OpenOptions::new().append(true).open("credit_scores.csv").unwrap());

    let mut credit_hit = fastrand::i16(0..20);
    let good_luck = fastrand::i8(5..11);
    if good_luck == 7 {
        credit_hit = -credit_hit*5;
    }
    let new_credit_score = *credit_score-credit_hit;

    cmd_writer.write_record(&[user_id.to_string(),new_credit_score.to_string()])?;

    Ok(new_credit_score)
}

async fn credit_increase_with_msg(user_id: &u64, message_length: usize) -> Result<(), csv::Error>{
    let mut cmd_writer = csv::WriterBuilder::new()
        .from_writer(fs::OpenOptions::new().append(true).open("credit_scores.csv").unwrap());

    let mut credit_hist = get_credit_hist_no_hit(user_id).await.unwrap();
    let mut credit_score = credit_hist.pop().unwrap();

    //NewValue = (((OldValue - OldMin) * (NewMax - NewMin)) / (OldMax - OldMin)) + NewMin
    let point_add_unrounded: f32 = ((message_length * 50)/(2000)) as f32;
    let point_add: i16 = point_add_unrounded.ceil() as i16;

    credit_score = credit_score + point_add;


    cmd_writer.write_record(&[user_id.to_string(),credit_score.to_string()])?;
    Ok(())
}