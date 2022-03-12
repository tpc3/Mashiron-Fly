use crate::data::remove;
use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
    utils::Colour,
};

#[command]
async fn delete(ctx: &Context, msg: &Message) -> CommandResult {
    let user = &ctx.http.get_current_user().await.unwrap();
    if msg.guild_id == None {
        return Ok(());
    }
    let content: Vec<&str> = msg.content.split(" ").collect();
    if content.len() == 2 && remove(msg.guild_id.unwrap().as_u64(), content[1]) {
        msg.react(&ctx.http, '👍').await.unwrap();
        return Ok(());
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
                embed.description("Invalid name(Maybe not exists?)");
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
