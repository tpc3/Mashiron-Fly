use crate::utils::split_codeblock;
use crate::data::new;
use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
    utils::Colour,
};

#[command]
async fn add(ctx: &Context, msg: &Message) -> CommandResult {
    let user = &ctx.http.get_current_user().await.unwrap();
    if msg.guild_id == None {
        return Ok(());
    }
    let why: String;
    match split_codeblock(&msg.content, "yaml") {
        Ok(yaml_str) => {
            if let Err(err) = new(msg.guild_id.unwrap().as_u64(), &yaml_str) {
                why = err;
            } else {
                msg.react(&ctx.http, '👍').await.unwrap();
                return Ok(());
            }
        }
        Err(err) => {
            why = err;
        }
    }
    msg.react(&ctx.http, '🤔').await.unwrap();
    msg.channel_id
        .send_message(&ctx, |new_msg| {
            new_msg.embed(|embed| {
                embed.author(|author| {
                    author.icon_url(user.face());
                    author.name(&user.name);
                    author
                });
                embed.title("error");
                embed.description(why);
                embed.colour(Colour::MAGENTA);
                embed.footer(|f| {
                    f.text(msg.timestamp.to_rfc2822());
                    f
                });
                embed
            });
            new_msg
        })
        .await
        .unwrap();
    Ok(())
}
