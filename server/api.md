# LicenseHub Public Registration APIs

这些接口用于客户端应用完成注册码绑定、设备授权检查、次数扣减和使用记录查询。统一前缀为 `/api`，统一返回结构如下：

```json
{
  "code": 0,
  "message": "success",
  "data": {},
  "success": true
}
```

## 绑定注册码

- `POST /reg/bind`
- `GET /reg/bind`

请求参数：

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `app_key` | string | 是 | 应用密钥 |
| `reg_code` | string | 是 | 注册码，兼容 `code` 参数别名 |
| `device_id` | string | 是 | 设备 ID |

时间类应用返回 `expire_time`，值为 Unix 秒级时间戳；次数类应用返回 `remain_count`。

注册码可绑定的设备数量取生成注册码时的应用“设备数上限”快照。重复绑定同一设备不占用新名额；超过上限时返回 `REG_CODE_DEVICE_LIMIT_REACHED`。

## 检查设备授权

- `POST /reg/check`
- `GET /reg/check`

请求参数：

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `app_key` | string | 是 | 应用密钥 |
| `device_id` | string | 是 | 设备 ID |

无设备记录时会按应用试用配置创建试用授权。

## 扣减次数

- `POST /reg/usecount`

请求参数：

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `app_key` | string | 是 | 应用密钥 |
| `device_id` | string | 是 | 设备 ID |
| `use_count` | number | 是 | 本次扣减次数，必须大于 0 |
| `use_info` | object | 否 | 使用记录附加信息 |

同一注册码绑定的全部设备共享该注册码的总次数。任一设备扣次后，其他绑定设备检查到的余额会同步减少。扣减成功后返回该设备当前可用的合计剩余次数，并写入 `use_records`。

注册码被撤销授权或对应订单退款后，其关联的全部设备有效期和设备次数都会被清空，共享次数余额同时归零。

## 查询使用记录

- `GET /reg/use_records`

请求参数：

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `app_key` | string | 是 | 应用密钥 |
| `device_id` | string | 是 | 设备 ID |
| `page` | number | 否 | 页码 |
| `page_size` | number | 否 | 每页数量 |

后台管理端可通过 `GET /api/admin/use_records/list` 按 `app_id`、`device_id` 查询使用记录。
