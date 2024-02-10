//! support.rs
//! ========== サポートコマンド実装 ==========
//! - /help: コマンドのヘルプを表示
//! - /version: コマンドのバージョン情報等を表示

use poise::serenity_prelude::*;
use poise::*;

use std::env;

use crate::auth::auth;
use std::fs::File;
use std::io::{BufRead, BufReader};

/// 返信に使うコンテキスト
pub type Context<'a> = poise::Context<'a, super::Data, Error>;
pub struct Data {}

// ============== help: コマンドの使い方の表示 ==============
// - 引数: なし
//
// コマンドのヘルプを表示させます。
// 使い方や引数の意味などを記す
//
// ======================================================

/// ヘルプの表示
#[poise::command(slash_command)]
pub async fn help(ctx: Context<'_>) -> Result<(), Error> {
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
    // ---------- ファイルの読み込み ----------
    let path = "help.txt";
    let input = File::open(path)?;
    let buffered: BufReader<File> = BufReader::new(input);
    let mut res: String = String::new();

    res += "```"; // Markdown形式で出力
    for line in buffered.lines() {
        res += &line?;
        res += "\n";
    }
    res += "```"; // Markdown形式で出力

    let rep_builder = CreateReply::default().ephemeral(true).content(res);
    let _ = ctx.send(rep_builder).await;
    Ok(())
}

// ============== version: コマンドのバージョン情報を表示 ==============
// - 引数: なし
//
// コマンドのヘルプを表示させます。
// 使い方や引数の意味などを記す
//
// ======================================================

// バージョン情報
#[poise::command(slash_command)]
pub async fn version(ctx: Context<'_>) -> Result<(), Error> {
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

    // ---------- Cargo.toml内のバージョンを取得 ----------
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    let _ = ctx
        .send(CreateReply::default().ephemeral(true).content(VERSION))
        .await;

    Ok(())
}
