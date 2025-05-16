use reqwest::blocking::Client;

use crate::{
    cli::input::read_line_to_string, config::Configuration, mrapi::interactions::search_package,
};

/// repeats prompt to search for mods and returns a vector of the slugs of all chosen mods
pub fn search_mods(client: &Client, config: &Configuration) -> Vec<String> {
    println!("Search for mods and add them to the pack.");

    let mut mods: Vec<String> = Vec::new();

    loop {
        println!("Please enter next query or enter 'q' to quit.");
        let query = read_line_to_string();
        if query == "q" {
            break;
        } else {
            match query_reader(&query, client, config) {
                Some(slug) => mods.push(slug),
                None => println!("No mods Found"),
            }
        }
    }

    mods
}

fn query_reader(query: &String, client: &Client, config: &Configuration) -> Option<String> {
    let mut offset = 0;
    loop {
        let slugs = search_package(client, query, config.staging, None, Some(offset));
        match slugs {
            Some(sl) => {
                println!(
                    "Select mod from 0 to {} or 'p'/'n' to change page, enter 'q' to quit.",
                    sl.len() - 1
                );
                let resp = read_line_to_string();
                match resp.as_str() {
                    "n" => offset += 10,
                    "p" => {
                        if offset < 10 {
                            println!("Already at first page");
                            continue;
                        } else {
                            offset -= 10;
                        }
                    }
                    "q" => {
                        break;
                    }
                    _ => {
                        let i: usize = resp.parse().expect("parse");
                        return Some(sl[i].clone());
                    }
                }
            }
            None => {
                break;
            }
        }
    }
    return None;
}
