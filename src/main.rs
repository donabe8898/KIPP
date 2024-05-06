//! # KIPP is an Integrated Project Management Program for Discord
//!
//! KIPPはDiscordBotとして起動し, タスク管理を行えるソフトウェアです.
//!
//! # リポジトリ
//! [https://github.com/donabe8898/KIPP](https://github.com/donabe8898/KIPP)
//!
//! # 使い方
//!
//! README.md参照

mod auth;
mod commands;
mod db;
mod disp;
mod imp;
mod support;

use poise::serenity_prelude as serenity;
use std::env;
use tokio;

/// 他のモジュールでも使いまわす
pub struct Data {}

// エラーハンドル用
//他のモジュールでも使いまわします
type Error = Box<dyn std::error::Error + Send + Sync>;
// type Context<'a> = poise::Context<'a, Data, serenity::Error>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    /* Bot起動フェーズ */
    dotenv::dotenv().ok();
    env_logger::init();

    let token = env::var("TOKEN").expect("missing get token");
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::showall(),
                commands::show(),
                commands::status(),
                commands::add(),
                commands::remove(),
                commands::clean(),
                commands::help(),
                commands::version(),
            ],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                Ok(Data {})
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
    Ok(())
}

/* 参考
    - 【Rust】 Rust + PostgreSQL + tokio_postgresでDBアクセスする方法
        - https://qiita.com/SakasuRobo/items/a72f916c1e1c8fb63de7
*/
