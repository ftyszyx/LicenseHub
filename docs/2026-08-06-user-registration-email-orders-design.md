# 用户注册、邮箱验证与购买订单归属设计方案

## 1. 背景与目标

LicenseHub 当前已经支持：

- 游客浏览商品、创建订单、支付并取得注册码。
- 用户使用用户名和密码注册、登录。
- 所有普通用户默认拥有推广能力；分销功能开启时可以进入推广中心。
- 管理员查看全部订单，游客通过订单号查询单笔订单。

本次计划补齐以下能力：

1. 在购买页和支付成功页引导游客注册，说明注册后可以长期保存订单和注册码；分销功能开启时，同时提示注册后可以分享推广链接获得佣金。
2. 登录用户购买时，将订单直接归属到当前用户。
3. 用户后台增加“我的订单”，可以查看自己的订单及订单发放的注册码。
4. 新用户注册必须完成邮箱验证。
5. 发送邮箱验证码时增加同邮箱 60 秒冷却，并使用邮箱、IP 多维限流防止批量发送。
6. 参考 `E:\opensource\mywork\lockpass-next` 的邮箱 challenge、验证码哈希、有效期、重发冷却、尝试次数、一次性消费和 SMTP 配置设计，但不照搬其账号体系。

本方案继续使用 LicenseHub 现有的 `users`、`user_roles`、JWT 和 RBAC，不新增独立账号主体，也不改变“所有用户默认可以推广”的原则。

## 2. 当前实现与主要缺口

### 2.1 注册

当前 `/api/register` 只接收：

```json
{
  "username": "user1",
  "password": "password"
}
```

接口直接创建用户、分配 `user` 角色并返回 JWT。当前缺少：

- 邮箱字段和邮箱唯一性约束。
- 邮箱验证码。
- 注册和邮件发送频率限制。
- 邮箱验证码的过期、失败次数和一次性消费记录。

### 2.2 订单归属

当前 `orders` 只有 `referrer_user_id`，它表示获得推广佣金的用户，不表示购买人。订单没有购买用户字段，因此：

- 登录用户和游客创建的订单没有区别。
- 用户后台无法按当前用户查询订单。
- 游客注册后无法自动找回已经购买的订单和注册码。

购买人和推广人必须使用两个不同字段，不能复用 `referrer_user_id`。

### 2.3 用户后台

当前普通用户后台只有首页和推广中心，没有“我的订单”页面。首页上的“订单查询”仍跳转到公共订单号查询页，并不代表登录用户自己的订单。

## 3. 总体设计原则

- `users` 仍然是唯一用户表，邮箱直接归入用户资料。
- 第一阶段保留“用户名 + 密码”登录方式，邮箱用于验证、订单归属和后续找回能力；暂不改为邮箱登录。
- 新注册用户必须先验证邮箱，再创建账号。
- 已存在的历史用户允许邮箱为空，不影响原有登录；后续可以增加“绑定邮箱”流程。
- 订单同时区分购买用户和推广用户：
  - `buyer_user_id`：实际购买人。
  - `referrer_user_id`：推广该订单并获得佣金的人。
- 登录状态由服务端 JWT 判断，前端不得提交或指定 `buyer_user_id`。
- 游客仍可购买，但结算前需要填写邮箱，用于订单找回和注册后的订单认领。
- 邮箱验证码只保存哈希，不保存明文。
- 所有表结构变化必须通过新的 migration 完成。
- 生产环境禁止执行本功能 migration 的回滚脚本。

## 4. 用户体验流程

### 4.1 游客在购买前注册

```text
游客进入购买页
    -> 页面展示注册提示
    -> 点击“立即注册”
    -> 保存当前商品、推广码和返回地址到 sessionStorage
    -> 完成邮箱验证码和注册
    -> 自动登录并返回原商品页
    -> 创建订单时服务端写入 buyer_user_id
    -> 支付成功后订单和注册码直接出现在“我的订单”
```

这是最推荐的路径，因为订单从创建开始就有明确归属。

### 4.2 游客先购买，支付后注册

```text
游客选择商品
    -> 结算弹窗要求填写购买邮箱
    -> 创建游客订单，保存 buyer_email 快照
    -> 支付成功并发放注册码
    -> 订单结果弹窗提示“注册并保存本订单”
    -> 注册页自动带入购买邮箱
    -> 用户完成该邮箱的验证并注册
    -> 注册事务自动认领相同邮箱的已支付、已发码和已退款订单
    -> 用户进入“我的订单”查看订单和注册码
```

