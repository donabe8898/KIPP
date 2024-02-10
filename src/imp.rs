//! タスクの追加を行う実装
//! エラーがserenityと別なので注意する.
//! エラーはすべてserenityのものへ統一化

// Copyright © 2024 donabe8898. All rights reserved.

use chrono::NaiveDate;

use poise::serenity_prelude::*;
use poise::*;

use std::time::Duration; // タイムアウト処理用

use tokio;

use tokio_postgres::{tls::NoTlsStream, Client, Connection, Error, Socket};

use super::*;
use crate::auth::auth;
type Context<'a> = poise::Context<'a, super::Data, serenity::Error>;

// ============== add task command: チャンネルにタスクを追加する ==============
// - task_name: タスクの名前を入力
// - member: タスクの担当者を選択
//
// タスクを作成してチャンネルに紐付けする。
// 新規作成されたタスクは自動的にUUIDが割り当てられる
// タスクは追加時はすべて進行中のステータスになる
//
// =============================================================================
/// タスクを1件追加します
#[poise::command(slash_command)]
pub async fn add(
    ctx: Context<'_>,
    #[description = "タスク名"] task_name: String,
    #[description = "タスクの概要"] description: Option<String>,
    #[description = "担当者"] member: Option<serenity::Member>,
    #[description = "〆切日"] deadline: Option<String>,
) -> Result<(), serenity::Error> {
    // ---------- サーバー認証 ----------
    if let Some(guild_id) = ctx.guild_id() {
        let _ = auth(guild_id);
    } else {
        let _ = ctx
            .send(
                CreateReply::default()
                    .ephemeral(true)
                    .content("ギルド内で実行されませんでした"),
            )
            .await;
    }
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

    // ---------- タスクの名前 ----------
    let tsk_name: String = task_name;
    // ---------- タスクの概要 ----------
    let description: Option<String> = if let Some(d) = description {
        d.into()
    } else {
        None
    };

    // ---------- タスクの担当者 ----------
    let member_id: Option<String> = if let Some(m) = member {
        format!("{}", m.user.id).into()
    } else {
        None
    };
    // ---------- タスクの〆切 ----------
    // 〆切を設定している？
    let dline: Option<NaiveDate> = if let Some(dl) = deadline {
        // フォーマットが正しい？
        let naive_date: Option<NaiveDate> = match chrono::NaiveDate::parse_from_str(&dl, "%Y-%m-%d")
        {
            Ok(date) => Some(date),
            Err(_) => None,
        };
        naive_date
    } else {
        None
    };
    // ---------- レコード作成用クエリ文 ----------
    // dlineは'None'
    // $1などはvaluesにしか使えないらしい
    let insert = format!(
        "insert into \"{}\" values (uuid_generate_v4(), $1, $2, $3, $4, 1);",
        channel_id
    );

    // ---------- テーブル作成用クエリ文 ----------
    let create = format!(
        "create table \"{}\" (\
            id uuid DEFAULT uuid_generate_v4(), \
            task_name text NOT NULL, \
            description text,\
            member text, \
            deadline date, \
            status smallint DEFAULT 1);",
        channel_id
    );
    println!("{}", create);
    // ---------- クエリ送信 ----------
    let _res = match client
        .query(&insert, &[&tsk_name, &description, &member_id, &dline])
        .await
    {
        Ok(_result) => {}
        Err(_e) => {
            let _e_res = match client.query(&create, &[]).await {
                Ok(_result) => {
                    let res = client
                        .query(&insert, &[&tsk_name, &description, &member_id, &dline])
                        .await;
                    if let Ok(_) = res {
                    } else {
                        return Err(serenity::Error::Other("タスクの登録に失敗しました".into()));
                    }
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

// ============== remove task command: タスクを削除する ==============
// - task_id: 削除したいタスクのUUID
//
// チャンネル内のタスクを１つ消します
// 削除したいタスクのUUIDを引数に取る必要があります
//
// ================================================================

/// タスクをチャンネルから削除します
#[poise::command(slash_command)]
pub async fn remove(
    ctx: Context<'_>,
    #[description = "タスクID"] task_id: String,
) -> Result<(), serenity::Error> {
    // ---------- サーバー認証 ----------
    if let Some(guild_id) = ctx.guild_id() {
        let _ = auth(guild_id);
    } else {
        let _ = ctx
            .send(
                CreateReply::default()
                    .ephemeral(true)
                    .content("ギルド内で実行されませんでした"),
            )
            .await;
    }
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
        .label("はい")
        .style(ButtonStyle::Secondary);

    // ---------- Noボタン ----------
    let btn_no = CreateButton::new("no")
        .label("いいえ")
        .style(ButtonStyle::Success);

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

// ============== status task command: タスクを編集する ==============
// - task_id: ステータスの変更をしたいタスクのID
//
// チャンネル内のタスクのステータスを変更します
// タスクのUUIDを引数に取る必要があります
//
// ================================================================

/// タスクのステータスを変更します
#[poise::command(slash_command)]
pub async fn status(
    ctx: Context<'_>,
    #[description = "タスクID"] task_id: String,
) -> Result<(), serenity::Error> {
    // ---------- サーバー認証 ----------
    if let Some(guild_id) = ctx.guild_id() {
        let _ = auth(guild_id);
    } else {
        let _ = ctx
            .send(
                CreateReply::default()
                    .ephemeral(true)
                    .content("ギルド内で実行されませんでした"),
            )
            .await;
    }
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

    // ---------- ステータスリスト作成 ----------
    // (emoji->今のとこなし, lavel, value)
    // ---------- まずは項目の作成 ----------
    let select_options = vec![
        CreateSelectMenuOption::new("進行中", "1"),
        CreateSelectMenuOption::new("完了済み", "0"),
    ];

    // ---------- セレクトメニュー作成 ----------
    let kind = CreateSelectMenuKind::String {
        options: select_options,
    };
    let menu =
        CreateSelectMenu::new("menu", kind).placeholder("タスクのステータスを選択してください");

    // ---------- アクションにメニュー追加 ----------
    let action = CreateActionRow::SelectMenu(menu);

    // ---------- メッセージにメニューを乗せる ----------
    let rep = CreateMessage::new().components(vec![action]);
    let _ = ctx.say("ステータスの変更を行います").await;

    // ---------- イベントハンドラを受け取る ----------
    let h = channel_id.send_message(ctx, rep).await;
    let handle = match h {
        Ok(result) => result,
        Err(_) => panic!("Send Error"),
    };

    // ---------- タイムアウト設定 ----------
    let mi = match handle
        .await_component_interaction(&ctx)
        .timeout(Duration::from_secs(60))
        .await
    {
        Some(interaction) => interaction,
        None => {
            let _ = handle.delete(ctx).await;
            return Err(serenity::Error::Other("タイムアウトしました".into()));
        }
    };
    let _ = handle.delete(ctx).await;

    // ---------- kindのシリアライズ ----------
    let serialized_kind: serde_json::Value = serde_json::to_value(&mi.data.kind)?;

    // ---------- シリアライズからvalueを取得 ----------
    let status_code;
    if let Some(value) = serialized_kind.get("values")
    // .and_then(serde_json::Value::as_str)
    {
        status_code = value.get(0).and_then(serde_json::Value::as_str).unwrap();
    } else {
        status_code = "not_found";
    }

    // ---------- DBへステータスを反映 ----------
    let status = status_code;
    match status {
        "not_found" => {
            let _ = ctx.reply("エラー発生").await;
        }
        _ => {
            // ---------- 反映クエリ ----------
            let status_change_query = format!(
                "update \"{}\" set status=\'{}\' where id=\'{}\'",
                channel_id.to_string(),
                status,
                task_id
            );
            // ---------- 反映依頼 ----------
            let result = client.query(&status_change_query, &[]).await;
            match result {
                Ok(_) => {
                    let _ = ctx
                        .send(
                            CreateReply::default()
                                .ephemeral(true)
                                .content("ステータスを変更しました"),
                        )
                        .await;
                }
                Err(_) => {
                    let _ = ctx
                        .send(
                            CreateReply::default()
                                .ephemeral(true)
                                .content("ステータスを変更できませんでした"),
                        )
                        .await;
                }
            };
        }
    }
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
