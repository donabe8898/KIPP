# Imadesho
Ticket制管理ツール for discord


# 起動
## Dockerでの立ち上げ
1. `Imadesho/docker`で`sudo docker-compose up -d && docker-compose exec postgres bash`してコンテナに入る。postgresはユーザー名なのでお好みで変更可能。
2. `su postgres`でユーザー切り替え
3. `psql`で入れる。

あとはテーブル作成など。

参考: [RustでPostgreSQLに接続する](https://qiita.com/takisawa/items/4327c5cb33a8d28ff5e9)