游客填写的邮箱只是订单快照，只有完成该邮箱验证后，才能把订单归属到用户。

### 4.3 登录用户购买

```text
用户登录
    -> 进入购买页
    -> 创建订单时携带现有 Authorization
    -> 后端从 JWT 取得用户 ID
    -> 写入 buyer_user_id 和用户邮箱快照
    -> 支付成功后可在购买弹窗和用户后台查看注册码
```

如果请求没有 Authorization，则按游客处理；如果携带了无效或过期的 Authorization，应返回 `401`，不能静默降级成游客订单。

### 4.4 推广提示文案规则

购买页在未登录状态展示一条紧凑提示，不使用大面积营销卡片：

- 分销功能开启：`注册后可永久保存订单和注册码，还能分享推广链接，好友购买后获得佣金。`
- 分销功能关闭：`注册后可永久保存订单和注册码，登录后随时查看。`

操作入口：

- `立即注册`
- `已有账号，去登录`

支付成功后，未登录用户再次展示更明确的 `注册并保存本订单` 按钮。登录用户不展示注册提示。

## 5. 邮箱注册流程

### 5.1 前端步骤

注册表单包含：

- 用户名。
- 邮箱。
- 邮箱验证码。
- 密码。
- 确认密码。

点击“发送验证码”后：

1. 前端提交邮箱地址。
2. 服务端执行同邮箱 60 秒原子冷却，以及邮箱、IP 小时和每日限流。
3. 限流通过后发送邮箱验证码。
4. 前端进入 60 秒倒计时。
5. 用户填写 6 位邮箱验证码。
6. 服务端验证邮箱验证码，返回短期注册凭证。
7. 前端使用注册凭证、用户名和密码完成注册。

更换邮箱后，原邮箱 challenge 和注册凭证立即在前端作废，必须重新发送验证码。

### 5.2 为什么使用注册凭证

不建议让 `/api/register` 直接接收一个可重复使用的邮箱验证码。推荐参考 `lockpass-next`：

```text
邮箱验证码验证成功
    -> 生成高熵 verification_token
    -> 数据库只保存 token_hash
    -> verification_token 15 分钟有效
    -> /api/register 在事务中锁定并消费 token
    -> 同一个 token 不能创建第二个用户
```

这样可以把“邮箱所有权验证”和“最终创建用户”分开，同时避免验证码重放。

## 6. 发送冷却与限流设计

邮箱验证码发送不再要求用户完成图片验证码，避免增加注册操作成本。防刷依赖服务端冷却和多维限流，前端倒计时只用于交互提示，不能作为安全边界。

Redis 使用原子 `INCR + EXPIRE` 或 Lua 维护以下键：

```text
auth:email-cooldown:{email_hash}
auth:email-rate:email:1h:{email_hash}
auth:email-rate:email:24h:{email_hash}
auth:email-rate:ip:1h:{ip_hash}
auth:email-rate:ip:24h:{ip_hash}
```

同一邮箱的冷却键默认 60 秒内只允许第一次请求成功，保证并发请求也不能重复发送。数据库 challenge 的 `resend_after` 同时保留，用于返回剩余冷却时间和 Redis 数据异常时的补充保护。

## 7. 邮箱验证码设计

### 7.1 默认参数

| 参数 | 默认值 | 说明 |
| --- | ---: | --- |
| 验证码长度 | 6 位数字 | 便于输入 |
| 验证码有效期 | 10 分钟 | 过期后必须重发 |
| 重发冷却 | 60 秒 | 同一邮箱短时间不能重复发送 |
| 最大验证次数 | 5 次 | 超过后 challenge 作废 |
| 注册凭证有效期 | 15 分钟 | 验证邮箱后完成注册的时间窗口 |

### 7.2 验证码哈希

数据库不得保存验证码明文。建议使用独立的 `EMAIL_CODE_SECRET` 计算：

```text
HMAC-SHA256(
  EMAIL_CODE_SECRET,
  challenge_id + normalized_email + purpose + code
)
```

校验时使用常量时间比较。`EMAIL_CODE_SECRET` 必须通过生产环境变量或密钥管理系统提供，不使用代码中的开发默认值，也不复用 JWT 密钥。

