use poise::serenity_prelude::{self as serenity, GuildId};
use std::env;

// guild_id: ctxのguildID
pub fn auth(guild_id: GuildId) -> Result<(), serenity::Error> {
    // .envからギルドIDとってくる
    let auth_guild = env::var("GUILD_ID").expect("missing get token");
    let auth_guild = GuildId::new(auth_guild.parse::<u64>().unwrap());

    if guild_id == auth_guild {
    } else {
        return Err(serenity::Error::Other(
            "This is an unauthorized guild.".into(),
        ));
    }

    Ok(())
}
