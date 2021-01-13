#![allow(warnings)]
extern crate csv;
extern crate hyper;
use rand::Rng;
use std::error::Error as AE;
use actix_cors::Cors;
use actix_web::{
    http::ContentEncoding, middleware, web, web::Bytes, App, HttpResponse, HttpServer,
};
use failure::Error;
use hyper::{Client, Request};
use hyper_tls::HttpsConnector;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::io;
use std::process;
use futures::executor::block_on;
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Record {
    Username: String,
    SubscribeDate: String,
    CurrentTier: String,
    Tenure: u16,
    Streak: u16,
    SubType: String,
    Founder: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct lotteryData {
    Username: String,
    SubscribeDate: String,
    CurrentTier: String,
    Tenure: u16,
    Streak: u16,
    SubType: String,
    Founder: bool,
    rand: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct GiftData {
    user_id: String,
    user_name: String,
    tier: String,
    is_gift: String,
    gifter_id: String,
    gifter_name: String,
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    lottery().await;
    HttpServer::new(|| {
        App::new()
            .wrap(
                Cors::new() // <- Construct CORS middleware builder
                    .allowed_methods(vec!["GET", "POST"])
                    .finish(),
            )
            .wrap(middleware::Compress::new(ContentEncoding::Br))
            .route("/get_winner/", web::post().to(winner))
    })
    .bind("0.0.0.0:5566")?
    .run()
    .await
}

async fn winner(_bytes: Bytes) -> Result<HttpResponse, Error> {
    #[derive(Serialize, Deserialize)]
    struct Body {
        pub count: i32,
    }
    let req = String::from_utf8_lossy(&_bytes);
    let body: Body = serde_json::from_str(&req)?;
    let mut winner: Vec<String> = Vec::new();
    // let winner: Vec<String> = get_winner(body.count);
    let w = get_winner(body.count);
    match w {
        Ok(v) => {
            winner = v;
        }
        Err(e) => {
            println!("Err {}", e);
        }
    }
    let res = serde_json::to_string(&winner).unwrap();
    Ok(HttpResponse::Ok().body(res))
}

fn get_winner(i: i32) -> Result<(Vec<String>), Box<AE>> {
    let mut count = i;
    let mut rdr = csv::Reader::from_path("lottery_pool.csv")?;
    let mut winner: Vec<String> = Vec::new();
    let mut lottery_pool: Vec<lotteryData> = Vec::new();
    let mut rng = rand::thread_rng();
    for result in rdr.deserialize() {
        let record: Record = result?;
        let data = lotteryData{
            Username: record.Username,
            SubscribeDate: record.SubscribeDate,
            CurrentTier: record.CurrentTier,
            Tenure: record.Tenure,
            Streak: record.Streak,
            SubType: record.SubType,
            Founder: record.Founder,
            rand: rng.gen::<f32>(),
        };
        lottery_pool.push(data.clone());
    }
    lottery_pool.sort_by(|a, b| b.rand.partial_cmp(&a.rand).unwrap());
    let mut done = false;
    for user in lottery_pool {
        let mut isChoosed = false;
        for w in &winner {
            if w == &user.Username {
                isChoosed = true;
            }
        }
        if !isChoosed {
            winner.push(user.Username);
            count -= 1;
        }
        if count == 0 {
            break;
        }
    }
    Ok(winner)
}

async fn get_gifter(user_id: String) -> Result<Value, Error> {
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);
    let broadcaster_id = "11561802";
    let client_id = "fsbi12mzorvkvfp9vtui6hkx4vj7hn";
    let authorization = "Bearer drgvgtw0csccrh6zs0vyqag4tx1wqx";
    let steam_uri = format!(
        "https://api.twitch.tv/helix/subscriptions?broadcaster_id={}&user_id={}",
        broadcaster_id, user_id
    );
    println!("{}", steam_uri);
    let request = Request::builder()
    .method("GET")
    .uri(steam_uri)
    .header("Client-ID", client_id)
    .header("Authorization", authorization)
    .body(hyper::Body::empty())
    .unwrap();
    let res = client.request(request).await?;
    let body = hyper::body::to_bytes(res).await?;
    let str_body = String::from_utf8_lossy(&body);
    let v: Value = serde_json::from_str(&str_body)?;
    let response = v.to_string();
    Ok(v)
}

async fn get_twitch_id(login: String) -> Result<Value, Error> {
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);
    let client_id = "fsbi12mzorvkvfp9vtui6hkx4vj7hn";
    let authorization = "Bearer drgvgtw0csccrh6zs0vyqag4tx1wqx";
    let steam_uri = format!(
        "https://api.twitch.tv/helix/users?login={}",
        login
    );
    println!("steam_uri: {}", steam_uri);
    let request = Request::builder()
        .method("GET")
        .uri(steam_uri)
        .header("Client-ID", client_id)
        .header("Authorization", authorization)
        .body(hyper::Body::empty())
        .unwrap();
    let res = client.request(request).await?;
    let body = hyper::body::to_bytes(res).await?;
    let str_body = String::from_utf8_lossy(&body);
    let v: Value = serde_json::from_str(&str_body)?;
    let response = v.to_string();
    Ok(v)
}

