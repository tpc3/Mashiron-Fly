use serenity::{
    client::Context,
    model::{channel::{Message, Channel, ChannelCategory}},
};
use std::{time::Duration};
use tokio::time::timeout;

use crate::config::CONFIG;
use once_cell::sync::Lazy;

pub static TIMEOUT: Lazy<Duration> = Lazy::new(|| Duration::from_millis(CONFIG.js.timeout));

pub struct Args<'a> {
    pub ctx: &'a Context,
    pub msg: &'a Message,
    pub js: &'a str,
    pub nickname: &'a str,
    pub args: Vec<&'a str>,
    pub channel: &'a Channel,
    pub category: Option<ChannelCategory>,
}

pub async fn js(ctx: &Context, msg: &Message, js: &str) -> String {
    let channel = &msg.channel(&ctx.cache).await.unwrap();
    let args = Args {
        ctx,
        msg,
        js,
        nickname: &msg.author_nick(&ctx.http).await.unwrap_or("".to_string()),
        channel,
        args: msg.content.split(" ").collect(),
        category: channel.clone().category(),
    };
    if CONFIG.js.enabled {
        let res = timeout(*TIMEOUT, eval(&args)).await;
        match res {
            Ok(ok) => ok,
            Err(_) => return "Timeout".to_string(),
        }
    } else {
        return "JS future is disabled".to_string();
    }
}

async fn eval(args: &Args<'_>) -> String {
    let context = quick_js::Context::builder()
        .memory_limit(CONFIG.js.memory)
        .build()
        .unwrap();

    //Vals
    context.set_global("scriptArgs", args.args.clone()).unwrap();
    context
        .set_global("name", &args.msg.author.name as &str)
        .unwrap();
    context.set_global("nickname", args.nickname).unwrap();
    context
        .set_global("channelId", &args.msg.channel_id.to_string() as &str)
        .unwrap();
    if let Some(ref_msg) = &args.msg.referenced_message {
        context
            .set_global("referencedMessage", ref_msg.content.clone())
            .unwrap();
    }
    context
        .set_global("author_id", args.msg.author.id.to_string())
        .unwrap();
    context
        .set_global(
            "author_avatar",
            args.msg.author.avatar_url().unwrap_or("".to_string()),
        )
        .unwrap();
    context
        .set_global("channel_isNsfw", args.channel.is_nsfw())
        .unwrap();
    if let Some(cat) = &args.category {
        context
            .set_global("channel_category_id", cat.id.to_string())
            .unwrap();
    } else {
        context.set_global("channel_isNsfw", "").unwrap();
    }

    context
        .eval_as::<String>(args.js)
        .unwrap_or_else(|err| format!("{}", err))
}
