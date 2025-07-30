use std::{collections::HashMap, sync::Arc};

use clap::Parser;
use reqwest::{Client, Url};
use reqwest::cookie::{CookieStore, Jar};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Qbittorent WebUI IP.
    #[arg(long)]
    ip: String,

    /// Qbittorent WebUI Port.
    #[arg(long)]
    port: String,

    /// Qbittorent WebUI Username.
    #[arg(long)]
    username: String,

    /// Qbittorent WebUI Password.
    #[arg(long)]
    password: String,
}


#[tokio::main]
async fn main() {
    let args = Args::parse();

    let login = get_login(args).await;
}

async fn get_login(args: Args) -> Result<Args, Box<dyn std::error::Error>> {
    let mut req_credentials = HashMap::new();
    req_credentials.insert("username", "admin");
    req_credentials.insert("password", "OrsoNero8641!");

    let cookie_store = Arc::new(Jar::default());
    let client = Client::builder()
        .cookie_provider(cookie_store.clone())
        .build()?;

    let _ = client.post("http://localhost:8080/api/v2/auth/login")
        .form(&req_credentials)
        .send()
        .await?;
    
    println!("{:?}", cookie_store.cookies(&Url::parse("http://localhost:8080/api/v2/auth/login").unwrap()));


    // let credentials: Credentials = serde_json::from_str(&json).expect("");
    
    // println!("{:?}", credentials);

    Ok(args)
}

// #[derive(serde::Deserialize, Debug)]
// struct Credentials {
    
//     set-cookie: String,

// }
