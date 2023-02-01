#![warn(clippy::pedantic)]

use std::borrow::Cow;
use std::collections::HashMap;
use std::env;

use anyhow::Result;
use dotenvy::dotenv;
use oauth1::{authorize, Token};

fn main() -> Result<()> {
    let _ = dotenv();

    let api_key = env::var("API_KEY")?;
    let api_key_secret = env::var("API_KEY_SECRET")?;
    let consumer_token = Token::new(api_key, api_key_secret);
    let mut params = HashMap::new();

    params.insert("oauth_callback", Cow::from("oob"));

    let oauth = authorize(
        "POST",
        "https://api.twitter.com/oauth/request_token",
        &consumer_token,
        None,
        Some(params),
    );

    println!("{oauth}");

    Ok(())
}
