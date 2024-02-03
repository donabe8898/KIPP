//! タスクの追加を行う実装
//! エラーがserenityと別なので注意する.
//! エラーはすべてserenityのものへ統一化

// Copyright © 2024 donabe8898. All rights reserved.

use poise::serenity_prelude::*;
use poise::serenity_prelude::{self as serenity, client, Channel, ChannelId, FutureExt, Mention};
use serde::{Deserialize, Serialize};
use tokio;
use tokio_postgres::{
    tls::{NoTlsStream, TlsConnect},
    Client, Connection, Error, NoTls, Row, Socket,
};

use super::*;
type Context<'a> = poise::Context<'a, super::Data, serenity::Error>;

/// DB test command
#[poise::command(slash_command)]
pub async fn test(ctx: Context<'_>) -> Result<(), serenity::Error> {
    //! DB test command
    /* 返答用string */
    let mut response = String::new();

    /* コマンドを実行したチャンネルのIDを取得 */
    let channel_id: String = ctx.channel_id().to_string();

    /*
    DBへの接続を試行

    tokio_postgres::Errorをserenity::Errorで返すことでエラー処理の簡略化と統一化を図る
    */
    let (client, conn) = match db_conn().await {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Connected error: {}", e);
            return Err(serenity::Error::Other("Database connection error".into()));
        }
    };

    /* 接続タスク実行 */
    tokio::spawn(async move {
        if let Err(e) = conn.await {
            eprintln!("connection err: {}", e);
        }
    });

    /* DBテーブル取得 */

    let rows = match client.query("select * from testdb", &[]).await {
        Ok(result) => result,
        Err(e) => {
            eprintln!("query error: {}", e);
            return Err(serenity::Error::Other("Query error".into()));
        }
    };
    /* 表示とdiscord返信 */
    for row in rows {
        let id: i32 = row.get(0);
        let name: &str = row.get(1);
        response += &format!("id: {}, name: {}\n", id, name);
    }
    let _ = ctx.say(response).await;
    Ok(())
}

/// タスクを1件追加します
#[poise::command(slash_command)]
pub async fn addtask(
    ctx: Context<'_>,
    #[description = "タスク名"] task_name: String,
    #[description = "担当者"] member: serenity::Member,
) -> Result<(), serenity::Error> {
    /* コマンドを実行したチャンネルのIDを取得 */
    let channel_id: String = ctx.channel_id().to_string();

    /*
    DBへの接続を試行

    tokio_postgres::Errorをserenity::Errorで返すことでエラー処理の簡略化と統一化を図る
    */
    let (client, conn) = match db_conn().await {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Connected error: {}", e);
            return Err(serenity::Error::Other("Database connection error".into()));
        }
    };

    /* 接続タスク実行 */
    tokio::spawn(async move {
        if let Err(e) = conn.await {
            eprintln!("connection err: {}", e);
        }
    });

    /*
    タスク登録
    テーブルが無ければ作成
    */
    let tsk_name: String = task_name;
    let member_id: UserId = member.user.id;
    let insert = format!(
        "insert into \"{}\" (id, task_name, member) values (uuid_generate_v4(), \'{}\', \'{}\');",
        channel_id, tsk_name, member_id
    );
    let create = format!("create table \"{}\" (id uuid DEFAULT uuid_generate_v4(), task_name text NOT NULL, member text NOT NULL);",channel_id);
    println!("{}\n{}", insert, create);

    let _res = match client.query(&insert, &[]).await {
        Ok(_result) => {}
        Err(_e) => {
            let _e_res = match client.query(&create, &[]).await {
                Ok(_result) => {
                    let _ = client.query(&insert, &[]).await;
                }
                Err(_e) => {
                    return Err(serenity::Error::Other("タスクの登録に失敗しました".into()));
                }
            };
        }
    };

    /* 完了メッセージ */

    let _ = ctx.reply("タスクを登録しました").await;
    Ok(())
}

/// データベースへの接続処理
pub async fn db_conn() -> Result<(Client, Connection<Socket, NoTlsStream>), Error> {
    let (client, conn) = tokio_postgres::Config::new()
        .user("postgres")
        .password("password")
        .host("localhost")
        .port(5432)
        .dbname("postgres")
        .connect(tokio_postgres::NoTls)
        .await?;

    Ok((client, conn))
}
