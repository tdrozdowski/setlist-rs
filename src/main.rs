use scraper::selector::CssLocalName;
use scraper::{CaseSensitivity, Element, Selector};
use std::env::args;

#[tokio::main]
async fn main() {
    let target_url = args().nth(1).expect(
        "Please provide a target URL as an argument", );
    println!("Target URL: {}", target_url);
    let client = reqwest::Client::new();
    let response = client.get(target_url).send().await.expect("Failed to get response");
    println!("Response: {:?}", response);
    let response_text = response.text().await.expect("Failed to get response text");
    //println!("Response text: {}", response_text);
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

    all_a_song_elements
        .iter()
        .flat_map(move |selector| {
            let song_label_class_cloned = song_label_class.clone();
            selector.clone()
                .filter(move |node| node.has_class(&song_label_class_cloned, CaseSensitivity::CaseSensitive))
                .map(move |elem| elem.text().collect::<String>())
        }
        )
        .for_each(|song_name| println!("Song name: {}", song_name));
}
