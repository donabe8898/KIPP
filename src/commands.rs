//! 全コマンドのrootモジュール



use poise::serenity_prelude::{
    self as serenity, ChannelId, CreateEmbed, CreateEmbedFooter, Error, UserId,
};
use poise::CreateReply;
use serenity::model::Timestamp;
use uuid::{self};
use crate::disp;
use crate::disp::Context;
use crate::imp;
use crate::support;


/// チャンネルごとにタスクの数を一覧形式で表示します。
#[poise::command(slash_command)]
pub async fn showall(ctx: Context<'_>,
                     #[description = "ユーザーを選択（任意）"] user: Option<serenity::User>,
                     #[description = "メッセージを自分以外にも表示"] display: Option<bool>) -> Result<(), Error> {
    let _ = disp::showall(ctx, user, display).await;
    Ok(())
}

/// チャンネルに属すタスクを表示
#[poise::command(slash_command)]
pub async fn show(ctx: Context<'_>,
                  #[description = "ユーザーを選択（任意）"] user: Option<serenity::User>,
                  #[description = "メッセージを自分以外にも表示"] display: Option<bool>, ) -> Result<(), serenity::Error> {
    let _ = disp::show(ctx, user, display).await;
    Ok(())
}

/// タスクを1件追加します
#[poise::command(slash_command)]
pub async fn add(
    ctx: Context<'_>,
    #[description = "タスク名"] task_name: String,
    #[description = "タスクの概要"] description: Option<String>,
    #[description = "担当者"] member: Option<serenity::Member>,
    #[description = "〆切日"] deadline: Option<String>,
) -> poise::serenity_prelude::Result<(), serenity::Error> {
    let _ = imp::add(ctx, task_name, description, member, deadline).await;
    Ok(())
}

/// タスクをチャンネルから削除します
#[poise::command(slash_command)]
pub async fn remove(
    ctx: Context<'_>,
    #[description = "タスクID"] task_id: String,
) -> poise::serenity_prelude::Result<(), serenity::Error> {
    let _ = imp::remove(ctx, task_id).await;
    Ok(())
}

/// タスクのステータスを変更します
#[poise::command(slash_command)]
pub async fn status(
    ctx: Context<'_>,
    task_id: String,
) -> poise::serenity_prelude::Result<(), serenity::Error> {
    let _ = imp::status(ctx, task_id).await;
    Ok(())
}


/// ヘルプの表示
#[poise::command(slash_command)]
pub async fn help(ctx: Context<'_>) -> Result<(), Error> {
    let _ = support::help(ctx).await;
    Ok(())
}

/// バージョン情報
#[poise::command(slash_command)]
pub async fn version(ctx: Context<'_>) -> Result<(), Error> {
    let _ = support::version(ctx).await;
    Ok(())
}