### 7.3 发送频率限制

发送邮件前执行多维限流：

| 维度 | 建议限制 |
| --- | --- |
| 同一邮箱 | 60 秒 1 次、1 小时 5 次、24 小时 10 次 |
| 同一 IP | 1 小时 20 次、24 小时 50 次 |
| 单个邮箱 challenge | 最多验证 5 次 |

限流使用 Redis 原子计数，并给键设置与窗口一致的 TTL。接口被限制时返回 `429` 和 `retry_after_seconds`。

### 7.4 邮件发送失败

邮件发送第一阶段可以同步执行，但必须设置连接和发送超时。推荐流程：

1. 数据库写入待发送 challenge。
2. 调用 SMTP 发送。
3. 发送成功后写入 `sent_at`。
4. 发送失败时写入 `send_failed_at` 并返回邮件服务错误；用户需要等待当前 60 秒冷却结束后重试。

不能在日志中输出生产环境邮箱验证码。只有 `log` 开发模式可以输出验证码，并明确标记为非生产模式。

## 8. 数据库变更

需要新增 migration，例如：

```text
202608061000_user_email_order_ownership.up.sql
202608061000_user_email_order_ownership.down.sql
```

生产环境只允许执行 up migration，禁止回滚。

### 8.1 `users` 表

新增：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `email` | `VARCHAR(320) NULL` | 统一转为小写并去除首尾空格 |
| `email_verified_at` | `TIMESTAMPTZ NULL` | 邮箱验证完成时间 |

索引：

```sql
CREATE UNIQUE INDEX uq_users_email_lower
ON users (LOWER(email))
WHERE email IS NOT NULL;
```

历史用户保持 `email = NULL`，不强制补数据，也不影响现有用户名密码登录。

### 8.2 `orders` 表

新增：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `buyer_user_id` | `INTEGER NULL` | 购买用户，外键指向 `users.id` |
| `buyer_email` | `VARCHAR(320) NULL` | 下单时购买邮箱快照 |

约束和索引：

- `buyer_user_id` 使用 `ON DELETE SET NULL`，删除用户时保留订单历史。
- 为 `buyer_user_id, created_at DESC` 建立组合索引。
- 为未归属订单的 `LOWER(buyer_email)` 建立部分索引，方便注册后认领。

`buyer_email` 是历史快照。用户以后修改邮箱，不修改历史订单中的邮箱。

SeaORM 模型需要分别定义 `BuyerUser` 和 `ReferrerUser` 两个关系，不能继续只使用一个模糊的 `Users` relation。

### 8.3 `email_verification_challenges` 表

建议字段：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `id` | `UUID` | challenge ID，由应用生成 |
| `email` | `VARCHAR(320)` | 标准化邮箱 |
| `purpose` | `VARCHAR(32)` | 第一阶段为 `register` |
| `code_hash` | `VARCHAR(64)` | 邮箱验证码 HMAC |
| `attempts` | `INTEGER` | 已失败次数 |
| `expires_at` | `TIMESTAMPTZ` | 验证码过期时间 |
| `resend_after` | `TIMESTAMPTZ` | 允许再次发送时间 |
| `sent_at` | `TIMESTAMPTZ NULL` | 邮件发送成功时间 |
| `send_failed_at` | `TIMESTAMPTZ NULL` | 最近发送失败时间 |
| `verified_at` | `TIMESTAMPTZ NULL` | 验证成功时间 |
| `consumed_at` | `TIMESTAMPTZ NULL` | 注册流程消费时间 |
| `created_at` | `TIMESTAMPTZ` | 创建时间 |

建立 `(email, purpose, created_at DESC)` 索引。验证时使用 `SELECT ... FOR UPDATE`，避免并发请求重复验证同一个 challenge。

### 8.4 `email_verification_tokens` 表

建议字段：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `token_hash` | `VARCHAR(64)` | 注册凭证哈希，主键 |
| `challenge_id` | `UUID` | 对应邮箱 challenge |
| `email` | `VARCHAR(320)` | 已验证邮箱快照 |
| `purpose` | `VARCHAR(32)` | `register` |
| `expires_at` | `TIMESTAMPTZ` | 凭证过期时间 |
| `consumed_at` | `TIMESTAMPTZ NULL` | 使用时间 |
| `created_at` | `TIMESTAMPTZ` | 创建时间 |

