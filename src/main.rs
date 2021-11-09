use std::process::exit;

use argh::FromArgs;
use itertools::Itertools;
use scraper::{ElementRef, Selector};

const HLTB_URL: &str = "https://howlongtobeat.com/search_results";

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
    let game = args.game.join(" ");

    let doc = search(&game);

    match parse_result_amount(&doc) {
        Some(amount) => {
            let s = if amount == "1" { "" } else { "s" };
            println!("Found {} matching game{}\n", amount, s);
        }
        None => {
            println!("Could not find any games for query '{}'", game);
            exit(1);
        }
    }

    let games = parse_games(&doc, args.number);
    games.iter().for_each(|g| print_game(&g));
}

#[derive(Debug)]
struct Game {
    name: String,
    entries: Vec<(String, String)>,
}

fn search(game: &str) -> scraper::Html {
    let form_data = &[
        ("queryString", game),
        ("t", "games"),
        ("sorthead", "popular"),
    ];

    let res = ureq::post(HLTB_URL)
        .query("page", "1")
        .send_form(form_data)
        .expect("Request to howlongtobeat.com failed")
        .into_string()
        .expect("Could not read response");

    scraper::Html::parse_document(&res)
}

fn parse_result_amount(doc: &scraper::Html) -> Option<String> {
    let selector_games_number = Selector::parse("h3").unwrap();
    return match doc.select(&selector_games_number).next() {
        Some(games_number) => {
            // Get the number of games from the sentence `We Found 25 Games for \"gris\"`
            games_number
                .inner_html()
                .as_str()
                .split_whitespace()
                .skip(2)
                .next()
                .map(str::to_string)
        }
        None => None,
    };
}

fn parse_games(doc: &scraper::Html, n: Option<usize>) -> Vec<Game> {
    let selector_list_item = Selector::parse(".search_list_details").unwrap();
    doc.select(&selector_list_item)
        .take(n.unwrap_or(5))
        .map(parse_game)
        .collect()
}

fn parse_game(item: ElementRef) -> Game {
    let selector_title = Selector::parse("a").unwrap();
    let selector_time =
        Selector::parse(".search_list_tidbit, .search_list_tidbit_short, .search_list_tidbit_long")
            .unwrap();

    let name = htmlescape::decode_html(
        &item
            .select(&selector_title)
            .next()
            .expect("Could not find game title")
            .inner_html(),
    )
    .unwrap();

    let times = item.select(&selector_time);
    let entries = times
        .map(|e| e.inner_html().trim().into())
        .tuples::<(String, String)>()
        .collect();

    Game { name, entries }
}

fn print_game(game: &Game) {
    let entries: String = game
        .entries
        .iter()
        .map(|e| format!("{}: {}\n", e.0, e.1))
        .collect();
    let bold_name = format!("\x1B[1m{}\x1B[0m", game.name);
    println!("{}\n{}", bold_name, entries);
}
