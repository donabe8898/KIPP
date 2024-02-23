//! 表示関係の実装

use poise::serenity_prelude::{
    self as serenity, ChannelId, CreateEmbed, CreateEmbedFooter, Error, UserId,
};
use poise::CreateReply;
use serenity::model::Timestamp;
use uuid::{self};

use crate::auth::auth;
use crate::imp;


/// 返信に使うコンテキスト
pub type Context<'a> = poise::Context<'a, super::Data, Error>;



/// チャンネルごとにタスクの数を一覧形式で表示します。
///
///
/// ユーザーを選択すると、そのユーザーが担当しているタスクの表示を行う。
/// 選択されなかったら普通にすべてのタスクを表示
///
/// # 引数
///
/// * `ctx` - コマンド起動時の情報が入ったブツ
/// * `user` - ユーザーを限定して出力させる場合は入力
/// * `display` - 自分だけのメッセージとして表示させる場合は`true`
///
pub async fn showall(
    ctx: Context<'_>,
    user: Option<serenity::User>,
    display: Option<bool>,
) -> Result<(), Error> {
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

    // ---------- コマンドを実行したチャンネルID ----------
    let _this_channel_id = ctx.channel_id().to_string();

    /*
    ---------- 共通処理 ----------
    DBへの接続を試行
    tokio_postgres::Errorをserenity::Errorで返すことでエラー処理の簡略化と統一化を図る
    */
    let (client, conn) = match imp::db_conn().await {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Connected error: {}", e);
            return Err(Error::Other("Database connection error".into()));
        }
    };

    /*
     ---------- 共通処理 ----------
    接続タスク実行
     */
    tokio::spawn(async move {
        if let Err(e) = conn.await {
            eprintln!("connection err: {}", e);
        }
    });

    // ---------- ギルド内のテキストチャンネル及びフォーラムチャンネルの取得 ----------
    // DB内のすべてのテーブル名を取得 "{}"はあとで除く
    let all_tables_query = "select tablename from pg_tables
    where schemaname not in('pg_catalog','information_schema')
    order by tablename;"
        .to_string();

    // クエリ投げ
    let tables = client.query(&all_tables_query, &[]).await;

    match tables {
        // ---------- テーブルが帰ってきた場合 ----------
        Ok(tables) => {
            // ========= ユーザー選択あり =========
            match user {
                Some(usr) => {
                    let usr_id = usr.id.to_string();

                    // 返信用
                    let mut rep_string: String = String::new();

                    for table in tables {
                        // チャンネルID
                        let channel_id: String = table.get("tablename");
                        // {}が帰ってきたらとばす
                        if &channel_id == "{}" {
                            continue;
                        }
                        // 検索クエリ
                        let cnt_query = format!(
                            "select count(*) from \"{}\" where member=\'{}\'",
                            channel_id, usr_id
                        );
                        // クエリ送信
                        let count = client.query(&cnt_query, &[]).await.unwrap();
                        // チャンネル内のタスクを数える
                        let count: i64 = count[0].get("count");
                        // --------- 返信

                        let channel_id = ChannelId::new(channel_id.parse::<u64>().unwrap());
                        match channel_id.to_channel(ctx.http()).await {
                            Ok(ch) => {
                                let s = format!("| {} | : {} 件\n", ch, count);
                                rep_string.push_str(&s);
                            }
                            Err(_) => {
                                let s = format!("| 不明なチャンネル | {}件\n", count);
                                rep_string.push_str(&s);
                            }
                        };
                    }
                    // ---------- 返信を見せるかどうか ----------
                    // 原則は自分のみ表示
                    let is_disp = if let Some(b) = display { !b } else { true };
                    // ---------- リプライビルダー作成 ----------
                    let rep = CreateReply::default()
                        .content(rep_string)
                        .ephemeral(is_disp);
                    let _ = ctx.send(rep).await;
                }
                // ========= ユーザー選択なし =========
                None => {
                    // 返信用
                    let mut rep_string: String = String::new();
                    for table in tables {
                        // チャンネルID
                        let channel_id: String = table.get("tablename");

                        // {}が帰ってきたらとばす
                        if &channel_id == "{}" {
                            continue;
                        }
                        // 検索クエリ
                        let cnt_query = format!("select count(*) from \"{}\";", channel_id);
                        // クエリ送信
                        let count = client.query(&cnt_query, &[]).await.unwrap();
                        // チャンネル内のタスクを数える
                        let count: i64 = count[0].get("count");
                        // TODO: 返信
                        let channel_id = ChannelId::new(channel_id.parse::<u64>().unwrap());
                        match channel_id.to_channel(ctx.http()).await {
                            Ok(ch) => {
                                let s = format!("| {} | : {} 件\n", ch, count);
                                rep_string.push_str(&s);
                            }
                            Err(_) => {
                                let s = format!("| 不明なチャンネル | {}件\n", count);
                                rep_string.push_str(&s);
                            }
                        };
                    }
                    // ---------- 返信を見せるかどうか ----------
                    let is_disp = if let Some(b) = display { !b } else { true };
                    // ---------- リプライビルダー作成 ----------
                    let rep = CreateReply::default()
                        .content(rep_string)
                        .ephemeral(is_disp);
                    let _ = ctx.send(rep).await;
                }
            };
        }
        // ---------- テーブルが帰ってこなかった場合（多分無い） ----------
        Err(_) => {
            return Err(Error::Other("Cannot find tasks.!".into()));
        }
    }

    Ok(())
}



