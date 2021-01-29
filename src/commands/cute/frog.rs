use rand::Rng;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::{model::channel::Message, prelude::Context};

#[command]
#[aliases("frogge")]
#[usage("")]
#[example("")]
#[description = "Send lil frogge picture stolen from http://allaboutfrogs.org/"]
pub async fn frog(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let data = ctx.data.read().await;

    crate::repeat_message!(ctx, msg, {
        let no = {
            // we want to free the lock as soon as possible
            let mut rng = data.get::<crate::Random>().unwrap().lock().await;
            rng.gen_range(1..=54)
        };

        format!(
            "http://www.allaboutfrogs.org/funstuff/random/00{:02}.jpg",
            no
        )
    });

    Ok(())
}
