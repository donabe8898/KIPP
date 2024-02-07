//! 表示関係の実装

use poise::serenity_prelude::model::user;
// use poise::serenity_prelude::model::channel;
use poise::serenity_prelude::{
    self as serenity, http, ChannelId, CreateEmbed, CreateEmbedFooter, CreateMessage, Embed,
    EmbedAuthor, Error, UserId,
};
use poise::CreateReply;

use serenity::model::Timestamp;
use serenity::prelude::*;
use std::any::Any;
use std::os::unix::thread;
use std::sync::{Arc, OnceLock};
use tokio::*;
use uuid::{self, Uuid};

use crate::imp;

// pub type Error = Box<dyn std::error::Error + Send + Sync>;

/// 返信に使うコンテキスト
pub type Context<'a> = poise::Context<'a, super::Data, Error>;
pub struct Data {}

/*
* - TODO: showall 全てのチャンネルで何件のタスクが登録されているか表示するコマンド
* - TODO: show チャンネルに紐付けられたタスクの表示
*
*/

/// 全タスクの状況をチャンネルごとに一覧形式で表示します。
#[poise::command(slash_command)]
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
        let tast_name: String = row.get("task_name");
        let users: String = row.get("member");

        response += &format!("id: {:?}, task_name: {}, users: {}\n", id, tast_name, users);
    }

    // println!("{}", response);
    let _ = ctx.reply(response).await;
    Ok(())
}

/// チャンネルに属すタスクを表示
#[poise::command(slash_command)]
pub async fn showtask(ctx: Context<'_>) -> Result<(), serenity::Error> {
    // コマンドを実行したチャンネルID
    let this_channel_id = ctx.channel_id();

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
    // クエリ
    let q: String = format!("select * from \"{}\"", this_channel_id);

    let rows = client.query(&q, &[]).await;
    match rows {
        Ok(rows) => {
            /* Embed作成 */

            let mut task_embeds = Vec::new();
            for row in rows {
                // まずはrowから情報を抜き出す
                let task_id = row.get::<&str, uuid::Uuid>("id").to_string();
                let task_name: String = row.get("task_name");
                let member: String = row.get("member");
                let status: i16 = row.get("status");
                let (status, color) = match status {
                    0 => ("完了済み", (0, 0, 0)),
                    1 => ("進行中", (0, 255, 0)),
                    _ => ("その他", (255, 0, 0)),
                };

                // UserIDに変換
                let member_int = member.parse::<u64>().unwrap();
                let usr_id: UserId = UserId::new(member_int);

                // UserIdからUserNameを探す
                let usr_name = usr_id.to_user(ctx).await;

                let usr_name = match usr_name {
                    Ok(usr) => usr.to_string(),
                    Err(_) => "不明なユーザー".to_string(),
                };
                // let usr_name = usr_name.to_string();

                let embed = CreateEmbed::default()
                    .title(task_name)
                    .description(format!("タスクID: {}", task_id))
                    .color(color)
                    .fields(vec![
                        ("担当者", usr_name, false),
                        ("ステータス", status.to_string(), false),
                    ])
                    .footer(CreateEmbedFooter::new("コマンド"))
                    .timestamp(Timestamp::now());

                task_embeds.push(embed);
            }
            let mut rep_builder = CreateReply::default();
            rep_builder.embeds = task_embeds;
            let _ = ctx.send(rep_builder).await;
        }
        Err(e) => {
            let _ = ctx.say("タスクはありません\u{2615}").await;
        }
    };

    // if rows.is_empty() {
    //     let _ = ctx.say("タスクはありません\u{2615}").await;
    // } else {
    // }

    Ok(())
}
