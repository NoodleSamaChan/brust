#![feature(str_split_once)]

mod commands;
mod imgflip;
mod utils;

use commands::*;
use rand::rngs::SmallRng;
use rand::SeedableRng;
use serenity::{
    async_trait,
    client::bridge::gateway::ShardManager,
    framework::{
        standard::macros::{help, hook},
        standard::{help_commands, Args, CommandGroup, CommandResult, HelpOptions},
        StandardFramework,
    },
    http::Http,
    model::gateway::Ready,
    model::prelude::*,
    prelude::*,
};
use std::collections::HashSet;
use std::env;
use std::sync::Arc;

struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

pub struct Random;

impl TypeMapKey for Random {
    type Value = Arc<Mutex<SmallRng>>;
}

#[help]
#[individual_command_tip = ":crab: To get help with an individual command, pass its name as an argument to this command. :crab:"]
#[wrong_channel = "Hide"]
#[max_levenshtein_distance(3)]
async fn my_help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let _ = help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
    Ok(())
}

#[hook]
async fn after(ctx: &Context, msg: &Message, command_name: &str, command_result: CommandResult) {
    if let Err(why) = command_result {
        let speech_bubble_emoji = "💬".parse::<ReactionType>().unwrap();

        eprintln!("Command '{}' returned error {:?}", command_name, why);

        let _err = msg.react(ctx, speech_bubble_emoji.clone()).await;

        let tmp_emoji = speech_bubble_emoji.clone();

        let want_error_msg = msg
            .await_reaction(ctx)
            .timeout(std::time::Duration::from_secs(60 * 10))
            .filter(move |reaction| reaction.emoji == tmp_emoji)
            .await;
        let _err = msg.delete_reaction_emoji(ctx, speech_bubble_emoji).await;
        if want_error_msg.is_some() {
            let _ = msg.reply(&ctx, why).await;
        }
    }
}

#[tokio::main]
async fn main() {
    kankyo::load(false).expect("Failed to load .env file");

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let http = Http::new_with_token(&token);

    // We will fetch your bot's owners and id
    let (owners, bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            if let Some(team) = info.team {
                owners.insert(team.owner_user_id);
            } else {
                owners.insert(info.owner.id);
            }
            match http.get_current_user().await {
                Ok(bot_id) => (owners, bot_id.id),
                Err(why) => panic!("Could not access the bot id: {:?}", why),
            }
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let framework = StandardFramework::new()
        .configure(|c| {
            c.prefix("!")
                .on_mention(Some(bot_id))
                .owners(owners)
                .delimiters(vec![", ", ",", " "])
        })
        .group(&GENERAL_GROUP)
        .group(&CUTE_GROUP)
        .after(after)
        .help(&MY_HELP);

    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
    }
    let mut rng = SmallRng::from_entropy();

    {
        let mut data = client.data.write().await;
        data.insert::<commands::Tg>(Arc::new(Mutex::new(commands::init_tg(&mut rng))));
        data.insert::<Random>(Arc::new(Mutex::new(rng)));
    }

    let shard_manager = client.shard_manager.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Could not register ctrl+c handler");
        shard_manager.lock().await.shutdown_all().await;
    });

    if let Err(why) = client.start().await {
        eprintln!("Client error: {:?}", why);
    }
}
