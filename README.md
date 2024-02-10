# KIPP - KIPP is an Integrated Project Management Program for Discord


KIPPはDiscordと上手に統合するプロジェクト管理プログラムです。


- TODOアプリとしてもお使い頂けます。

# Usase
```
[] - 必須の引数
{} - 任意の引数

SlashCommand:
    /help                            ヘルプの表示
    /version                         バージョン情報の表示

    /add     [task_name] [member]    タスクの追加
    /remove  [task_id]               タスクの削除
    /status  [task_id]               タスクのステータスを変更

    /showall {member}                チャンネルごとのタスク件数を出力
    /show    {member}                タスクの表示
```



# Run
## Dockerでの立ち上げ
1. `KIPP/docker`で`sudo docker-compose up -d && docker-compose exec postgres bash`してコンテナに入る。postgresはユーザー名なのでお好みで変更可能。
2. `su postgres`でユーザー切り替え
3. `psql`で入れる。
4. `CREATE EXTENSION IF NOT EXISTS "uuid-ossp";`を実行してUUID拡張機能をインストール
5. `env_sampel`を参考にプロジェクトルートに`.env`を作成
6. 起動


参考: [RustでPostgreSQLに接続する](https://qiita.com/takisawa/items/4327c5cb33a8d28ff5e9)


# License
MIT License

