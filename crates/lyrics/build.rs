use std::env;
use std::fs::{read_dir, read_to_string};
use std::path::PathBuf;

use anyhow::Result;
use itertools::Itertools;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Song {
    spotify_link: String,
    lyrics: Vec<String>,
}

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=raw");

    let mut songs: Vec<Song> = Vec::new();
    let parentheses_regex = Regex::new(r"\([^)]*\)")?;

    for entry in read_dir("raw")?.into_iter().collect::<Vec<_>>() {
        let raw_contents = read_to_string(entry?.path())?;
        let contents = parentheses_regex.replace_all(raw_contents.trim(), "");
        let mut lyrics =
            contents.split('\n').map(|l| l.trim()).filter(|l| !l.is_empty()).collect::<Vec<_>>();

        let spotify_link = lyrics.remove(0).to_string();

        songs.push(Song {
            spotify_link: spotify_link.trim().to_string(),
            lyrics: lyrics.into_iter().unique().map(|l| l.to_lowercase()).collect(),
        })
    }

    let path: PathBuf = [env::var("OUT_DIR")?, "lyrics.rs".into()].iter().collect();

    uneval::to_file(songs, path)?;

    Ok(())
}
