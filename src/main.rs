use std::process::exit;

use argh::FromArgs;
use serde::Deserialize;

const HLTB_URL: &str = "https://howlongtobeat.com/api/search";

#[derive(Debug, FromArgs)]
/// hltb only really only needs the game argument
struct Args {
    #[argh(positional)]
    game: Vec<String>,

    #[argh(option, short = 'n')]
    /// the maximum number of games to display (Default 5)
    number: Option<usize>,
}

fn main() {
    let args: Args = argh::from_env();
    if args.game.is_empty() {
        println!("Usage: hltb <game name>\nExample: hltb ori and the");
        exit(1)
    }

    let doc = search(&args);

    match doc.data.len() {
        0 => {
            println!("Could not find any games");
            exit(1);
        }
        amount => {
            let s = if amount == 1 { "" } else { "s" };
            println!("Found {} matching game{}", amount, s);
        }
    }

    let mut data = doc.data;
    data.sort_by(
        |lhs, rhs|
        rhs.comp_all_count.cmp(&lhs.comp_all_count)
    );
    data
        .iter()
        .take(args.number.unwrap_or(5))
        .for_each(print_game);
}

#[derive(Deserialize)]
struct SearchResponse {
    data: Vec<SearchData>,
}

#[derive(Deserialize)]
struct SearchData {
    game_name: String,
    comp_main: usize,
    comp_plus: usize,
    comp_100: usize,
    comp_all_count: usize,
    release_world: usize,
}

fn search(args: &Args) -> SearchResponse {
    let request = ureq::json!({
        "searchPage": 1,
        "searchType": "games",
        "searchTerms": &args.game,
        "size": 20
    });

    ureq::post(HLTB_URL)
        .set("Referer", "https://howlongtobeat.com/")
        .send_json(ureq::json!(request))
        .expect("Request to howlongtobeat.com failed")
        .into_json::<SearchResponse>()
        .expect("Could not read response")
}

fn print_game(game: &SearchData) {
    println!("\n\x1B[1m{} ({})\x1B[0m", game.game_name, game.release_world);
    [
        ("Main Story", game.comp_main),
        ("Main + Extra", game.comp_plus),
        ("Completionist", game.comp_100),
    ]
    .into_iter()
    .for_each(|(n, t)| println!("{n}: {}", seconds_to_string(t)));
}

fn seconds_to_string(time: usize) -> String {
    if time == 0 {
        return "--".to_owned()
    }
    let mut hours = time / 3600;
    let minutes = (time / 60) % 60;
    let fraction = if hours < 100 && minutes > 15 && minutes < 45 {
        "½"
    } else {
        if minutes >= 45 {
            hours += 1;
        }
        ""
    };
    format!("{}{} Hours", hours, fraction)
}
