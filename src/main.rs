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

use std::env;
use std::fs::File;
use std::io::{Read, stdin};

use anyhow::{Context, anyhow};
use camino::Utf8PathBuf;
use clap::Parser;
use reqwest::Url;
use reqwest::blocking::Client;
use serde::Deserialize;

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

    /// The path of the file to upload.
    ///
    /// If not provided, the file will be read from standard input.
    path: Option<Utf8PathBuf>,
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

    let mut url = Url::parse(API_URL).unwrap();
    url.query_pairs_mut().append_pair("api_key", &api_key);

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
