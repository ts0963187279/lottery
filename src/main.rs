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
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

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

#[derive(Serialize, Deserialize, Clone, Debug)]
struct CheckAuthData {
    access_token: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct GetGifterData {
    authorization: String,
    broadcaster_id: String,
    ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct GetUsersData {
    authorization: String,
    ids: Vec<String>,
}

const CLIENT_ID: &str = "dp74x753ef0ri5gopkz53a6l44a30k";

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("server.key", SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file("hibiki-rain_com.crt").unwrap();
    HttpServer::new(|| {
        App::new()
            .wrap(
                Cors::new() // <- Construct CORS middleware builder
                    .allowed_methods(vec!["GET", "POST"])
                    .finish(),
            )
            .wrap(middleware::Compress::new(ContentEncoding::Br))
            .route("/get_winner/", web::post().to(winner))
            .route("/check_auth/", web::post().to(check_auth))
            .route("/get_gifter/", web::post().to(get_gifter))
            .route("/get_users/", web::post().to(get_users))
    })
    .bind_openssl("0.0.0.0:5566", builder)?
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

async fn check_auth(_bytes: Bytes) -> Result<HttpResponse, Error> {
    let req = String::from_utf8_lossy(&_bytes);
    println!("check_auth req: {}", req);
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);
    let data: CheckAuthData = serde_json::from_str(&req)?;
    let twitch_uri = format!(
        "https://id.twitch.tv/oauth2/validate"
    );
    let request = Request::builder()
    .method("GET")
    .uri(twitch_uri)
    .header("Authorization", format!("OAuth {}", data.access_token))
    .body(hyper::Body::empty())
    .unwrap();
    let res = client.request(request).await?;
    let body = hyper::body::to_bytes(res).await?;
    let str_body = String::from_utf8_lossy(&body);
    let v: Value = serde_json::from_str(&str_body)?;
    let response = v.to_string();
    Ok(HttpResponse::Ok().body(response))
}

async fn get_gifter(_bytes: Bytes) -> Result<HttpResponse, Error> {
    println!("get gifter req");
    let req = String::from_utf8_lossy(&_bytes);
    let data: GetGifterData = serde_json::from_str(&req)?;
    let mut ids: String = format!("");
    for id in data.ids {
        ids = format!("{}&user_id={}",ids ,id);
    }
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);
    let steam_uri = format!(
        "https://api.twitch.tv/helix/subscriptions?broadcaster_id={}{}",
        data.broadcaster_id, ids
    );
    let request = Request::builder()
    .method("GET")
    .uri(steam_uri)
    .header("Client-ID", CLIENT_ID)
    .header("Authorization", format!("Bearer {}", data.authorization))
    .body(hyper::Body::empty())
    .unwrap();
    let res = client.request(request).await?;
    let body = hyper::body::to_bytes(res).await?;
    let str_body = String::from_utf8_lossy(&body);
    let v: Value = serde_json::from_str(&str_body)?;
    let response = v.to_string();
    Ok(HttpResponse::Ok().body(response))
}

async fn get_users(_bytes: Bytes) -> Result<HttpResponse, Error> {
    println!("get users req");
    let req = String::from_utf8_lossy(&_bytes);
    let data: GetUsersData = serde_json::from_str(&req)?;
    let mut ids: String = format!("");
    let mut count = 0;
    for id in data.ids {
        count += 1;
        ids = format!("{}&login={}",ids ,id);
        println!("{}, {}", count, id);
        if count == 100 {
            break;
        }
    }
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);
    let uri = format!(
        "https://api.twitch.tv/helix/users?{}",
        ids
    );
    let request = Request::builder()
        .method("GET")
        .uri(uri)
        .header("Client-ID", CLIENT_ID)
        .header("Authorization",format!("Bearer {}", data.authorization))
        .body(hyper::Body::empty())
        .unwrap();
    let res = client.request(request).await?;
    let body = hyper::body::to_bytes(res).await?;
    let str_body = String::from_utf8_lossy(&body);
    let v: Value = serde_json::from_str(&str_body)?;
    let response = v.to_string();
    Ok(HttpResponse::Ok().body(response))
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

// async fn get_gifter(user_id: String) -> Result<Value, Error> {
//     let https = HttpsConnector::new();
//     let client = Client::builder().build::<_, hyper::Body>(https);
//     let broadcaster_id = "11561802";
//     let client_id = "fsbi12mzorvkvfp9vtui6hkx4vj7hn";
//     let authorization = "Bearer drgvgtw0csccrh6zs0vyqag4tx1wqx";
//     let steam_uri = format!(
//         "https://api.twitch.tv/helix/subscriptions?broadcaster_id={}&user_id={}",
//         broadcaster_id, user_id
//     );
//     println!("{}", steam_uri);
//     let request = Request::builder()
//     .method("GET")
//     .uri(steam_uri)
//     .header("Client-ID", client_id)
//     .header("Authorization", authorization)
//     .body(hyper::Body::empty())
//     .unwrap();
//     let res = client.request(request).await?;
//     let body = hyper::body::to_bytes(res).await?;
//     let str_body = String::from_utf8_lossy(&body);
//     let v: Value = serde_json::from_str(&str_body)?;
//     let response = v.to_string();
//     Ok(v)
// }

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
    println!("start parse");
    let mut rdr = csv::Reader::from_path("lottery_list.csv")?;
    let mut lottery_pool: Vec<Record> = Vec::new();
    for result in rdr.deserialize() {
        let record: Record = result?;
        println!("record : {:?}", record);
        // if record.SubType == "gift" {
        //     let mut v = get_twitch_id(record.Username.clone()).await?;
        //     let mut id_len = v["data"][0]["id"].to_string().len();
        //     let userID: String = v["data"][0]["id"].to_string()[1..id_len-1].to_string();
        //     if let Some(gifterID) = gifters.get(&userID) {
        //         v = get_twitch_user(gifterID.to_string()).await?;
        //         let gifterName = v["data"][0]["login"].to_string();
        //         println!("{}", gifterName);
        //         let mut newRecord = record.clone();
        //         newRecord.Username = gifterName;
        //         for _ in 0..record.Tenure/3 {
        //             lottery_pool.push(newRecord.clone());
        //         }
        //         lottery_pool.push(newRecord.clone());
        //     } else {
        //         for _ in 0..record.Tenure/3 {
        //             lottery_pool.push(record.clone());
        //         }
        //         lottery_pool.push(record.clone());
        //     }
        // }else{
            for _ in 0..record.Tenure/3 {
                lottery_pool.push(record.clone());
            }
            lottery_pool.push(record.clone());
            println!("record : {:?}", record.clone());
        // }
    }
    let mut wtr = csv::Writer::from_path("lottery_pool.csv")?;
    for u in lottery_pool {
        wtr.serialize(u)?;
    }
    println!("done!");
    Ok(())
}