//! 認証系の実装

use poise::serenity_prelude::{self as serenity, GuildId};
use poise::CreateReply;
use std::env;

pub type Context<'a> = poise::Context<'a, super::Data, serenity::Error>;

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
///

pub async fn auth(ctx: Context<'_>) -> Result<(), serenity::Error> {
    // ctxからguildid取得
    let guild_id = ctx.guild_id().unwrap();

    // .envからギルドIDとってくる
    let env_guild = env::var("GUILD_ID").expect("missing get token");
    let env_guild = GuildId::new(env_guild.parse::<u64>().unwrap());

    // ギルドが違っていた場合
    if guild_id != env_guild {
        let _ = ctx
            .send(
                CreateReply::default()
                    .ephemeral(true)
                    .content("⚠ このサーバーでは実行できません"),
            )
            .await;
        return Err(serenity::Error::Other(
            "This is an unauthorized guild.".into(),
        ));
    }

    Ok(())
}

/// password認証が必要なコマンドの実装
///
/// # 引数
/// * `password` - 入力されたパスワード
///
/// パスワードを要求するスラッシュコマンドに利用する
///
/// 入力された引数と照らしあわせる文字列は`.env`ファイルの`PASSWORD=`に記述
pub async fn passwd(ctx: Context<'_>, password: String) -> Result<(), serenity::Error> {
    let check_pass = env::var("PASSWORD").expect("missing get token");
    if password != check_pass {
        return Err(serenity::Error::Other("failed password".into()));
    }
    Ok(())
}
