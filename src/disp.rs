use poise::serenity_prelude::model::channel;
use poise::serenity_prelude::{
    self as serenity, ChannelId, CreateEmbed, Embed, EmbedAuthor, Error, ForumTagId, FutureExt,
    Guild,
};
use poise::CreateReply;

use serenity::prelude::*;
use std::any::Any;
use std::os::unix::thread;
use std::sync::{Arc, OnceLock};
use tokio::*;
use uuid::{self, Uuid};

use crate::imp;

// pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, super::Data, Error>;
pub struct Data {}

/*
* - TODO: showall 全てのチャンネルで何件のタスクが登録されているか表示するコマンド
* - TODO: show チャンネルに紐付けられたタスクの表示
*
*/

#[poise::command(slash_command)]
// showall 全てのチャンネルで何件のタスクが登録されているか表示するコマンド
pub async fn showall(ctx: Context<'_>) -> Result<(), Error> {
    // コマンドを実行したチャンネルID
    let this_channel_id = ctx.channel_id().to_string();

    /*
    共通処理
    DBへの接続を試行
    tokio_postgres::Errorをserenity::Errorで返すことでエラー処理の簡略化と統一化を図る
    */
    let (client, conn) = match imp::db_conn().await {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Connected error: {}", e);
            return Err(serenity::Error::Other("Database connection error".into()));
        }
    };

    /*
    共通処理
    接続タスク実行
     */
    tokio::spawn(async move {
        if let Err(e) = conn.await {
            eprintln!("connection err: {}", e);
        }
    });

    /* テーブル取得 */
    let q: String = format!("select * from \"{}\"", this_channel_id);
    // let q = format!("select * from testdb");
    let rows = match client.query(&q, &[]).await {
        Ok(result) => result,
        Err(e) => {
            eprintln!("query error: {}", e);
            return Err(serenity::Error::Other("Query error".into()));
        }
    };

    /* row分解して送信形式にまとめる */
    let mut response = String::new();
    for row in rows {
        let id: String = row.get::<&str, uuid::Uuid>("id").to_string();
        let tast_name: String = row.get("tast_name");
        let users: String = row.get("users");

        response += &format!("id: {:?}, tast_name: {}, users: {}\n", id, tast_name, users);
    }

    // println!("{}", response);
    let _ = ctx.reply(response).await;
    Ok(())
}

#[poise::command(slash_command)]
// Embedのテスト
pub async fn embedtest(ctx: Context<'_>) -> Result<(), serenity::Error> {
    // コマンドを実行したチャンネルID
    let this_channel_id = ctx.channel_id().to_string();

    /*
    共通処理
    DBへの接続を試行
    tokio_postgres::Errorをserenity::Errorで返すことでエラー処理の簡略化と統一化を図る
    */
    let (client, conn) = match imp::db_conn().await {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Connected error: {}", e);
            return Err(serenity::Error::Other("Database connection error".into()));
        }
    };

    /*
    共通処理
    接続タスク実行
     */
    tokio::spawn(async move {
        if let Err(e) = conn.await {
            eprintln!("connection err: {}", e);
        }
    });

    /* テーブル取得 */
    let q: String = format!("select * from \"{}\"", this_channel_id);
    // let q = format!("select * from testdb");
    let rows = match client.query(&q, &[]).await {
        Ok(result) => result,
        Err(e) => {
            eprintln!("query error: {}", e);
            return Err(serenity::Error::Other("Query error".into()));
        }
    };

    /* Embed作成 */

    let mut embeds = Vec::new();

    for row in rows {
        // まずはrowから情報を抜き出す
        let task_id = row.get::<&str, uuid::Uuid>("id").to_string();
        let task_name: String = row.get("tast_name");
        let users: String = row.get("users");

        let embed = CreateEmbed::default()
            .title(task_id)
            .description("Embed Description")
            .color((0, 255, 0))
            .field(task_name, users, false);

        embeds.push(embed);
    }
    for embed in embeds {
        let rep = CreateReply::default().content("").embed(embed);
        ctx.send(rep).await?;
    }

    // /* row分解して送信形式にまとめる */
    // let mut response = String::new();
    // for row in rows {
    //     let id: String = row.get::<&str, uuid::Uuid>("id").to_string();
    //     let tast_name: String = row.get("tast_name");
    //     let users: String = row.get("users");

    //     response += &format!("id: {:?}, tast_name: {}, users: {}\n", id, tast_name, users);
    // }

    Ok(())
}
