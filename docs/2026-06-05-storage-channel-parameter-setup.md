# 同步渠道参数获取与配置指南

本文说明如何在 Aliyun OSS 和 Cloudflare R2 中获取同步渠道参数，并在后台「系统管理 / 同步渠道」中填写。

同步完成后，版本文件固定发布到：

```text
{public_base_url}/apps/{app_id}/latest.json
```

其中 `{app_id}` 是应用管理中的应用 ID，不是数据库自增 ID。
`public_base_url` 是客户端访问地址前缀，可以只包含域名，也可以包含路径。例如 R2 公开访问前缀填写 `https://apphub.1postpro.com/apphub` 时，最终地址会是 `https://apphub.1postpro.com/apphub/apps/{app_id}/latest.json`。

## 字段对应关系

| 后台字段 | Aliyun OSS | Cloudflare R2 |
| --- | --- | --- |
| 服务商 | `aliyun_oss` | `cloudflare_r2` |
| Bucket | OSS Bucket 名称 | R2 Bucket 名称 |
| Region | 后台不需要填写 | 通常填写 `auto` |
| Endpoint | OSS 地域 Endpoint，例如 `oss-cn-guangzhou.aliyuncs.com` | S3 API Endpoint，例如 `https://<ACCOUNT_ID>.r2.cloudflarestorage.com` |
| Access Key ID | RAM 用户 AccessKey ID | R2 API Token 的 Access Key ID |
| Access Key Secret | RAM 用户 AccessKey Secret | R2 API Token 的 Secret Access Key |
| 客户端访问地址前缀 | Aliyun 当前按 `https://{bucket}.{endpoint}` 自动生成 | 自定义域名、`r2.dev` 公共地址，或带路径的公开前缀 |
| 前缀 | 默认 `apps` | 默认 `apps` |
| 存储类型 | Aliyun OSS 可选 | R2 不需要 |
| 权限类型 | 推荐 `public-read` | R2 通过公共域名控制访问 |

## Aliyun OSS

### 1. 创建或确认 Bucket

进入阿里云控制台：

1. 打开「对象存储 OSS」。
2. 创建 Bucket，或选择已有 Bucket。
3. 记录 Bucket 名称，例如：

```text
bytefuse
```

4. 在 Bucket 概览或「地域和访问域名」中找到公网 Endpoint，例如广州地域：

```text
oss-cn-guangzhou.aliyuncs.com
```

后台填写 Endpoint 时填地域 Endpoint 即可，不要填带 Bucket 的域名。

正确：

```text
oss-cn-guangzhou.aliyuncs.com
```

不要填：

```text
bytefuse.oss-cn-guangzhou.aliyuncs.com
```

系统上传时会自动使用三段式 Bucket 域名：

```text
https://bytefuse.oss-cn-guangzhou.aliyuncs.com/apps/{app_id}/latest.json
```

### 2. 创建 RAM 用户 AccessKey

推荐使用 RAM 用户 AccessKey，不要直接使用阿里云主账号 AccessKey。

操作步骤：

1. 打开「访问控制 RAM」。
2. 创建 RAM 用户，启用 OpenAPI 调用访问。
3. 创建 AccessKey。
4. 保存 `AccessKey ID` 和 `AccessKey Secret`，Secret 创建后通常不能再次查看。
5. 给 RAM 用户授权 OSS 写入目标 Bucket 的权限。

如果只给版本同步使用，权限可以尽量限制到目标 Bucket 和 `apps/*` 目录。至少需要允许上传对象；如果后台选择对象权限 `public-read`，还要确保允许设置对象 ACL 或有等效权限。

示例策略可按实际 Bucket 名称调整：

```json
{
  "Version": "1",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "oss:PutObject",
        "oss:GetObject",
        "oss:PutObjectAcl"
      ],
      "Resource": [
        "acs:oss:*:*:bytefuse/apps/*"
      ]
    }
  ]
}
```

### 3. 后台填写示例

在「系统管理 / 同步渠道」中新建渠道：

