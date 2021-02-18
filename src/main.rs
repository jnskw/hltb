use std::process::exit;

use argh::FromArgs;
use prettytable::{cell, format, row, Table};
use scraper::{ElementRef, Selector};

const HLTB_URL: &str = "https://howlongtobeat.com/search_results";

#[derive(Debug, FromArgs)]
/// Test
struct Args {
    #[argh(positional)]
    game: Vec<String>,
    #[argh(option, short = 'n')]
    /// the maximum number of games to display (Default 5)
    number: Option<usize>,
}

#[derive(Debug)]
struct Game {
    name: String,
    main: String,
    extra: String,
    completionist: String,
}

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let args: Args = argh::from_env();
    let game = args.game.join(" ");

    let form_data = &[
        ("queryString", game.as_str()),
        ("t", "games"),
        ("sorthead", "popular"),
    ];

    let res = ureq::post(HLTB_URL)
        .query("page", "1")
        .send_form(form_data)?
        .into_string()?;

    let doc = scraper::Html::parse_document(&res);

    let selector_games_number = Selector::parse("h3").unwrap();
    match doc.select(&selector_games_number).next() {
        Some(games_number) => {
            // Get the number of games from the sentence `We Found 25 Games for \"gris\"`
            if let Some(n) = games_number
                .inner_html()
                .as_str()
                .split_whitespace()
                .skip(2)
                .next()
            {
                let word = if n == "1" { "game" } else { "games" };
                println!("Found {} matching {}", n, word);
            }
        }
        None => {
            println!("Could not find a game for query '{}'", game);
            exit(1);
        }
    }

    let selector_list_item = Selector::parse(".search_list_details").unwrap();
    let games = doc
        .select(&selector_list_item)
        .map(parse_game)
        .take(args.number.unwrap_or(5));

    draw_game_table(games);

    Ok(())
}

fn parse_game(item: ElementRef) -> Game {
    let selector_title = Selector::parse("a").unwrap();
    let selector_time = Selector::parse(".search_list_tidbit").unwrap();

    // TODO less unwrap 🥺
    let name = htmlescape::decode_html(&item.select(&selector_title).next().unwrap().inner_html())
        .unwrap();
    let mut times = item.select(&selector_time);
    let main = times.nth(1).unwrap().inner_html().trim().into();
    let extra = times.nth(1).unwrap().inner_html().trim().into();
    let completionist = times.nth(1).unwrap().inner_html().trim().into();

    Game {
        name,
        main,
        extra,
        completionist,
    }
}

fn draw_game_table(games: impl IntoIterator<Item = Game>) {
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
    table.set_titles(row![b->"Game", b->"Main Story", b->"Main+Extra", b->"Completionist"]);
    games.into_iter().for_each(|game| {
        table.add_row(row![game.name, game.main, game.extra, game.completionist]);
    });
    table.printstd();
}
