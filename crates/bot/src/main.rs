#![warn(clippy::pedantic)]
#![allow(clippy::similar_names)]

#[macro_use]
extern crate tracing;

use std::env;
use std::sync::Arc;

use anyhow::Result;
use dotenvy::dotenv;
use egg_mode::auth::verify_tokens;
use egg_mode::tweet::DraftTweet;
use egg_mode::{KeyPair, Token};
#[cfg(unix)]
use tokio::signal::unix as signal;
#[cfg(windows)]
use tokio::signal::windows as signal;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

// At second :00, at minute :00, every 2 hours starting at 00am, of every day
const POST_SCHEDULE: &str = "0 0 0/2 ? * * *";

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenv();

    let subscriber = FmtSubscriber::builder().with_max_level(Level::INFO).finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let consumer = KeyPair::new(env::var("API_KEY")?, env::var("API_KEY_SECRET")?);
    let access = KeyPair::new(env::var("ACCESS_TOKEN")?, env::var("ACCESS_TOKEN_SECRET")?);
    let token = Arc::new(Token::Access {
        consumer,
        access,
    });

    let current_user = verify_tokens(&token).await?;
    info!("current user: {} (@{})", current_user.name, current_user.screen_name);

    let mut sched = JobScheduler::new().await?;
    let job = Job::new_async(POST_SCHEDULE, move |_uuid, _lock| {
        let token = token.clone();

        Box::pin(async move {
            let (lyric, spotify_link) = lyrics::get_random_lyric();
            let tweet = DraftTweet::new(lyric).send(&token).await.unwrap();

            DraftTweet::new(spotify_link).in_reply_to(tweet.id).send(&token).await.unwrap();

            info!("posted tweet ({}).", tweet.id);
        })
    })?;

    let guid = job.guid();

    info!("created job ({guid}).");

    sched.add(job).await.unwrap();
    sched.start().await?;

    #[cfg(unix)]
    {
        let [mut s1, mut s2, mut s3] = [
            signal::signal(signal::SignalKind::hangup()).unwrap(),
            signal::signal(signal::SignalKind::interrupt()).unwrap(),
            signal::signal(signal::SignalKind::terminate()).unwrap(),
        ];

        tokio::select! {
            v = s1.recv() => v.unwrap(),
            v = s2.recv() => v.unwrap(),
            v = s3.recv() => v.unwrap(),
        };
    }

    #[cfg(windows)]
    {
        let (mut s1, mut s2) = (signal::ctrl_c().unwrap(), signal::ctrl_break().unwrap());

        tokio::select! {
            v = s1.recv() => v.unwrap(),
            v = s2.recv() => v.unwrap(),
        };
    }

    sched.remove(&guid).await?;
    sched.shutdown().await?;

    Ok(())
}
