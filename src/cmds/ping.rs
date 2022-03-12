use crate::{config::CONFIG};
use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::{
        prelude::*,
    },
    prelude::*,
    utils::Colour,
};
use crate::utils::link;

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    let user = &ctx.http.get_current_user().await.unwrap();
    msg.channel_id
        .send_message(&ctx.http, |msg_res| {
            msg_res.embed(|embed| {
                embed.author(|author| {
                    author.icon_url(user.face());
                    author.name(&user.name);
                    author
                });
                embed.title("Pong!");
                embed.image(&CONFIG.infos.image);
                embed.footer(|f| {
                    f.text(msg.timestamp.to_rfc2822());
                    f
                });
                embed.field(
                    link(
                        &crate::built_info::PKG_NAME.to_string(),
                        &crate::built_info::PKG_HOMEPAGE.to_string(),
                    ),
                    format!(
                        "{} {} {}",
                        crate::built_info::PKG_VERSION,
                        crate::built_info::RUSTC,
                        crate::built_info::TARGET
                    ),
                    false,
                );
                embed.colour(Colour::DARK_GREEN);
                embed
            });
            msg_res.reference_message(msg);
            msg_res
        })
        .await
        .unwrap();
    Ok(())
}
