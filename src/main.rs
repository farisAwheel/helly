/* TODO
    -[] proper error handling
        -[] what to do on disconnect
        -[] what to do if poise throws an error
        -[] define your own proper error handling
    
    -[] tierlist archiver
        -[] if message w/ image is sent in tylist
            -[] download the image
            -[] scan for common tierlistmaker colors (the gray background, the tier colors, the custom name colors)
            -[] if one is found, send a message to ask if you want to archive it then run through archive flow
        -[] create an archive command
            -[] get contributors
            -[] get the message
            -[] send the image in the archive

    -[] debug mode
        -[] print command debug information
        -[] enable deregister and register button command
        -[] register to guild instead of global

    -[] refactor commands crate to be a folder of commands
*/

mod commands;

use poise::serenity_prelude::{self as serenity, EditMessage, Event, futures};
use regex::Regex;
use std::sync::OnceLock;
use std::time::Duration;
use url::Url;
use futures::StreamExt;

struct Data {} // if i choose to implement some sort of db, this data struct will help me to pass the model of my data forward (i think)
type Error = Box<dyn std::error::Error + Send + Sync>; // this Error type is a async dynamic error type that fits inside the Box typing
type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() {
    println!("running server...");

    dotenvy::dotenv().expect("failed to read .env");

    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN environment variable");
    let intents = serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;
    let guild = std::env::var("GUILD_ID") // the guild id is unecessary when registering to global, but great for testing
        .expect("missing GUILD_ID environment variable")
        .parse::<u64>()
        .map(serenity::model::id::GuildId::new)
        .expect("GUILD_ID is invalid");

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            prefix_options: poise::PrefixFrameworkOptions {
                mention_as_prefix: true,
                ..Default::default()
            },
            commands: vec![
                commands::ping(),
            ],
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_in_guild(ctx, &framework.options().commands, guild).await?; // currently registering all commands to guild, will be global on final release
                Ok(Data {})
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}

async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    _data: &Data,
) -> Result<(), Error> {
    static URL_REGEX: OnceLock<Regex> = OnceLock::new();
    static TIERLIST_CHANNEL: OnceLock<u64> = OnceLock::new();

    // set up the regex only once, statically
    let re = URL_REGEX
        .get_or_init(|| { 
            Regex::new(r"https?:\/\/(www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_\+.~#?&//=]*)").expect("Invalid regex")});

    let tierlist_channel = TIERLIST_CHANNEL
            .get_or_init(|| {
                std::env::var("TIERLIST_CHANNEL").expect("missing TIERLIST_CHANNEL environment variable")
                    .parse::<u64>().expect("invalid tierlist channel id")});

    match event {
        serenity::FullEvent::Message { new_message } => {
            if new_message.channel_id.get() == *tierlist_channel && !new_message.attachments.is_empty() && !new_message.author.bot {
                println!("no");
            }

            let Some(ma) = re.captures(&new_message.content) else {
                return Ok(());
            };
            
            let url = Url::parse(ma.get(0)
                .expect("Invalid match").as_str())
                .expect("Invalid url");

            let Some(res) = create_emebed_url(&url) else {
                return Ok(())
            };

            // println!("{res}"); // ik there's a way to run as debug, this should be excluded from --release but i still want it

            // taken directly from poise docs, waits for the embed to exist, then attempts to remove it, or waits 2000ms to attempt to remove it
            new_message.reply(ctx, res).await?;
            let mut new_message = new_message.clone();
            let mut message_updates = serenity::collector::collect(&ctx.shard, move |ev| match ev {
                Event::MessageUpdate(x) if x.id == new_message.id => Some(()),
                _ => None,
            });
            let _ = tokio::time::timeout(Duration::from_millis(2000), message_updates.next()).await;
            new_message.edit(ctx, EditMessage::new().suppress_embeds(true)).await?;
        },
        _ => {}
    }
    Ok(())
}

/// This function borrows a URL and returns Some(String) based on if the URL can be matched
/// to one of the supported media sites
/// 
/// The returned string is a URL to an embed service, i.e. if the inputted url is twitter.com or x.com
/// the string would be fxtwitter.com as that url will embed the tweet automatically in Discord
fn create_emebed_url(url: &Url) -> Option<String> {
    if url.path() == "/" {
        return None
    }

    let Some(base_url) = url.host() else {
        return None
    };

    let url_str = url.as_str();
    
    // can make a small optimization by modifying the url struct directly instead of string replace
    // BUT lowkey it's not that serious
    match base_url.to_string().as_str() {
        "x.com" | "www.x.com" => Some(url_str.replace("x.com", "fxtwitter.com")),
        "twitter.com" | "www.twitter.com" => Some(url_str.replace("twitter.com", "fxtwitter.com")),
        "tiktok.com" | "www.tiktok.com" => Some(url_str.replace("tiktok.com", "tnktok.com")),
        _ => None
    }   
}