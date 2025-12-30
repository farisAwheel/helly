use crate::{Context, Error};

/// Get the ping of the server to Discord
#[poise::command(slash_command, prefix_command)]
pub async fn ping(
    ctx: Context<'_>,
) -> Result<(), Error> {
    let ping = ctx.ping().await;
    if ping.as_nanos() == 0 {
        ctx.say("the bot just connected, so the ping has not populated yet").await?;
        return Ok(());
    }
    let response = format!("the ping is currently {}ms", ping.as_millis());
    ctx.say(response).await?;
    Ok(())
}
