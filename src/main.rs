use chrono::Utc;
use scraper::selector::CssLocalName;
use scraper::{CaseSensitivity, Element, Selector};
use serde::{Deserialize, Serialize};
use std::env;
use std::env::args;
use tokio::fs;

/*

 */
#[derive(Serialize, Deserialize)]
struct Claims {
    pub iss: String,
    pub exp: i64,
    pub iat: i64,
}

#[derive(Debug)]
struct AppleApi {
    pub team_id: String,
    pub key_id: String,
    pub private_key_path: String,
    pub api_url: String,
}

#[tokio::main]
async fn main() {
    let apple_api = init_apple_api();
    let target_url = args().nth(1).expect(
        "Please provide a target URL as an argument", );
    println!("Target URL: {}", target_url);
    let client = reqwest::Client::new();
    let response = client.get(target_url).send().await.expect("Failed to get response");
    println!("Response: {:?}", response);
    let response_text = response.text().await.expect("Failed to get response text");
    //println!("Response text: {}", response_text);

    let all_song_titles = scrape_songs_from_setlist(&response_text);

    all_song_titles.iter().for_each(|song_name| println!("Song name: {}", song_name));
    println!("{} songs found", all_song_titles.len());

    println!("Creating JWT");
    let jwt = create_jwt(apple_api).await;
    println!("JWT: {}", jwt);
    println!("Done!")
}

fn scrape_songs_from_setlist(response_text: &String) -> Vec<String> {
    let selector = Selector::parse("div").unwrap();
    let setlist_class = CssLocalName::from("setlistList");
    let song_label_class = CssLocalName::from("songLabel");

    let document = scraper::Html::parse_document(&response_text);

    let set_list_node = document
        .select(&selector)
        .filter(
            |node| node.has_class(&setlist_class, CaseSensitivity::CaseSensitive)
        )
        .next()
        .expect("Failed to find set list node");
    println!("Set list node: {:#?}", set_list_node.clone());
    // select all descendants given the path: //*[@id="id29"]/ol/li[2]/div[1]/a
    let all_song_selector = Selector::parse("li").unwrap();
    let a_selector = Selector::parse("a").unwrap();
    let all_li_elements = set_list_node.select(&all_song_selector).collect::<Vec<_>>();
    let all_a_song_elements = all_li_elements.iter().map(|elem| elem.select(&a_selector)).collect::<Vec<_>>();

    let all_song_titles = all_a_song_elements
        .iter()
        .flat_map(move |selector| {
            let song_label_class_cloned = song_label_class.clone();
            selector.clone()
                .filter(move |node| node.has_class(&song_label_class_cloned, CaseSensitivity::CaseSensitive))
                .map(move |elem| elem.text().collect::<String>())
        }
        ).collect::<Vec<_>>();
    all_song_titles
}

// init AppleApi struct from .env file
fn init_apple_api() -> AppleApi {
    dotenv::dotenv().ok();
    let team_id = env::var("APPLE_TEAM_ID").expect("APPLE_TEAM_ID not set");
    let key_id = env::var("APPLE_KEY_ID").expect("APPLE_KEY_ID not set");
    let private_key_path = env::var("APPLE_PRIVATE_KEY_PATH").expect("APPLE_PRIVATE_KEY_PATH not set");
    let api_url = env::var("APPLE_API_URL").expect("APPLE_API_URL not set");
    AppleApi {
        team_id,
        key_id,
        private_key_path,
        api_url,
    }
}

// create and sign JWT with ES256 and contents of p8 file, using jsonwebtoken
async fn create_jwt(apple_api: AppleApi) -> String {
    let private_key = fs::read_to_string(apple_api.private_key_path).await.expect("Failed to read private key");
    let private_key = jsonwebtoken::EncodingKey::from_ec_pem(private_key.as_bytes()).expect("Failed to parse private key");
    let mut header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::ES256);
    header.kid = Some(apple_api.key_id);
    let now = Utc::now().timestamp_millis();
    let expires_at = now + 60 * 60 * 1000;
    let payload = Claims {
        iss: apple_api.team_id.to_string(),
        exp: expires_at,
        iat: now,
    };
    let token = jsonwebtoken::encode(&header, &payload, &private_key).expect("Failed to create token");
    token
}