服务端只返回一次明文 token，数据库只保存 SHA-256 或 HMAC 哈希。

## 9. 订单创建与归属规则

### 9.1 创建订单请求

调整请求：

```json
{
  "plan_id": 1,
  "pay_type": "wechat_native",
  "referral_code": "LHXXXXXXXXXX",
  "buyer_email": "buyer@example.com"
}
```

规则：

- 已登录用户：
  - 服务端从 JWT 写入 `buyer_user_id`。
  - `buyer_email` 使用用户表中的已验证邮箱。
  - 忽略前端伪造的购买用户信息。
- 游客：
  - `buyer_email` 必填并进行格式校验和标准化。
  - `buyer_user_id` 保持为空。
- 登录用户如果尚未绑定邮箱，允许购买，但应提示先绑定邮箱；第一阶段历史用户仍可按用户 ID 归属订单。
- 购买人和推广人相同时不产生推广佣金，防止自购自返。

### 9.2 注册后的订单认领

在创建用户、分配角色和消费邮箱注册凭证的同一个数据库事务中执行：

```sql
UPDATE orders
SET buyer_user_id = :new_user_id,
    updated_at = NOW()
WHERE buyer_user_id IS NULL
  AND LOWER(buyer_email) = :verified_email
  AND status IN (1, 2, 5);
```

状态范围：

- `1` 已支付。
- `2` 已发码。
- `5` 已退款。

失败、关闭的游客订单默认不自动放入用户后台，避免别人使用某个邮箱批量创建垃圾订单。对于注册时仍在支付中的当前订单，支付成功回调应再次根据已验证邮箱尝试归属。

认领操作必须幂等，只更新 `buyer_user_id IS NULL` 的订单，不能覆盖已经归属其他用户的订单。

### 9.3 支付成功时补充归属

支付成功事务中，如果订单仍没有 `buyer_user_id` 且存在 `buyer_email`：

1. 查询已验证且邮箱匹配的用户。
2. 找到唯一用户时写入 `buyer_user_id`。
3. 再完成发码、订单状态和佣金处理。

这样可以覆盖“游客创建订单后，在支付完成前注册”的情况。

## 10. API 设计

### 10.1 发送邮箱验证码

```http
POST /api/auth/email-verifications
```

请求：

```json
{
  "email": "user@example.com"
}
```

返回：

```json
{
  "challenge_id": "...",
  "email": "user@example.com",
  "expires_at": "2026-08-06T10:10:00Z",
  "resend_after_seconds": 60
}
```

### 10.2 验证邮箱验证码

```http
POST /api/auth/email-verifications/{challenge_id}/verify
```

请求：

```json
{
  "code": "123456"
}
```

返回一次性的 `verification_token` 和过期时间。

### 10.3 完成注册

```http
POST /api/register
```

请求调整为：

```json
{
  "username": "user1",
  "password": "strong-password",
  "email": "user@example.com",
  "verification_token": "..."
}
```

事务内执行：

1. 校验用户名和密码规则。
2. 根据 token hash 锁定注册凭证。
3. 校验未过期、未消费、用途为 `register`，且邮箱一致。
4. 再次检查用户名和邮箱唯一性。
5. 创建用户并写入 `email_verified_at`。
6. 分配普通用户角色并生成推广码。
7. 认领该邮箱的有效游客订单。
8. 消费 challenge 和注册凭证。
9. 提交事务后签发 JWT。

### 10.4 用户订单

建议沿用当前用户接口前缀：

```http
GET /api/admin/me/orders?page=1&page_size=20&status=2
GET /api/admin/me/orders/{order_no}
```

接口只能查询 `buyer_user_id = claims.user_id` 的订单，不依赖普通用户的 `orders:read` 管理权限。

列表返回：

- 订单号、应用、套餐、金额、支付方式、状态。
- 支付时间、创建时间、退款状态。
- 已发放注册码；前端提供复制按钮。

详情接口继续复用 `OrderInfo` 的构造逻辑，但必须在查询阶段增加购买用户条件，不能先按订单号查出后再判断。

## 11. 邮件服务与管理员设置

### 11.1 管理员设置项

在管理员“系统设置”页面增加“注册与邮件”分区：

