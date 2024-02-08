//! support.rs
//! ========== サポートコマンド実装 ==========
//! - /help: コマンドのヘルプを表示
//! - /version: コマンドのバージョン情報等を表示

use poise::serenity_prelude::*;
use poise::*;

use std::env;

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
    Ok(())
}
