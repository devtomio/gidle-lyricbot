#![warn(clippy::pedantic)]
#![allow(clippy::similar_names)]

#[macro_use]
extern crate tracing;

mod banner;
mod healthcheck;

use std::env;
use std::sync::Arc;

use anyhow::{Context, Result};
use dotenvy::dotenv;
use salvo::listener::TcpListener;
use salvo::{Router, Server};
#[cfg(unix)]
use tokio::signal::unix as signal;
#[cfg(windows)]
use tokio::signal::windows as signal;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use twitter_v2::authorization::Oauth1aToken;
use twitter_v2::TwitterApi;

// At second :00, at minute :00, every 2 hours starting at 00am, of every day
const POST_SCHEDULE: &str = "0 0 0/2 ? * * *";

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenv();

    banner::print();

    let subscriber = FmtSubscriber::builder().with_max_level(Level::INFO).finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let token = Oauth1aToken::new(
        env::var("API_KEY")?,
        env::var("API_KEY_SECRET")?,
        env::var("ACCESS_TOKEN")?,
        env::var("ACCESS_TOKEN_SECRET")?,
    );

    let twitter = Arc::new(TwitterApi::new(token));

    let current_user = twitter.get_users_me().send().await?;
    let current_user = current_user.data().context("Current user not present")?;

    info!("current user: {} (@{})", current_user.name, current_user.username);

    let mut sched = JobScheduler::new().await?;
    let job = Job::new_async(POST_SCHEDULE, move |_uuid, _lock| {
        let twitter = twitter.clone();

        Box::pin(async move {
            let (lyric, spotify_link) = lyrics::get_random_lyric();
            let tweet = twitter.post_tweet().text(lyric.to_string()).send().await.unwrap();
            let tweet = tweet.data().unwrap();

            twitter
                .post_tweet()
                .text(spotify_link.to_string())
                .in_reply_to_tweet_id(tweet.id)
                .send()
                .await
                .unwrap();

            info!("posted tweet ({}).", tweet.id);
        })
    })?;

    let guid = job.guid();

    info!("created job ({guid}).");

    sched.add(job).await.unwrap();
    sched.start().await?;

    let healthcheck_server_handle = tokio::spawn(async move {
        let port = env::var("PORT").map(|port| port.parse::<u16>().unwrap_or(3000)).unwrap_or(3000);
        let host = env::var("HOST").unwrap_or("0.0.0.0".to_string());
        let address = format!("{host}:{port}");
        let router = Router::new().path("/healthcheck").get(healthcheck::healthcheck);

        info!("healthcheck server listening on {address}.");
        Server::new(TcpListener::bind(&address)).serve(router).await;
    });

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
    healthcheck_server_handle.abort();

    Ok(())
}
