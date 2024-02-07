//! タスクの追加を行う実装
//! エラーがserenityと別なので注意する.
//! エラーはすべてserenityのものへ統一化

// Copyright © 2024 donabe8898. All rights reserved.

use futures_util::TryFutureExt;
use poise::reply::ReplyHandle;
use poise::serenity_prelude::*;
use poise::*;

use serde::{de::IntoDeserializer, Deserialize, Serialize};
use std::time::Duration; // タイムアウト処理用
use std::{any::Any, thread::sleep};
use tokio;
use tokio::time::timeout;
use tokio_postgres::{
    tls::{NoTlsStream, TlsConnect},
    Client, Connection, Error, NoTls, Row, Socket,
};

use super::*;
type Context<'a> = poise::Context<'a, super::Data, serenity::Error>;

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
            eprintln!("connection error: {}", e);
            eprintln!("コネクションエラー: {}", e);
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

// TODO: タスクの追加
/// タスクを1件追加します
#[poise::command(slash_command)]
pub async fn addtask(
    ctx: Context<'_>,
    #[description = "タスク名"] task_name: String,
    #[description = "担当者"] member: serenity::Member,
) -> Result<(), serenity::Error> {
    /* コマンドを実行したチャンネルのIDを取得 */
    let channel_id = ctx.channel_id();

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

    NOTE: ステータス
    - 進行中 = 1
    - 終了 = 0
    */

    let tsk_name: String = task_name;
    let member_id: UserId = member.user.id;

    // レコード作成用クエリ文
    let insert = format!(
        "insert into \"{}\" (id, task_name, member, status) values (uuid_generate_v4(), \'{}\', \'{}\', 1);",
        channel_id, tsk_name, member_id
    );

    // テーブル作成用クエリ文
    let create = format!("create table \"{}\" (id uuid DEFAULT uuid_generate_v4(), task_name text NOT NULL, member text NOT NULL, status smallint DEFAULT 1);",channel_id);

    // クエリ送信
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

// TODO: タスクの完了

/// タスクをチャンネルから削除します
#[poise::command(slash_command)]
pub async fn removetask(
    ctx: Context<'_>,
    #[description = "タスクID"] task_id: String,
) -> Result<(), serenity::Error> {
    /* コマンドを実行したチャンネルのIDを取得 */
    let channel_id = ctx.channel_id();
    // ---------- DB処理 ----------
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
            eprintln!("connection error: {}", e);
            eprintln!("コネクションエラー: {}", e);
        }
    });

    // ---------- DB処理おわり ----------

    // ---------- Yesボタン ----------
    let btn_yes = CreateButton::new("yes")
        .emoji('\u{1f697}')
        .label("はい")
        .style(ButtonStyle::Success);

    // ---------- Noボタン ----------
    let btn_no = CreateButton::new("no")
        .emoji('\u{2615}')
        .label("いいえ")
        .style(ButtonStyle::Secondary);

    // ---------- アクションにボタンを追加 ----------
    let buttons = CreateActionRow::Buttons(vec![btn_yes, btn_no]);

    let rep2 = CreateMessage::default().components(vec![buttons]);

    let _ = ctx.say("本当に削除しますか？").await;

    let h = channel_id.send_message(ctx, rep2).await;

    let handle = match h {
        Ok(result) => result,
        Err(_) => panic!("送信失敗"),
    };

    // ---------- タイムアウトの秒数を指定 ----------
    let mi = match handle
        .await_component_interaction(&ctx)
        .timeout(Duration::from_secs(20))
        .await
    {
        Some(interaction) => interaction,
        None => {
            let _ = handle.delete(ctx).await;

            return Err(serenity::Error::Other("タイムアウトしました。".into()));
        }
    };

    let _ = handle.delete(ctx).await;

    let id: &str = &mi.data.custom_id;
    let _ = match id {
        "yes" => {
            // ========== 削除処理 ==========

            // 削除クエリ
            let remove_query = format!(
                "delete from \"{}\" where id=\'{}\';",
                channel_id.to_string(),
                task_id
            );
            // DBテーブルまるごと削除する際のクエリ（タスクが全部無くなったとき）
            let remove_table_query = format!("drop table \"{}\";", channel_id.to_string());
            // テーブルの行数を数えるクエリ
            let count_row_query = format!("select count(*) from \"{}\"", channel_id.to_string());

            // ========== 削除依頼 ==========
            let _res = match client.query(&remove_query, &[]).await {
                Ok(_result) => {
                    // ========== メッセージ送信でユーザーにお知らせ ==========
                    let _ = channel_id
                        .send_message(ctx, CreateMessage::default().content("削除しました"))
                        .await
                        .map(|_| ());

                    // ========== 行数カウント ==========
                    let count_row = client.query(&count_row_query, &[]).await.unwrap();
                    let count: i64 = count_row[0].get("count");

                    // ========== 0行だったらテーブルごと削除 =========
                    if count == 0i64 {
                        let _ = client.query(&remove_table_query, &[]).await;
                        let _ = channel_id
                            .send_message(
                                ctx,
                                CreateMessage::default()
                                    .content("チャンネル内タスクが全て無くなりました。"),
                            )
                            .await
                            .map(|_| ());
                    }
                }

                Err(_e) => {
                    return Err(serenity::Error::Other("削除できませんでした".into()));
                }
            };
        }
        "no" => {
            // ========== メッセージ送信でユーザーにお知らせ ==========
            let _ = channel_id
                .send_message(ctx, CreateMessage::default().content("中止しました"))
                .map(|_| ())
                .await;
        }
        _ => {
            panic!("エラー");
        }
    };

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
/* 通常メッセージの送信
let _ = channel_id.send_message(ctx,CreateMessage::default()
    .content("チャンネル内タスクが全て無くなりました。"),).await.map(|_| ());
*/
