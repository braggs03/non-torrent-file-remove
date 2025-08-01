use std::collections::HashSet;
use std::fs;
use std::{collections::HashMap, sync::Arc};

use clap::Parser;
use reqwest::Client;
use reqwest::cookie::Jar;
use walkdir::WalkDir;

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

    /// Whether files should be deleted.
    #[arg(long, default_value_t = false)]
    destructive: bool,

    #[arg(long, short, default_value_t = false)]
    debug: bool,

}

#[derive(serde::Deserialize, Debug)]
struct TorrentSavePath {
    save_path: String,
}

#[derive(serde::Deserialize, Debug)]
struct TorrentInfo {
    hash: String,
}

#[derive(serde::Deserialize, Debug)]
struct TorrentFile {
    name: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let api_url = format!("http://{}:{}/api/v2", args.ip, args.port);

    let client = get_login_client(&args, &api_url).await?;

    let save_path: String = client
        .get(format!("{}/app/preferences", api_url))
        .send()
        .await?
        .json()
        .await
        .and_then(|save_path: TorrentSavePath| Ok(save_path.save_path))?;

    let torrent_hashes: Vec<TorrentInfo> = client
        .get(format!("{}/torrents/info", api_url))
        .send()
        .await?
        .json()
        .await?;


    let all_torrent_files = get_torrent_files(&client, &api_url, &save_path, torrent_hashes).await?;

    let _ = remove_torrent_files_and_directories(&args, all_torrent_files, &save_path).await?;

    Ok(())
}

async fn get_login_client(args: &Args, api_url: &str) -> Result<Client, Box<dyn std::error::Error>> {
    let mut req_credentials = HashMap::new();
    req_credentials.insert("username", &args.username);
    req_credentials.insert("password", &args.password);

    let cookie_store = Arc::new(Jar::default());
    let client = Client::builder()
        .cookie_provider(cookie_store.clone())
        .build()?;

    let _ = client.post(format!("{}/auth/login", api_url))
        .form(&req_credentials)
        .send()
        .await?;

    Ok(client)
}

async fn get_torrent_files(
    client: &Client, 
    api_url: &str, 
    save_path: &str, 
    all_torrent_info:Vec<TorrentInfo>, 
) -> Result<HashSet<String>, Box<dyn std::error::Error>> {

    let mut all_torrent_files = HashSet::new();

    for torrent in all_torrent_info.iter() {
        let url = format!("{}/torrents/files?hash={}", api_url, torrent.hash);
    
        let files: Vec<TorrentFile> = client.get(url)
            .send()
            .await?
            .json()
            .await?;
            
        files.iter().for_each(|file| {
            all_torrent_files.insert(format!("{}/{}", save_path, file.name).replace("\\", "/"));
        });
    }

    Ok(all_torrent_files)
}

async fn remove_torrent_files_and_directories(args: &Args, all_torrent_files: HashSet<String>, save_path: &str) -> Result<(), Box<dyn std::error::Error>> {

    let mut entries = Vec::new();

    WalkDir::new(format!("{}", save_path)).into_iter().for_each(
        |entry| {
            entries.push(entry.unwrap());
        }
    );

    entries.sort_by(|a,b| {
        match a.depth().cmp(&b.depth()) {
            std::cmp::Ordering::Less => std::cmp::Ordering::Greater,
            std::cmp::Ordering::Greater => std::cmp::Ordering::Less,
            std::cmp::Ordering::Equal => {
                if a.file_type().is_dir() {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Greater 
                }
            },
        }
    });

    for entry in entries {
        let file_path = entry.path().to_str().expect("Error: Walk error.");
        let file_path = file_path.replace("\\", "/");
        let file_type = entry.file_type();

        if file_type.is_file() && !all_torrent_files.contains(&file_path) {
            if args.debug {
                println!("Found dangling file: {}", &file_path);
            }
            if args.destructive {
                match fs::remove_file(&file_path) {
                    Ok(_) => {},
                    Err(err) => {
                        if args.debug {
                            println!("Removing File Error: {}, {}", err, &file_path)
                        }
                    },
                }
            }
        } else if file_type.is_dir() && fs::read_dir(&file_path).unwrap().next().is_none() {
            if args.debug {
                println!("Found dangling folder: {}", &file_path);
            }
            if args.destructive {
                match fs::remove_dir(&file_path) {
                    Err(err) => {
                        if args.debug {
                            println!("Removing Empty Directory Error: {}, {}", err, &file_path)
                        }
                    },
                    _ => {},
                };
            }
        }
    }  



    Ok(())
}
