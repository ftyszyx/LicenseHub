# intro

# build and run

```
cargo build
cargo run
```

# test

test all

```
cargo test -- --test-threads=1
```

test apptest

```
cargo test --test app_tests -- --test-threads=1
cargo test --test reg_codes_tests -- --test-threads=1
cargo test --test resources_tests -- --test-threads=1
cargo test --test role_tests -- --test-threads=1
cargo test --test user_tests -- --test-threads=1
```

### 数据库迁移

```
cargo install sqlx-cli
```

数据库迁移

```bash
sqlx migrate run --database-url postgres://test:123456@localhost:5432/hub
```

清除

```
sqlx migrate revert --database-url postgres://test:123456@localhost:5432/hub
sqlx migrate revert --target-version 0 --database-url postgres://test:123456@localhost:5432/hub

```

### 生成entity

```
cargo install sea-orm-cli
```

```
sea-orm-cli generate entity -u "postgres://test:123456@localhost:5432/hub" -o "crates/data_model/src" --with-serde both
```

## docker发布

## 一些命令

### build docker

```
docker compose build server
```

### test docker

```
如果有 bash：docker compose run --rm -it server bash
```

#### 在服务器上执行

```
update_server.sh
```
