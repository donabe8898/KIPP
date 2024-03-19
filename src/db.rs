//! postgres関係の実装

use poise::serenity_prelude::*;

// タイムアウト処理用
use super::*;
use tokio;
use tokio_postgres::{Client, Error};

/// データベースへの接続を確立する処理
///
///
/// 一定時間立つと接続は解除されるため、切断処理は実装しなくてもOK
///
pub async fn db_conn() -> Result<Client, Error> {
    let (client, conn) = tokio_postgres::Config::new()
        .user("postgres")
        .password("password")
        .host("localhost")
        .port(5432)
        .dbname("postgres")
        .connect(tokio_postgres::NoTls)
        .await?;

    tokio::spawn(async move {
        if let Err(e) = conn.await {
            eprintln!("connection error: {}", e);
            eprintln!("コネクションエラー: {}", e);
        }
    });
    Ok(client)
}

/// Postgresへ接続する
pub async fn connect_to_db() -> Result<Client, serenity::Error> {
    match db_conn().await {
        Ok(result) => Ok(result),
        Err(e) => {
            eprintln!("Connected error: {}", e);
            return Err(serenity::Error::Other("Database connection error".into()));
        }
    }
}
