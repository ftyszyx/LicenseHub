# 应用版本配置同步到对象存储方案

## 背景

应用已有版本字段：

- `app_vername`
- `app_vercode`
- `app_download_url`
- `app_res_url`
- `app_update_info`
- `manifest_extra`

新功能要把这些应用配置发布为公开可读取的版本文件，客户端通过对象存储中的
`apps/{app_id}/latest.json` 获取最新版本、更新说明和下载地址。

数据库仍是版本信息源数据，对象存储只作为公开分发层。

`app_res_url` 仍保留在应用表中，但发布 manifest 时为空则不输出 `res_url`。
`manifest_extra` 用于每个应用的自定义发布字段，后台以键值行编辑，保存为 JSON object。

## 目标

1. 一个应用可以手动同步到多个对象存储平台。
2. 支持 Aliyun OSS 和 Cloudflare R2。
3. 同步路径固定为 `apps/{app_id}/latest.json`，暂不考虑回滚和历史版本文件。
4. 每次同步都写入版本同步日志。
5. 同步渠道配置独立建表管理，风格参考支付渠道管理。

## 数据表

### apps

应用表新增发布扩展字段：

```sql
ALTER TABLE "apps"
    ADD COLUMN "manifest_extra" JSONB NOT NULL DEFAULT '{}'::jsonb;
```

`manifest_extra` 只用于版本 manifest 的 `extra` 字段。为空对象时不输出；非空时整体放入 `extra`，
避免自定义字段和平铺的系统字段冲突。

### storage_channels

用于管理对象存储同步渠道。

```sql
CREATE TABLE "storage_channels" (
    "id" SERIAL PRIMARY KEY,
    "name" VARCHAR(255) NOT NULL,
    "provider" VARCHAR(32) NOT NULL,
    "status" SMALLINT NOT NULL DEFAULT 1,
    "sort_order" INTEGER NOT NULL DEFAULT 0,
    "config" JSONB NOT NULL,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

`provider` 取值：

- `aliyun_oss`
- `cloudflare_r2`

`status` 取值：

- `0`: disabled
- `1`: enabled

`config` 示例：

```json
{
  "bucket": "licensehub",
  "endpoint": "oss-cn-hangzhou.aliyuncs.com",
  "access_key_id": "xxx",
  "access_key_secret": "xxx",
  "public_base_url": "",
  "prefix": "apps",
  "storage_class": "standard",
  "object_acl": "public-read"
}
```

Aliyun OSS 创建渠道时只需要填写 `endpoint`、`bucket` 和访问密钥。`endpoint` 可以填
`oss-cn-guangzhou.aliyuncs.com` 这种不带协议的形式，后端会统一保存为 `https://...`。
`public_base_url` 可以不填，系统会根据 `bucket` 和 `endpoint` 自动补齐：

```text
public_base_url = https://{bucket}.{endpoint}
```

Aliyun OSS 的 `storage_class` 可选值：

- `standard`
- `ia`
- `archive`
- `cold_archive`
- `deep_cold_archive`

同步上传时会把该配置写入 `x-oss-storage-class` 请求头。

Aliyun OSS 的 `object_acl` 用于设置上传后的对象权限，默认值为 `public-read`，这样客户端可以直接读取
`latest.json`。可选值：

- `default`: 继承 Bucket
- `private`: 私有
- `public-read`: 公共读
- `public-read-write`: 公共读写

当值不是 `default` 时，同步上传会把该配置写入 `x-oss-object-acl` 请求头；老渠道没有该字段时会按
`public-read` 处理。

Cloudflare R2 示例：

```json
{
  "bucket": "licensehub",
  "endpoint": "https://<account_id>.r2.cloudflarestorage.com",
  "access_key_id": "xxx",
  "access_key_secret": "xxx",
  "public_base_url": "https://cdn.example.com",
  "prefix": "apps"
}
```

Cloudflare R2 的 `endpoint` 依赖账号 ID，`public_base_url` 通常是自定义域名或 r2.dev
公开域名，不能只根据 region 推导，需要手动配置。

