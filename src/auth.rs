//! 認証系の実装

use poise::serenity_prelude::{self as serenity, GuildId};
use std::env;

/// ギルドIDを比較するメソッド
///
/// # 引数
///
/// * `guild_id` - ギルドID
///
/// スラッシュコマンドを使用した場所のギルドIDが引数に渡される.
/// Botの読み取る.envファイルのギルドIDと異なる場合はエラーを返す.
///
/// これはBotが他のサーバーに招待されDBの中身（タスク）を見られることを防ぐ目的がある.
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
