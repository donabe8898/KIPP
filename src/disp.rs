use std::os::unix::thread;

use poise::serenity_prelude::{self as serenity, ChannelId, Error};

use tokio::*;

// pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, super::Data, Error>;
pub struct Data {}

// フォーラムIDから全スレッドのIDを取得
#[poise::command(slash_command)]
// フォーラムIDから全スレッドのIDを取得
pub async fn getforum(ctx: Context<'_>) -> Result<(), Error> {
    /* 返信 */
    let mut response = String::new();

    /* フォーラム取得 */
    // TODO: Debug
    let forum_channel_id = ChannelId(1201022878880120935);

    /*
    スレッド一覧取得
    discordではスレッドから直接IDを取得できないので, スレッドの最初のメッセージをスレッドIDと見なして処理する必要がある
    */
    let cache_and_http = ctx.serenity_context();

    let threads = forum_channel_id
        .messages(&cache_and_http, |m| m)
        .await
        .map_err(|_| serenity::Error::Other("Failed to fetch messages".into()))?;

    println!("{:?}", threads);

    /* スレッドのIDを取得 */
    for thread_id in threads {
        response.push_str(&format!("Thread ID: {}\n", thread_id.id));
        println!("{}", &thread_id.id);
    }

    /* 返信する */

    // let _ = ctx.say(&response).await;
    // HACK: デバッグ
    let _ = ctx.say("おけ").await;
    Ok(())
}

#[poise::command(slash_command)]
// デバッグ用：チャンネルIDの取得
pub async fn getchannelid(ctx: Context<'_>) -> Result<(), Error> {
    let forum_channel_id = ChannelId(1201022878880120935);

    let resp = ctx.channel_id();
    let _ = ctx.say(format!("{:?}", resp)).await;
    Ok(())
}
