# KIPP - KIPP is an Integrated Project Management Program for Discord

KIPPはDiscordと上手に統合するプロジェクト管理プログラムです.

- 要するにタスク管理Botなので、TODOアプリとしても使えます.

# Usage
```
[] - 必須の引数
{} - 任意の引数

SlashCommand:
    # ヘルプ
    /help

    # バージョン表示
    /version

    # タスク追加
    /add [task_name] {description} {member} {deadline}
        - task_name <text>      : タスク名
        - description <text>    : タスクの説明
        - member <User>         : 担当者
        - deadline <YYYY-MM-DD> : 締切日
            入力例: 2024-9-11

    # タスク削除
    /remove  [task_id]
        - task_id <text> : タスクのUUID

    # タスクのステータスを変更
    /status  [task_id]
        - task_id <text> : タスクのUUID

    # ギルド内のタスク数を表示
    /showall {member} {display}
        - member <User>  : 担当者
        - display <bool> : 他の人にも見せる

    /show  {member} {display}
        - member <User>  : 担当者
        - display <bool> : 他の人にも見せる

```

> [!TIP]
> プロジェクトで`cargo doc --no-deps --open`コマンドを使用してドキュメントを見ることができます。



# Run
## Dockerでの立ち上げ

> [!WARNING]
> Discord botは`.env`ファイルに記載されるギルド（サーバー）でしか動作しません。

1. `KIPP/docker`で`sudo docker-compose up -d && docker-compose exec postgres bash`してコンテナに入る。postgresはユーザー名なのでお好みで変更可能。
2. `su postgres`でユーザー切り替え
3. `psql`コマンドでpostgresに入る
4. `CREATE EXTENSION IF NOT EXISTS "uuid-ossp";`を実行してUUID拡張機能をインストール
5. `dotenv.sample`を参考にプロジェクトルートに`.env`を作成
6. bot起動


参考: [RustでPostgreSQLに接続する](https://qiita.com/takisawa/items/4327c5cb33a8d28ff5e9)

# アップデート情報

- README.mdのUsaseの内容を修正しました.
- コードの保守性とドキュメンテーションコメントの生成を可能にする為, commands.rsにモジュール分割しました.


# License
MIT License

