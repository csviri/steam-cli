use std::collections::HashSet;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let name = "STEAM_API_KEY";

    let api_key = match env::var(name) {
        Ok(v) => v,
        Err(_) => panic!("${} is not set", name),
    };

    let mut args: Vec<String> = env::args().collect();
    args.remove(0);

    // if args.len() < 2 {
    //     panic!("At least 2 user IDs needed")
    // }

    let mut app_ids = HashSet::new();

    for user_id in args {
        let actual_ids = get_game_ids(user_id, api_key.clone()).await?;
        if app_ids.is_empty() {
            app_ids = actual_ids;
        } else {
            app_ids = &app_ids & &actual_ids
        }
    }

    println!("{:#?}", app_ids);
    Ok(())
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
