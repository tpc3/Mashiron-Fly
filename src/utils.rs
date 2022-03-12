use serenity::{
    client::Context,
    model::{channel::Message, id::EmojiId},
};
use yaml_rust::Yaml;

pub fn codeblock(s: &str, lang: &str) -> String {
    format!("```{}\n{}\n```", lang, s)
}

pub fn link(title: &str, link: &str) -> String {
    format!("[{}]({})", title, link)
}

pub fn split_codeblock(s: &str, lang: &str) -> Result<String, String> {
    let mut i = 0;
    let mut inner = "".to_string();
    for v in s.lines() {
        if i == 1 {
            if v != format!("```{}", lang) {
                return Err("Invalid markdown syntax or language".to_string());
            }
        } else if i > 1 {
            if v == "```" {
                return Ok(inner);
            } else {
                inner += &format!("{}\n", v);
            }
        }
        i += 1;
    }
    Err("EOF: No codeblock end tag".to_string())
}

pub async fn react(ctx: &Context, msg: &Message, yaml: &Yaml) {
    match yaml {
        Yaml::String(s) => {
            if s.chars().count() == 1 {
                msg.react(&ctx.http, s.chars().next().unwrap()).await;
            }
        }
        Yaml::Integer(i) => {
            if let Some(guild) = msg.guild(&ctx.cache).await {
                let emoji = EmojiId(*i as u64);
                if guild.emojis.contains_key(&emoji) {
                    msg.react(&ctx.http, guild.emojis[&emoji].clone()).await;
                }
            }
        }
        _ => (),
    }
}