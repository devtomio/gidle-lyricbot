#![warn(clippy::pedantic)]
#![allow(clippy::missing_panics_doc, clippy::must_use_candidate)]

use once_cell::sync::Lazy;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Song {
    spotify_link: String,
    lyrics: Vec<String>,
}

pub static SONGS: Lazy<Vec<Song>> =
    Lazy::new(|| include!(concat!(env!("OUT_DIR"), "/lyrics.rs")));

pub fn get_random_song<'a>() -> &'a Song {
    SONGS.choose(&mut rand::thread_rng()).unwrap()
}

pub fn get_random_lyric<'a>() -> (&'a str, &'a str) {
    let song = get_random_song();

    (song.lyrics.choose(&mut rand::thread_rng()).unwrap(), &song.spotify_link)
}
