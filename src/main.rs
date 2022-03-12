// Reply with user-provided value may cause some "Err"s, but we don't want to handle and show it every single time.
#![allow(unused_must_use)]

mod cmds;
mod config;
mod data;
mod runner;
mod utils;
mod js;
pub mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

use std::fs;

use cmds::{add::*, delete::*, help::*, list::*, ping::*};
use runner::run;
use serenity::{
    async_trait,
    client::{Client, Context, EventHandler},
    framework::{standard::macros::group, StandardFramework},
    model::{channel::Message, gateway::Ready, prelude::Activity},
};
use std::time::Instant;
use tracing::info;

#[group]
#[commands(ping, help, list, delete, add)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot || msg.guild_id == None {
            return;
        }
        if config::CONFIG.debug {
            let now = Instant::now();
            run(&ctx, &msg).await;
            println!("Hook took {:.2?}", now.elapsed());
        } else {
            run(&ctx, &msg).await;
        }
    }
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
        ctx.set_activity(Activity::playing(&config::CONFIG.infos.activity))
            .await;
    }
}

#[tokio::main]
async fn main() {
    fs::create_dir_all(&config::CONFIG.data).unwrap();

    let framework = StandardFramework::new()
        .configure(|c| {
            c.prefix(&config::CONFIG.infos.prefix);
            c.allow_dm(false);
            c
        })
        .group(&GENERAL_GROUP);

    // Login with a bot token from the environment
    let mut client = Client::builder(&config::CONFIG.token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    // start listening for events by starting a single shard
    if let Err(why) = client.start_shards(config::CONFIG.shards).await {
        println!("An error occurred while running the client: {:?}", why);
    }
}
