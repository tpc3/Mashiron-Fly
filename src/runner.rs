use std::collections::HashMap;
use std::sync::RwLock;

use crate::config::*;
use crate::data::*;
use crate::js::js;
use crate::utils::react;
use chrono::Local;
use fancy_regex::Regex;
use once_cell::sync::Lazy;
use rand::seq::SliceRandom;
use rand::thread_rng;
use serenity::client::Context;
use serenity::model::channel::Message;
use std::time::Duration;
use tokio::time::sleep;
use yaml_rust::Yaml;

static CACHE: Lazy<RwLock<HashMap<String, String>>> = Lazy::new(|| init());

pub async fn run(ctx: &Context, msg: &Message) {
    if let Some(gid) = msg.guild_id {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&gid.as_u64().to_be_bytes());
        hasher.update(msg.content.as_bytes());
        let hash = &hasher.finalize().to_string();
        if CACHE.read().unwrap().contains_key(hash) {
            let cache = CACHE.read().unwrap()[hash].clone();
            msg.reply(&ctx.http, cache).await.unwrap();
            return;
        }
        let yaml = get(msg.guild_id.unwrap().as_u64());
        if msg.content.starts_with(&CONFIG.infos.prefix) {
            let content: Vec<&str> = msg.content.split(" ").collect();
            let cmd: &str = &content[0].replace(&CONFIG.infos.prefix, "");
            runner(&ctx, &msg, &yaml, cmd).await;
        }
        let content = msg.content.clone();
        if let Some(yaml_hash) = yaml.as_hash() {
            for (key, value) in yaml_hash {
                match value {
                    Yaml::String(_) => {
                        let key_str = key.as_str().unwrap();
                        if content == key_str {
                            runner(&ctx, &msg, &yaml, key_str).await;
                        }
                    }
                    Yaml::Hash(_) => {
                        //OR trigger
                        let mut trigger = false;
                        match &value["trigger"] {
                            Yaml::Hash(_) => match &value["trigger"]["content"] {
                                Yaml::String(s) => {
                                    if let Ok(re) = Regex::new(&s) {
                                        trigger = re.is_match(&content).unwrap_or(false);
                                    }
                                }
                                Yaml::Array(arr) => {
                                    for i in arr {
                                        if let Yaml::String(s) = i {
                                            if let Ok(re) = Regex::new(&s) {
                                                let result = re.is_match(&content).unwrap_or(false);
                                                if result && !trigger {
                                                    trigger = true
                                                }
                                            }
                                        }
                                    }
                                }
                                _ => (),
                            },
                            Yaml::String(s) => {
                                if let Ok(re) = Regex::new(&s) {
                                    trigger = re.is_match(&content).unwrap_or(false);
                                }
                            }
                            _ => (),
                        }

                        match &value["trigger"]["uid"] {
                            Yaml::Integer(i) => {
                                if *i as u64 == *msg.author.id.as_u64() && trigger {
                                    runner(&ctx, &msg, &yaml, key.as_str().unwrap()).await;
                                }
                            }
                            Yaml::Array(arr) => {
                                for val in arr {
                                    if let Yaml::Integer(i) = val {
                                        if *i as u64 == *msg.author.id.as_u64() && trigger {
                                            runner(&ctx, &msg, &yaml, key.as_str().unwrap()).await;
                                        }
                                    }
                                }
                            }
                            _ => {
                                if trigger {
                                    runner(&ctx, &msg, &yaml, key.as_str().unwrap()).await;
                                }
                            }
                        }
                    }
                    _ => (),
                }
            }
        }
    }
}

fn init() -> RwLock<HashMap<String, String>> {
    RwLock::new(HashMap::new())
}

// Return section of the yaml.
async fn runner(ctx: &Context, msg: &Message, yaml: &Yaml, cmd: &str) {
    match &yaml[cmd] {
        Yaml::String(s) => {
            let reply = builder(&ctx, &msg, &s).await;
            msg.reply(&ctx.http, reply).await.unwrap();
        }
        Yaml::Hash(_) => {
            let mut reply_msg = Result::Ok(msg.clone());
            let mut ifreply = false;
            match &yaml[cmd]["return"] {
                Yaml::String(s) => {
                    let reply = builder(&ctx, &msg, &s).await;
                    ifreply = true;
                    reply_msg = msg.reply(&ctx.http, reply).await;
                    msg.react(&ctx.http, '👀').await.unwrap();
                }
                Yaml::Array(arr) => {
                    let rnd = arr.choose(&mut thread_rng());
                    if let Some(chosen) = rnd {
                        if let Some(s) = chosen.as_str() {
                            let reply = builder(&ctx, &msg, &s.to_string()).await;
                            ifreply = true;
                            reply_msg = msg.reply(&ctx.http, reply).await;
                            msg.react(&ctx.http, '👀').await.unwrap();
                        }
                    }
                }
                _ => (),
            }
            match &yaml[cmd]["react"] {
                Yaml::String(_) => {
                    react(&ctx, &msg, &yaml[cmd]["react"]).await;
                }
                Yaml::Integer(_) => {
                    react(&ctx, &msg, &yaml[cmd]["react"]).await;
                }
                Yaml::Array(arr) => {
                    for i in arr {
                        react(&ctx, &msg, &i).await;
                    }
                }
                _ => (),
            }
            match &yaml[cmd]["delete"] {
                Yaml::Integer(i) => {
                    if ifreply {
                        if let Ok(reply_msg) = reply_msg {
                            sleep(Duration::from_secs(*i as u64)).await;
                            //ToDo: Error handling
                            reply_msg.delete(&ctx.http).await.unwrap();
                        }
                    }
                }
                _ => (),
            }
            if let Yaml::String(script) = &yaml[cmd]["js"] {
                let res = js(ctx, msg, &script).await;
                if !res.trim().is_empty() {
                    msg.reply(&ctx.http, res).await.unwrap();
                }
            }
        }
        _ => (),
    }
}

async fn builder(ctx: &Context, msg: &Message, template: &String) -> String {
    let mut reply: String = template.clone();
    if reply.contains("{") {
        // Variable detected
        let mut nickname = msg.author_nick(&ctx.http).await.unwrap_or("".to_string());
        if nickname == "" {
            nickname = msg.author.name.to_string()
        }
        let map = HashMap::from([
            ("{uid}", msg.author.id.to_string()),
            ("{nickname}", nickname),
        ]);
        for (key, value) in map.into_iter() {
            reply = reply.replace(key, &value)
        }
    }
    if reply.contains("$(") {
        //args detected
        for (i, value) in msg
            .content
            .split(" ")
            .collect::<Vec<&str>>()
            .iter()
            .enumerate()
        {
            reply = reply.replace(&format!("$({})", i), value);
        }
    }
    if reply.contains("%") {
        //date detected
        reply = Local::now().format(&reply).to_string();
    }
    reply
}
