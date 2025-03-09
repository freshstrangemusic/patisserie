/* patisserie - A CLI for pastery.net
 * Copyright (C) 2025  Beth Rennie
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

mod language;

use std::env;
use std::fs::File;
use std::io::{Read, stdin};

use anyhow::{Context, anyhow};
use camino::{Utf8Path, Utf8PathBuf};
use clap::Parser;
use reqwest::Url;
use reqwest::blocking::Client;
use serde::Deserialize;

use crate::language::{guess_language, parse_language};

const API_URL: &str = "https://www.pastery.net/api/paste/";
const API_KEY_ENV_VAR: &str = "PASTERY_API_KEY";

#[derive(Parser)]
/// A CLI for https://www.pastery.net, the sweetest pastebin in the world.
struct Options {
    /// Your pastery API key.
    ///
    /// If not provided, it will be read from the PASTERY_API_KEY environment
    /// variable.
    ///
    /// You can find this at https://www.pastery.net/account/.
    #[arg(long = "api-key")]
    api_key: Option<String>,

    /// The duration that this paste will live for.
    ///
    /// After this time, the paste will be deleted.
    ///
    /// You can specify a period of minutes or a value followed by one of the following units:
    /// m(inute), h(our), d(ay), mo(nth), y(ear)
    #[arg(short, long = "duration", default_value = "1d", value_parser = parse_duration)]
    duration: u32,

    /// The language for the paste.
    ///
    /// If not provided, patisserie will attempt to guess based on the file
    /// extension. You can use the special value "autodetect" to have pastery
    /// detect the language.
    #[arg(short, long = "lang", value_parser = parse_language)]
    language: Option<&'static str>,

    /// The title of the paste.
    ///
    /// If not provided, the name of the file will be used instead.
    #[arg(short, long)]
    title: Option<String>,

    /// The number of times the paste can be viewed before expiring.
    ///
    /// If not provided, the paste will not have view-based expiration.
    #[arg(long)]
    max_views: Option<u32>,

    /// The path of the file to upload.
    ///
    /// If not provided, the file will be read from standard input.
    path: Option<Utf8PathBuf>,
}

fn parse_duration(s: &str) -> Result<u32, anyhow::Error> {
    const ONE_MINUTE: u32 = 1;
    const ONE_HOUR: u32 = 60;
    const ONE_DAY: u32 = ONE_HOUR * 24;
    const ONE_WEEK: u32 = ONE_DAY * 7;
    const ONE_MONTH: u32 = ONE_DAY * 30;
    const ONE_YEAR: u32 = ONE_DAY * 365;
    const ONE_HUNDRED_YEARS: u32 = ONE_YEAR * 100;

    fn too_long(s: &str) -> anyhow::Error {
        anyhow!("Duration `{}' is too long; maximum duration is 100y", s)
    }

    let (amount, unit) = s
        .find(|c: char| !c.is_ascii_digit())
        .map(|idx| s.split_at(idx))
        .unwrap_or_else(|| (s, "m"));

    let amount: u32 = amount.parse().expect("amount is entirely ascii digits");

    let scale = match unit {
        "m" => ONE_MINUTE,
        "h" => ONE_HOUR,
        "d" => ONE_DAY,
        "w" => ONE_WEEK,
        "mo" => ONE_MONTH,
        "y" => ONE_YEAR,
        _ => {
            return Err(anyhow!(
                "Unknown unit `{}'; expected one of `m', `h', `d', `w', `mo', or `y'",
                unit
            ));
        }
    };

    amount
        .checked_mul(scale)
        .ok_or_else(|| too_long(s))
        .and_then(|t| {
            if t > ONE_HUNDRED_YEARS {
                Err(too_long(s))
            } else {
                Ok(t)
            }
        })
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Response {
    Paste { url: String },
    Error { error_msg: String },
}

impl Response {
    fn into_result(self) -> Result<String, anyhow::Error> {
        match self {
            Self::Paste { url } => Ok(url),
            Self::Error { error_msg } => Err(anyhow!(error_msg)),
        }
    }
}

fn main() -> Result<(), anyhow::Error> {
    let options = Options::parse();

    // We do not use the `env` feature of clap because it will print the value of
    // environment variables in help text.
    let api_key = options
        .api_key
        .map_or_else(|| env::var(API_KEY_ENV_VAR), Ok)?;

    let mut buffer = String::new();
    if let Some(path) = &options.path {
        File::open(path)
            .with_context(|| format!("Could not open file `{}' for reading", path))?
            .read_to_string(&mut buffer)
            .with_context(|| format!("Could not read file `{}'", path))?;
    } else {
        stdin()
            .read_to_string(&mut buffer)
            .context("Could not read from stdin")?;
    }

    let language = options
        .language
        .or_else(|| options.path.as_deref().and_then(guess_language))
        .unwrap_or("autodetect");

    let title = options.title.or_else(|| {
        options
            .path
            .as_deref()
            .and_then(Utf8Path::file_name)
            .map(ToOwned::to_owned)
    });

    let mut url = Url::parse(API_URL).unwrap();
    {
        let mut query = url.query_pairs_mut();

        query
            .append_pair("api_key", &api_key)
            .append_pair("duration", &options.duration.to_string())
            .append_pair("language", language);

        if let Some(title) = title {
            query.append_pair("title", &title);
        }

        if let Some(max_views) = options.max_views {
            query.append_pair("max_views", &max_views.to_string());
        }
    }

    let client = Client::new();

    let paste_url = client
        .post(url)
        .body(buffer)
        .send()
        .context("Could not make HTTP request")?
        .json::<Response>()
        .context("Could not parse JSON response")?
        .into_result()?;

    println!("{}", paste_url);

    Ok(())
}
