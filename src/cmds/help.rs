use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
    utils::Colour,
};
use crate::config::CONFIG;

#[command]
async fn help(ctx: &Context, msg: &Message) -> CommandResult {
    let user = &ctx.http.get_current_user().await.unwrap();
    msg.channel_id
        .send_message(&ctx, |new_msg| {
            new_msg.embed(|embed| {
                embed.author(|author| {
                    author.icon_url(user.face());
                    author.name(&user.name);
                    author
                });
                embed.title("help");
                embed.description(&CONFIG.infos.help);
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
