/*
データベース関係の実装
エラーがserenityと別なので注意する.

Copyright © 2024 donabe8898. All rights reserved.
*/

use poise::serenity_prelude::{self as serenity, client, Channel, ChannelId, FutureExt};
use serde::{Deserialize, Serialize};
use tokio;
use tokio_postgres::{
    tls::{NoTlsStream, TlsConnect},
    Client, Connection, Error, NoTls, Row, Socket,
};

use super::*;
type Context<'a> = poise::Context<'a, super::Data, serenity::Error>;

/* NOTE: エラー型はすべてserenity::Errorへ統一してしまう. */

// DB test command
#[poise::command(slash_command)]
pub async fn test(ctx: Context<'_>) -> Result<(), serenity::Error> {
    /* 返答用string */
    let mut response = String::new();

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
    // 表示とdiscord返信
    for row in rows {
        let id: i32 = row.get(0);
        let name: &str = row.get(1);
        response += &format!("id: {}, name: {}\n", id, name);
    }
    let _ = ctx.say(response).await;
    Ok(())
}

/*
チャンネル内のフォーラムを全取得
フォーラムidは15らしい
*/

/* データベース接続 */
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
