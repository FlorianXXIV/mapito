use std::{fmt::Display, str::FromStr};

use crate::{
    cli::input::read_line_to_string, mc_info::MVDescriptor, mrapi::client::ApiClient,
    util::error::ApiError,
};

/// repeats prompt to search for mods and returns a vector of the slugs of all chosen mods
pub fn search_mods(client: &ApiClient, version_desc: Option<&MVDescriptor>) -> Vec<String> {
    println!("Search for mods and add them to the pack.");

    let mut mods: Vec<String> = Vec::new();

    loop {
        let query = match prompt_for::<String>("Please enter next query") {
            Some(q) => q,
            None => {
                break;
            }
        };
        match query_reader(&query, client, version_desc) {
            Ok(slug) => mods.push(slug),
            Err(e) => println!("{}", e),
        }
    }

    mods
}

/// Prompt user for type T, if user eners 'q' returns None
/// otherwise returns Some(T)
pub fn prompt_for<T: FromStr>(prompt: &str) -> Option<T>
where
    T::Err: Display,
{
    loop {
        println!("{} or press 'q' to quit:", prompt);
        let result = read_line_to_string();
        if result == "q" {
            break;
        }
        let obj = match T::from_str(&result) {
            Ok(obj) => obj,
            Err(e) => {
                println!("parsing input failed");
                println!("{}", e);
                continue;
            }
        };
        return Some(obj);
    }
    None
}

/// Prompt the user for multiple objects of type T
pub fn prompt_multiple<T: FromStr + Display>(prompt: &str) -> Vec<T>
where
    T::Err: Display,
{
    let mut ret: Vec<T> = Vec::new();
    println!("Enter multiple");
    loop {
        match prompt_for::<T>(prompt) {
            Some(obj) => ret.push(obj),
            None => break,
        };
        println!(
            "Currently selected {}",
            ret.iter()
                .map(|obj| obj.to_string() + " ")
                .collect::<String>()
        );
    }

    ret
}

/// prompt user to select one item of a list.
pub fn list_select<T: Display + Copy>(prompt: &str, options: &[T]) -> Option<T> {
    println!("{prompt}:");
    for (i, t) in options.iter().enumerate() {
        println!("[{i}]: {t}");
    }
    let j = prompt_for::<usize>("Select a Number")?;
    Some(options[j])
}

fn query_reader(
    query: &String,
    client: &ApiClient,
    version_desc: Option<&MVDescriptor>,
) -> Result<String, ApiError> {
    let mut offset = 0;
    let facets = match version_desc {
        Some(vd) => Some(vec![
            vec![("versions".to_string(), vd.mc_ver.to_string())],
            vec![("categories".to_string(), vd.loader.to_string())],
        ]),

        None => None,
    };
    loop {
        let slugs = client.search(query, None, Some(offset), &facets)?;
        if slugs.is_empty() {
            if offset >= 10 {
                offset -= 10;
                continue;
            }
            break;
        }
        println!(
            "Select mod from 0 to {} or 'p'/'n' to change page, enter 'q' to quit.",
            slugs.len() - 1
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
                let i: usize = match resp.parse() {
                    Ok(u) => u,
                    Err(e) => {
                        println!("{}", e);
                        continue;
                    }
                };
                return Ok(slugs[i].clone());
            }
        }
    }
    Err(ApiError::not_found())
}