async fn get_twitch_user(id: String) -> Result<Value, Error> {
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);
    let client_id = "fsbi12mzorvkvfp9vtui6hkx4vj7hn";
    let authorization = "Bearer drgvgtw0csccrh6zs0vyqag4tx1wqx";
    let steam_uri = format!(
        "https://api.twitch.tv/helix/users?id={}",
        id
    );
    let request = Request::builder()
        .method("GET")
        .uri(steam_uri)
        .header("Client-ID", client_id)
        .header("Authorization", authorization)
        .body(hyper::Body::empty())
        .unwrap();
    let res = client.request(request).await?;
    let body = hyper::body::to_bytes(res).await?;
    let str_body = String::from_utf8_lossy(&body);
    let v: Value = serde_json::from_str(&str_body)?;
    let response = v.to_string();
    Ok(v)
}

async fn lottery() -> Result<(), Box<dyn AE>> {
    let mut rdr = csv::Reader::from_path("gift_list.csv")?;
    let mut gifters: BTreeMap<String, String> = BTreeMap::new();
    for result in rdr.deserialize() {
        println!("result : {:?}", result);
        let giftData: GiftData = result?;
        gifters.insert(giftData.user_id, giftData.gifter_id); 
    }
    rdr = csv::Reader::from_path("lottery_list.csv")?;
    let mut lottery_pool: Vec<Record> = Vec::new();
    for result in rdr.deserialize() {
        let record: Record = result?;
        if record.SubType == "gift" {
            let mut v = get_twitch_id(record.Username.clone()).await?;
            let mut id_len = v["data"][0]["id"].to_string().len();
            let userID: String = v["data"][0]["id"].to_string()[1..id_len-1].to_string();
            if let Some(gifterID) = gifters.get(&userID) {
                v = get_twitch_user(gifterID.to_string()).await?;
                let gifterName = v["data"][0]["login"].to_string();
                println!("{}", gifterName);
                let mut newRecord = record.clone();
                newRecord.Username = gifterName;
                for _ in 0..record.Tenure/3 {
                    lottery_pool.push(newRecord.clone());
                }
                lottery_pool.push(newRecord.clone());
            } else {
                for _ in 0..record.Tenure/3 {
                    lottery_pool.push(record.clone());
                }
                lottery_pool.push(record.clone());
            }
        }else{
            for _ in 0..record.Tenure/3 {
                lottery_pool.push(record.clone());
            }
            lottery_pool.push(record.clone());
        }
    }
    let mut wtr = csv::Writer::from_path("lottery_pool.csv")?;
    for u in lottery_pool {
        wtr.serialize(u)?;
    }
    Ok(())
}