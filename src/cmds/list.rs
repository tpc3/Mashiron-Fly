
use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
    utils::Colour,
};
use crate::{data::{get, dump}, utils::codeblock};

#[command]
async fn list(ctx: &Context, msg: &Message) -> CommandResult {
    let user = &ctx.http.get_current_user().await.unwrap();
    if msg.guild_id == None {
        return Ok(())
    }
    let mut out_str = dump(&get(msg.guild_id.unwrap().as_u64()));
    if out_str.len() == 0 {
        out_str = "Empty".to_string();
    }
    if out_str.chars().count() > 2000 {
        msg.channel_id.send_files(&ctx.http, vec![(out_str.as_bytes(), "list.yaml")], |m| m).await.unwrap();
        return Ok(())
    }
    msg.channel_id
        .send_message(&ctx, |new_msg| {
            new_msg.embed(|embed| {
                embed.author(|author| {
                    author.icon_url(user.face());
                    author.name(&user.name);
                    author
                });
                embed.title("list");
                embed.description(codeblock(&out_str, "yaml"));
                embed.colour(Colour::DARK_GREEN);
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