| 字段 | 示例 |
| --- | --- |
| 渠道名称 | `Aliyun OSS - bytefuse` |
| 服务商 | `阿里云 OSS` |
| 状态 | `启用` |
| Bucket | `bytefuse` |
| Endpoint | `oss-cn-guangzhou.aliyuncs.com` |
| Access Key ID | RAM 用户 AccessKey ID |
| Access Key Secret | RAM 用户 AccessKey Secret |
| 存储类型 | `标准存储` |
| 权限类型 | `公共读` |
| 前缀 | `apps` |

`权限类型` 推荐选择 `公共读`，因为客户端需要直接读取 `latest.json`。如果选择 `继承 Bucket`，请确认 Bucket Policy 或 Bucket ACL 已允许公网读取对应对象。

### 4. Aliyun OSS 验证

应用发布成功后访问：

```text
https://{bucket}.{endpoint}/apps/{app_id}/latest.json
```

例如：

```text
https://bytefuse.oss-cn-guangzhou.aliyuncs.com/apps/NAE2W1U7444J/latest.json
```

如果浏览器能直接看到 JSON，说明公开读取配置正确。

### 5. Aliyun OSS 常见问题

#### 403 `SecondLevelDomainForbidden`

通常是上传请求打到了二级域名，例如：

```text
https://oss-cn-guangzhou.aliyuncs.com/apps/...
```

后台 Endpoint 应填写地域 Endpoint，不要填完整对象 URL；当前系统会自动拼成：

```text
https://{bucket}.{endpoint}/{object_key}
```

#### 发布成功但浏览器访问 403

检查同步渠道的 `权限类型` 是否为 `公共读`，然后重新同步版本。旧对象不会因为修改渠道配置自动变公开，必须重新上传一次。

也要检查 Bucket Policy 是否显式拒绝公网读取。

#### 中国内地 Bucket 默认 Endpoint 受限

阿里云官方文档提到，2025-03-20 起部分中国内地新用户在数据 API 操作上需要使用自定义域名。若默认 OSS Endpoint 上传受限，请参考阿里云 OSS 自定义域名/CNAME 文档，后续也可以在系统中扩展 Aliyun 自定义公开域名配置。

## Cloudflare R2

### 1. 创建或确认 Bucket

进入 Cloudflare Dashboard：

1. 打开「Storage & databases / R2」。
2. 创建 Bucket，或选择已有 Bucket。
3. 记录 Bucket 名称，例如：

```text
licensehub
```

### 2. 创建 R2 API Token

R2 使用 S3 兼容 API 上传，需要 R2 API Token。

操作步骤：

1. 打开 Cloudflare Dashboard。
2. 进入「Storage & databases / R2 / Overview」。
3. 在 API Tokens 区域点击 Manage。
4. 创建 Account API token 或 User API token。
5. 权限选择 `Object Read & Write`。
6. 建议选择 `Apply to specific buckets only`，只授权给目标 Bucket。
7. 创建后复制并保存：
   - `Access Key ID`
   - `Secret Access Key`
   - S3 API Endpoint

Endpoint 格式通常是：

```text
https://<ACCOUNT_ID>.r2.cloudflarestorage.com
```

R2 的 SDK region 通常填写：

```text
auto
```

### 3. 配置客户端访问地址前缀

R2 上传 Endpoint 和客户端访问地址前缀是两回事：

- `Endpoint`：用于后台上传，格式是 `https://<ACCOUNT_ID>.r2.cloudflarestorage.com`。
- `客户端访问地址前缀`：给 App 客户端读取 `latest.json`，必须是公开可访问的 URL 前缀。

生产环境推荐绑定自定义域名：

```text
https://updates.example.com
```

如果你的公开访问路径带有 Bucket 名称或其他路径，例如：

```text
https://apphub.1postpro.com/apphub
```

后台也要填写完整前缀。系统会在它后面追加对象路径，生成：

```text
https://apphub.1postpro.com/apphub/apps/{app_id}/latest.json
```

操作步骤：

1. 进入 R2 Bucket。
2. 打开 Settings。
3. 在 Custom Domains 中添加域名。
4. 等待状态变为 Active。
5. 后台 `客户端访问地址前缀` 填该域名或完整公开前缀，不要带最后的 `/`。