| 设置项 | 默认值 | 说明 |
| --- | --- | --- |
| 开放用户注册 | 关闭 | 替代当前仅启动时读取的 `REGISTER_OPEN` |
| 邮件发送模式 | `log` | `log` 仅开发环境使用，生产使用 `smtp` |
| 发件人 | 空 | 例如 `LicenseHub <no-reply@example.com>` |
| SMTP 主机 | 空 | 邮件服务器地址 |
| SMTP 端口 | `587` | 默认 STARTTLS |
| SMTP 用户名 | 空 | 登录用户名 |
| SMTP 密码 | 空 | 只允许写入，不返回明文 |
| TLS 模式 | `starttls` | 第一阶段支持 `starttls`、`tls` |

验证码有效期、重发冷却和限流第一阶段使用服务端常量，不放到管理员页面，避免配置过多或被误设为不安全值。

### 11.2 动态注册开关

当前路由只在启动时根据 `REGISTER_OPEN` 决定是否注册 `/api/register`。调整后路由应始终存在，由 handler 实时读取系统设置：

- 关闭时返回 `REGISTRATION_DISABLED`。
- 关闭时购买页不显示注册入口。
- 已经发出的邮箱 challenge 在关闭注册后不能继续完成注册。

`REGISTER_OPEN` 可以保留为首次部署默认值，但系统设置初始化后以数据库配置为准。

### 11.3 SMTP 密钥保护

- 管理接口只返回 `smtp_password_set: true/false`。
- 密码输入留空表示保持原值，清除密码必须使用明确操作。
- 推荐新增 `SETTINGS_ENCRYPTION_KEY`，SMTP 密码加密后再写入 `system_settings`。
- `EMAIL_CODE_SECRET` 只从环境变量读取，不通过管理页面展示或修改。
- 公共 `/api/site-settings` 只返回 `registration_enabled` 等公开状态，绝不返回 SMTP 配置。
- 增加“发送测试邮件”按钮和管理员接口，保存配置前后都可以验证可用性。

### 11.4 邮件实现

参考 `lockpass-next/server/src/mailer.rs`，在 LicenseHub 增加独立 `Mailer` 服务：

- 使用 `lettre` 异步 SMTP。
- 同时发送纯文本和 HTML 邮件。
- 邮件主题和内容使用 LicenseHub 站点名称。
- HTML 中对站点名和展示文本进行转义。
- SMTP 调用设置超时。
- 日志只记录脱敏邮箱、challenge ID、发送结果和耗时。

## 12. 前端改动

### 12.1 注册页

重构现有 `RegisterView.vue`：

- 增加邮箱输入和发送验证码按钮。
- 增加 6 位邮箱验证码输入。
- 显示 60 秒重发倒计时。
- 注册成功后根据 `redirect` 返回购买页，否则进入用户首页。
- 从购买页跳转时预填购买邮箱，但不能跳过验证。

### 12.2 购买页

调整 `ProductCatalog.vue`：

- 未登录时展示注册提示条。
- 游客结算时增加购买邮箱。
- 登录用户显示当前账号，不要求重复填写邮箱。
- 保存待购买套餐、支付方式、推广码和返回地址，注册或登录后恢复。
- 支付成功且为游客时展示“注册并保存本订单”。
- 分销关闭时隐藏佣金相关提示，但保留订单保存提示。

### 12.3 用户后台

增加：

- 路由：`/user/orders`。
- 菜单：`我的订单`。
- 页面：订单列表、状态筛选、订单详情、注册码复制。

用户首页快捷入口由公共“订单查询”改为“我的订单”；公共订单号查询仍保留给未登录游客。

## 13. 安全补充

### 13.1 公共订单查询

当前公共接口只需要订单号就会返回注册码，订单号实际上承担了访问凭证的作用。建议与本功能同批或紧随其后增加高熵 `order_access_token`：

- 创建游客订单时返回一次明文 token，数据库只保存 hash。
- 前端保存到本地并在轮询、支付结果和公共查询时提交。
- 新订单查询需要 `order_no + access_token`。
- 已登录用户通过 `buyer_user_id` 查询，不需要该 token。
- 历史订单可以保留一段兼容期，再逐步关闭仅凭订单号查询。

这项不是邮箱注册的前置条件，但能够避免订单号泄露后注册码被他人查看。

### 13.2 日志与隐私

