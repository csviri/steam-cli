use std::collections::HashMap;
use std::env;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let name = "STEAM_API_KEY";
    let api_key;
    match env::var(name) {
        Ok(v) => api_key = v,
        Err(e) => panic!("${} is not set", name)
    }

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        panic!("At least 2 user IDs needed")
    }
    let first = args.get(0).unwrap();

    let url = format!("https://api.steampowered.com/IPlayerService/GetOwnedGames/v0001/?key={api_key}&steamid={first}&format=json");

    println!("url: {}", url);

    let resp = reqwest::get(url)
        .await?
        .json::<serde_json::Value>()
        .await?;

    println!("{:#?}", resp);
    Ok(())
}