/// チャンネルに属すタスクを表示
///
///
/// ユーザーを選択すると、そのユーザーが担当しているタスクの表示を行う。
/// 選択されなかったら普通にすべてのタスクを表示
///
/// # 引数
///
/// * `ctx` - コマンド起動時の情報が入ったブツ
/// * `user` - ユーザーを限定して出力させる場合は入力
/// * `display` - 自分だけのメッセージとして表示させる場合は`true`

pub async fn show(
    ctx: Context<'_>,
    user: Option<serenity::User>,
    display: Option<bool>,
) -> Result<(), Error> {
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
            return Err(Error::Other("Database connection error".into()));
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
    let q: String;
    match user {
        // ---------- ユーザー選択あり->指定ユーザーのタスク ----------
        Some(usr) => {
            let usr_id = usr.id.to_string();
            q = format!(
                "select * from \"{}\" where member=\'{}\'",
                this_channel_id, usr_id
            );
        }
        // ---------- ユーザー選択なし->全ユーザーのタスク ----------
        None => {
            q = format!("select * from \"{}\"", this_channel_id);
        }
    }

    // ---------- 返信を見せるかどうか ----------
    // 原則は自分のみ表示
    let is_disp = if let Some(b) = display { !b } else { true };

    let rows = client.query(&q, &[]).await;
    match rows {
        Ok(rows) => {
            /* Embed作成 */

            // rows<vec>の中身が空でない場合
            if !rows.is_empty() {
                let mut task_embeds = Vec::new();
                for row in rows {
                    // ---------- まずはrowから情報を抜き出す ----------
                    // タスクID
                    let task_id = row.get::<&str, uuid::Uuid>("id").to_string();
                    // タスク名
                    let task_name: String = row.get("task_name");
                    // 概要
                    let description: Option<String> = row.get("description");
                    // 担当者
                    let member: Option<String> = row.get("member");
                    // 〆切日
                    let deadline: Option<chrono::NaiveDate> = row.get("deadline");
                    // ステータス
                    let status: i16 = row.get("status");
                    let (status, color) = match status {
                        0 => ("完了済み", (0, 0, 0)),
                        1 => ("進行中", (0, 255, 0)),
                        _ => ("その他", (255, 0, 0)),
                    };

                    // ---------- descriptionがNoneなら無にする ----------
                    let con_description = match description {
                        Some(d) => d,
                        None => "説明なし".to_string(),
                    };

                    // UserIDに変換
                    // 最終的にembedへ組み込む
                    let content_user_name;
                    // ---------- memberがNoneかどうか ----------
                    if let Some(m) = member {
                        let member_int = m.parse::<u64>().unwrap();
                        let usr_id = UserId::new(member_int);
                        // UserIdからUserNameを探す
                        let usr_name = usr_id.to_user(ctx).await;

                        let usr_name = match usr_name {
                            Ok(usr) => usr.to_string(),
                            Err(_) => "不明なユーザー".to_string(),
                        };
                        content_user_name = usr_name;
                    } else {
                        content_user_name = "None".to_string();
                    };
                    // ---------- 締切日が設定されているかどうか ----------
                    let dline = if let Some(d) = deadline {
                        d.format("%Y-%m-%d").to_string()
                    } else {
                        "〆切はありません".to_string()
                    };

                    let embed = CreateEmbed::default()
                        .title(task_name)
                        .description(format!("{}", con_description))
                        .color(color)
                        .fields(vec![
                            ("タスクID", task_id, false),
                            ("担当者", content_user_name, true),
                            ("〆切", dline, true),
                            ("ステータス", status.to_string(), true),
                        ])
                        .footer(CreateEmbedFooter::new("コマンド"))
                        .timestamp(Timestamp::now());

                    task_embeds.push(embed);
                }
                let mut rep_builder = CreateReply::default().ephemeral(is_disp);
                rep_builder.embeds = task_embeds;
                let _ = ctx.send(rep_builder).await;
            }
            // rows<vec>の中身が空の場合
            else {
                let rep_builder = CreateReply::default()
                    .ephemeral(is_disp)
                    .content("タスクはありません\u{2615}");
                let _ = ctx.send(rep_builder).await;
            }
        }
        Err(_) => {
            let _ = ctx.reply("タスクはありません\u{2615}").await;
        }
    };


    Ok(())
}