如果只填写域名，例如：

```text
apphub.1postpro.com
```

系统会自动保存为：

```text
https://apphub.1postpro.com
```

生产环境仍建议填写完整的 `https://...`，避免误判。

开发测试也可以启用 Public Development URL，得到类似 `r2.dev` 的公共地址。但 Cloudflare 官方说明 `r2.dev` 主要用于非生产环境，生产环境建议使用自定义域名。

### 4. 后台填写示例

在「系统管理 / 同步渠道」中新建渠道：

| 字段 | 示例 |
| --- | --- |
| 渠道名称 | `Cloudflare R2 - licensehub` |
| 服务商 | `Cloudflare R2` |
| 状态 | `启用` |
| Bucket | `licensehub` |
| Region | `auto` |
| Endpoint | `https://<ACCOUNT_ID>.r2.cloudflarestorage.com` |
| 客户端访问地址前缀 | `https://apphub.1postpro.com/apphub` |
| Access Key ID | R2 API Token 的 Access Key ID |
| Access Key Secret | R2 API Token 的 Secret Access Key |
| 前缀 | `apps` |

同步后客户端访问：

```text
https://apphub.1postpro.com/apphub/apps/{app_id}/latest.json
```

### 5. Cloudflare R2 常见问题

#### 上传成功但客户端访问地址打不开

检查 `客户端访问地址前缀` 是否填成了 S3 API Endpoint。

错误：

```text
https://<ACCOUNT_ID>.r2.cloudflarestorage.com
```

正确：

```text
https://updates.example.com
```

如果公开访问路径包含 Bucket 路径，也要一并填写：

```text
https://apphub.1postpro.com/apphub
```

或者测试环境的 Public Bucket URL：

```text
https://pub-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx.r2.dev
```

#### 403 或 404

检查：

1. 自定义域名是否 Active。
2. Bucket 的 Public URL Access 是否 Allowed。
3. 同步路径是否为 `apps/{app_id}/latest.json`。
4. 后台 `前缀` 是否填写为 `apps`。

#### 签名失败

检查：

1. Endpoint 是否为 `https://<ACCOUNT_ID>.r2.cloudflarestorage.com`。
2. Region 是否为 `auto`。
3. Access Key ID / Secret Access Key 是否来自同一个 R2 API Token。
4. Token 是否拥有目标 Bucket 的 Object Read & Write 权限。

## 发布后检查清单

1. 渠道状态为 `启用`。
2. 前缀为 `apps`。
3. 应用管理中点击「同步版本」。
4. 打开「发布记录」，确认状态为成功。
5. 复制公开地址，在浏览器或命令行访问：

```bash
curl -i "https://example.com/apps/{app_id}/latest.json"
```

期望：

- HTTP 状态为 `200`。
- `Content-Type` 为 JSON 或可被浏览器正常显示。
- JSON 中包含 `app_id`、`version`、`download_url`、`published_at` 等字段。

## 安全建议

1. 不要使用主账号 AccessKey。
2. AccessKey 只授权目标 Bucket 和 `apps/*` 路径。
3. 不要把 AccessKey 写进代码仓库。
4. 离职、泄露或测试结束后及时禁用旧 Key。
5. R2 的 `r2.dev` 公共地址适合测试，生产环境优先使用自定义域名。

## 参考资料

- Aliyun OSS 地域和 Endpoint：<https://www.alibabacloud.com/help/en/oss/choose-an-oss-region>
- Aliyun OSS RAM 用户 AccessKey：<https://www.alibabacloud.com/help/doc-detail/375246.html>
- Aliyun OSS Bucket ACL：<https://www.alibabacloud.com/help/en/oss/developer-reference/manage-bucket-acls-1>
- Aliyun OSS 访问域名和网络方案：<https://www.alibabacloud.com/help/en/oss/user-guide/access-and-network-overview>
- Cloudflare R2 S3 API：<https://developers.cloudflare.com/r2/get-started/s3/>
- Cloudflare R2 Public Buckets：<https://developers.cloudflare.com/r2/data-access/public-buckets/>