- 请求日志中的邮箱应脱敏，例如 `u***@example.com`。
- 不记录邮箱验证码、注册凭证、SMTP 密码和完整 Authorization。
- 用户列表和用户详情可以返回邮箱及验证状态，但继续禁止返回密码哈希和收款账户。
- 公共订单接口不返回 `buyer_email`。

### 13.3 密码规则

第一阶段建议：

- 长度 8 到 72 字节，兼容 bcrypt 输入限制。
- 服务端和前端同时校验，但以服务端为准。
- 注册、登录错误不能包含密码、验证码或 token 原文。

## 14. 建议错误码

| 错误码 | 场景 |
| --- | --- |
| `REGISTRATION_DISABLED` | 注册功能关闭 |
| `EMAIL_INVALID` | 邮箱格式错误 |
| `EMAIL_ALREADY_REGISTERED` | 邮箱已注册 |
| `EMAIL_CODE_RATE_LIMITED` | 邮件发送过于频繁 |
| `EMAIL_CODE_INVALID` | 邮箱验证码错误 |
| `EMAIL_CODE_EXPIRED` | 邮箱验证码过期 |
| `EMAIL_CODE_ATTEMPTS_EXCEEDED` | 邮箱验证码错误次数过多 |
| `EMAIL_VERIFICATION_TOKEN_INVALID` | 注册凭证无效或已消费 |
| `EMAIL_VERIFICATION_TOKEN_EXPIRED` | 注册凭证过期 |
| `BUYER_EMAIL_REQUIRED` | 游客下单未填写邮箱 |

## 15. 测试范围

### 15.1 后端单元测试

- 邮箱标准化和格式校验。
- 邮箱验证码 HMAC 和常量时间校验。
- 密码长度边界。
- SMTP 配置校验和邮件模板 HTML 转义。
- 日志邮箱脱敏。

### 15.2 后端集成测试

- 同一邮箱 60 秒冷却和并发发送限制。
- 同一邮箱和同一 IP 的小时、每日发送限流。
- 邮箱验证码成功、错误、过期、超次和重复验证。
- 注册凭证过期和重复消费。
- 用户名和邮箱并发注册时唯一约束生效。
- 用户、角色、注册凭证消费和订单认领在一个事务中完成。
- 登录用户下单自动写入 `buyer_user_id`。
- 游客订单在注册后被认领。
- 游客支付期间注册，支付成功后补充订单归属。
- 用户只能读取自己的订单和注册码。
- 购买人等于推广人时不产生佣金。

### 15.3 前端测试

- 邮箱验证码倒计时和重发。
- 注册返回购买页并恢复套餐、支付方式和推广码。
- 登录与未登录购买流程。
- 分销开关开/关时提示文案和入口正确。
- 用户后台订单状态、注册码显示和复制。
- 手机和桌面宽度下表单、弹窗、提示条不溢出。

## 16. 实施顺序

### 第一阶段：数据和基础服务

1. 新增 migration 和 SeaORM 模型字段。
2. 增加 Mailer、邮箱标准化、验证码哈希和 Redis 限流能力。
3. 增加邮箱发送冷却、邮箱 challenge 和注册凭证接口。
4. 改造注册事务。

### 第二阶段：订单归属

1. 创建订单支持可选登录身份和游客邮箱。
2. 支付成功时补充订单归属。
3. 注册时认领历史成功订单。
4. 增加用户订单接口。
5. 增加自购不返佣规则。

### 第三阶段：页面

1. 改造注册页面。
2. 增加购买页注册提示和游客邮箱输入。
3. 增加支付成功后的注册入口。
4. 增加用户后台“我的订单”。
5. 增加管理员邮件设置和测试邮件功能。

### 第四阶段：安全加固与验收

1. 增加公共订单访问 token。
2. 完成限流、并发、权限和日志脱敏测试。
3. 使用游客购买、购买后注册、登录购买、推广购买四条完整链路进行浏览器自测。

## 17. 第一阶段明确不包含

- 邮箱验证码登录。
- 忘记密码和邮件重置密码。
- 用户自行修改或解绑邮箱。
- 第三方 OAuth 登录。
- 短信验证码。
- 购买成功邮件和注册码邮件。
- 管理员人工合并两个用户账号。

这些能力可以复用本次的邮箱 challenge、注册凭证和 Mailer 基础设施，在后续迭代增加。