### app_version_sync_logs

记录每一次应用版本配置同步结果。一次手动同步多个渠道时，每个渠道写一条日志。

```sql
CREATE TABLE "app_version_sync_logs" (
    "id" BIGSERIAL PRIMARY KEY,
    "app_id" INTEGER NOT NULL,
    "storage_channel_id" INTEGER NOT NULL,
    "provider" VARCHAR(32) NOT NULL,
    "object_key" VARCHAR(512) NOT NULL,
    "public_url" TEXT NOT NULL,
    "manifest" JSONB NOT NULL,
    "status" SMALLINT NOT NULL,
    "error_message" TEXT,
    "etag" VARCHAR(255),
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "finished_at" TIMESTAMPTZ
);
```

`status` 取值：

- `0`: pending
- `1`: success
- `2`: failed

## Manifest 格式

对象路径：

```text
apps/{app_id}/latest.json
```

这里的 `{app_id}` 使用 `apps.app_id`，不是数据库自增 ID。

内容：

```json
{
  "schema_version": 1,
  "app_id": "ABC123",
  "name": "My App",
  "version": {
    "name": "1.0.0",
    "code": 1
  },
  "update_info": "本次更新说明",
  "download_url": "https://example.com/app.exe",
  "res_url": "https://example.com/res.zip",
  "extra": {
    "channel": "stable",
    "force_update": false,
    "min_version_code": 100
  },
  "published_at": "2026-06-04T10:00:00+08:00"
}
```

字段说明：

- `res_url`: 来自 `apps.app_res_url`，为空时不输出。
- `extra`: 来自 `apps.manifest_extra`，用于后台为每个应用配置的自定义字段；为空对象时不输出。

## 后端接口

同步渠道管理：

```text
GET    /api/admin/storage-channels/list
POST   /api/admin/storage-channels
PUT    /api/admin/storage-channels/{id}
DELETE /api/admin/storage-channels/{id}
```

版本配置同步：

```text
GET  /api/admin/apps/{id}/version-manifest
POST /api/admin/apps/{id}/sync-version
GET  /api/admin/version-sync-logs
```

`POST /api/admin/apps/{id}/sync-version` 请求体：

```json
{
  "channel_ids": [1, 2]
}
```

`channel_ids` 为空或不传时，同步所有启用渠道。

## 同步流程

1. 管理员在应用列表点击“同步版本”。
2. 后端读取应用信息并生成 manifest。
3. 后端读取选中的同步渠道；未传渠道时读取所有启用渠道。
4. 对每个渠道计算 `object_key`：

```text
{prefix}/{app.app_id}/latest.json
```

`prefix` 默认 `apps`，保存前会去掉首尾 `/`。

5. 后端逐个上传 manifest。
6. 每个渠道单独写同步日志。某个渠道失败不会影响其他渠道继续同步。
7. 接口返回每个渠道的结果列表。

## 权限

新增权限资源：

- `storage_channels:read`
- `storage_channels:create`
- `storage_channels:update`
- `storage_channels:delete`
- `version_sync:read`
- `version_sync:create`

管理员角色默认拥有以上权限。

## 前端入口

1. 系统菜单增加“同步渠道”页面，用于管理 OSS/R2 渠道。
2. 应用管理页面补充版本字段编辑：
   - 版本名称
   - 版本码
   - 下载地址
   - 资源地址
   - 更新说明
3. 应用管理操作列增加“同步版本”。
4. 同步弹窗支持多选渠道、预览 manifest、展示同步结果。

## 测试建议

后端测试使用 `mock://` endpoint 避免真实访问网络。endpoint 以 `mock://` 开头时，上传逻辑直接返回成功，并写入 `mock-etag`。

需要覆盖：

1. 同步渠道 CRUD。
2. manifest 预览内容。
3. 指定多个渠道手动同步。
4. 不传 `channel_ids` 时同步所有启用渠道。
5. 同步成功后写入日志。
