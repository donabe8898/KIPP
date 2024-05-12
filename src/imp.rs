//! DB内のデータを追加・編集・削除する実装
//!
//! エラーはすべてserenityのものへ統一化
// Copyright © 2024 donabe8898. All rights reserved.

use super::*;
// use crate::auth::auth;
use crate::db::connect_to_db;
// use crate::Context;
use chrono::NaiveDate;
// use poise::serenity_prelude::model::guild;
use poise::serenity_prelude::*;
use poise::*;
use std::time::Duration; // タイムアウト処理用

pub type Context<'a> = poise::Context<'a, super::Data, serenity::Error>;

/// タスクを1件追加します
///
///
/// タスクを作成してチャンネルに紐付けする.
///
///
/// 新規作成されたタスクは自動的にUUIDが割り当てられる.
///
///
/// タスクは追加時はすべて進行中のステータスになる.
///
/// # 引数
///
/// * `ctx` - コマンド起動時の情報が入ったブツ
/// * `task_name` - タスク名（必須）
/// * `description` - タスクの概要や説明があれば入力
/// * `member` - タスクの担当者を決める場合に入力
/// * `deadline` - タスクの期限日を設定する場合は入力
///

pub async fn add(
    ctx: Context<'_>,
    task_name: String,
    description: Option<String>,
    member: Option<serenity::Member>,
    deadline: Option<String>,
) -> Result<(), serenity::Error> {
    /* コマンドを実行したチャンネルのIDを取得 */
    let channel_id = ctx.channel_id();

    // ---------- 共通処理 ----------
    // DBへの接続を試行
    let client = connect_to_db().await.unwrap();

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
    let rep = CreateReply::default()
        .ephemeral(true)
        .content("タスクを登録しました");

    let _ = ctx.send(rep).await;
    Ok(())
}

/// タスクをチャンネルから削除
///
/// Bot側から削除するかどうか聞いてくる. 一定時間内に応答がなければタイムアウトという
/// 形で削除しない選択を取る.
///
/// # 引数
///
/// * `ctx` - コマンド起動時の情報が入ったブツ
/// * `task_id` - タスクのID (UUIDv4)

pub async fn remove(ctx: Context<'_>, task_id: String) -> Result<(), serenity::Error> {
    /* コマンドを実行したチャンネルのIDを取得 */
    let channel_id = ctx.channel_id();
    // ---------- DB処理 ----------

    // ---------- 共通処理 ----------
    // DBへの接続を試行
    let client = connect_to_db().await.unwrap();

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

    // ---------- 削除しますか？？？？ ----------
    let how_del = CreateReply::default()
        .ephemeral(true)
        .content("本当に削除しますか？");
    let _ = ctx.send(how_del).await;

    // ---------- ボタンハンドル送信 ----------
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

/// タスクのステータスを変更します
///
/// ステータスをどれに変更するかのプルダウンメニューが表示される.
/// こちらも一定時間応答がないとタイムアウトする
///
/// # 引数
///
/// * `ctx` - コマンド起動時の情報が入ったブツ
/// * `task_id` - タスクのID (UUIDv4)

pub async fn status(ctx: Context<'_>, task_id: String) -> Result<(), serenity::Error> {
    /* コマンドを実行したチャンネルのIDを取得 */
    let channel_id = ctx.channel_id();

    // DBへの接続を試行
    let client = connect_to_db().await.unwrap();

    // ---------- ステータスリスト作成 ----------
    // (emoji->今のとこなし, lavel, value)
    // ---------- まずは項目の作成 ----------
    // TODO: 未着手1と進行中2を実装
    let select_options = vec![
        CreateSelectMenuOption::new("進行中", "2"),
        CreateSelectMenuOption::new("未着手", "1"),
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

/// チャンネル削除等で残ったテーブルを削除する処理コマンド
///
/// __*注意!*__: 非アクティブのスレッドも削除される為, 定期的にスレッドの活性化をおすすめします.
///
/// # 引数
///
/// * `ctx` - コマンド起動時の情報が入ったブツ
/// * `password` - 管理者用パスワード
pub async fn clean(ctx: Context<'_>, password: String) -> Result<(), serenity::Error> {
    // パスワード確認
    match auth::passwd(ctx, password).await {
        Ok(_) => {}
        Err(_) => {
            let rep = CreateReply::default()
                .content("パスワードが違います")
                .ephemeral(true);
            let _ = ctx.send(rep).await;
            return Err(serenity::Error::Other("削除できませんでした".into()));
        }
    }

    // DBへの接続を試行
    let client = connect_to_db().await.unwrap();

    // ギルドオブジェクト取得
    // WARNING: この辺怪しい
    let guild_id: GuildId = ctx.guild_id().unwrap();

    let http = ctx.clone().http();
    let channels = guild_id.channels(http).await?;
    let threads = guild_id.get_active_threads(http).await?;

    // ギルド内の全チャンネルID取得
    // テキストチャンネルとスレッドのまとめ
    let mut threds: Vec<String> = Vec::new();
    for (key, _) in &channels {
        threds.push(key.to_string());
    }
    for th in &threads.threads {
        threds.push(th.id.to_string());
    }

    // println!("{:#?}\n{:#?}", chs, threds);

    // クエリ
    // DB内のすべてのテーブル名を取得 "{}"はあとで除く
    let all_tables_query = "select tablename from pg_tables
    where schemaname not in('pg_catalog','information_schema')
    order by tablename;"
        .to_string();

    // クエリ投げ
    let tables = client.query(&all_tables_query, &[]).await;

    // クエリ失敗したら削除処理に移行しない
    // クエリのエラー処理と同時にテーブルをVec<String>へ
    let tables = match tables {
        Ok(tables) => {
            let mut tbs: Vec<String> = Vec::new();
            for tb in tables {
                let tb_chid: String = tb.get("tablename");
                // {}が帰ってきたらとばす
                if &tb_chid == "{}" {
                    continue;
                }

                tbs.push(tb_chid);
            }
            // string化したテーブルを返す
            tbs
        }
        Err(_e) => {
            return Err(serenity::Error::Other("削除できませんでした".into()));
        }
    };

    // テーブルIDのチャンネルorスレッドが存在しなければ削除処理
    // NOTE: tables: DB内のチャンネルID
    // NOTE: chs, threds: discord鯖内のチャンネルID
    // 削除カウンター int
    let mut count: u64 = 0;

    // 1. 探索して削除
    for tb in &tables {
        let mut is_find = false;
        for id in &threds {
            if tb == id {
                is_find = true;
                break;
            }
        }
        if !is_find {
            let delete_query = format!("drop table \"{}\";", tb);
            let _ = client.query(&delete_query, &[]).await;
            count += 1;
        }
    }

    // 削除した件数を返信
    if 0 < count {
        let rep = CreateReply::default()
            .content(format!("タスクを{}件削除しました", count))
            .ephemeral(true);
        let _ = ctx.send(rep).await;
    } else {
        let rep = CreateReply::default()
            .content("タスクを削除しませんでした")
            .ephemeral(true);
        let _ = ctx.send(rep).await;
    }

    Ok(())
}
