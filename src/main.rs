use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::env;
use std::env::temp_dir;
use tokio::fs;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let name = "STEAM_API_KEY";

    let api_key = match env::var(name) {
        Ok(v) => v,
        Err(_) => panic!("${} env var is not set", name),
    };

    let mut args: Vec<String> = env::args().collect();
    args.remove(0);

    if args.is_empty() {
        panic!("At least 1 user ID needed")
    }

    let mut app_ids = HashSet::new();
    for user_id in args {
        let actual_ids = get_game_ids(user_id, api_key.clone()).await?;
        if app_ids.is_empty() {
            app_ids = actual_ids;
        } else {
            app_ids = &app_ids & &actual_ids
        }
    }

    let names = app_ids_to_names(app_ids).await?;

    println!("Common games: {:#?}", names);
    Ok(())
}

async fn app_ids_to_names(
    app_ids: HashSet<i64>,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut mapping_file = temp_dir();
    mapping_file.push("appid_to_names.json");
    println!("tmp mapping file: {}", mapping_file.to_str().unwrap());

    if !mapping_file.is_file() {
        println!("NOT Found appid to name file. Downloading it");
        let contents = reqwest::get("https://api.steampowered.com/ISteamApps/GetAppList/v2/")
            .await?
            .text()
            .await?;
        let mut created = File::create(mapping_file.clone().as_path()).await?;
        created.write_all(contents.as_bytes()).await?;
        created.sync_all().await?;
    } else {
        println!("Found appid to name mapping file.")
    }
    let contents = fs::read(mapping_file).await?;
    let value: Value = serde_json::from_slice(contents.as_slice()).unwrap();
    let vals = value
        .get("applist")
        .unwrap()
        .get("apps")
        .unwrap()
        .as_array()
        .unwrap();
    let mut map = HashMap::new();
    for v in vals {
        map.insert(
            v.get("appid").unwrap().as_i64().unwrap(),
            String::from(v.get("name").unwrap().as_str().unwrap()),
        );
    }

    let mut result = Vec::new();
    for app_id in app_ids {
        let name = map.get(&app_id);
        if name.is_none() {
            println!("Cannot find name for id: {}", app_id)
        } else {
            result.push(name.unwrap().clone());
        }
    }

    return Ok(result);
}

async fn get_game_ids(
    user_id: String,
    api_key: String,
) -> Result<HashSet<i64>, Box<dyn std::error::Error>> {
    let url = format!("https://api.steampowered.com/IPlayerService/GetOwnedGames/v0001/?key={api_key}&steamid={user_id}&format=json");

    let resp = reqwest::get(url).await?.json::<serde_json::Value>().await?;

    let mut result = HashSet::new();
    let games = resp
        .get("response")
        .unwrap()
        .get("games")
        .unwrap()
        .as_array()
        .unwrap();
    for g in games {
        let num = g.get("appid").unwrap().as_number().unwrap();
        let int = num.as_i64().unwrap();
        result.insert(int);
    }
    Ok(result)
}
