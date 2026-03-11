# LicenseHub Public Registration APIs

本文档说明新增的注册码公开接口，统一返回结构如下：

```json
{
  "code": 0,
  "message": "success",
  "data": {},
  "success": true
}
```

请求前缀：`/api`

## 1. 绑定注册码

- 路径：`POST /reg/bind`
- 路径：`GET /reg/bind`
- 说明：将注册码绑定到设备；时间类型应用返回过期时间，次数类型应用返回剩余次数。

### 请求参数

| 字段        | 类型   | 必填 | 说明     |
| ----------- | ------ | ---- | -------- |
| `app_key`   | string | 是   | 应用密钥 |
| `reg_code`  | string | 是   | 注册码   |
| `device_id` | string | 是   | 设备 ID  |

### 请求示例

```json
{
  "app_key": "demo-app-key",
  "reg_code": "ABCD-EFGH-IJKL",
  "device_id": "device-001"
}
```

### 成功响应示例

时间类型应用，`expire_time` 为 Unix 时间戳（秒）：

```json
{
  "code": 0,
  "message": "success",
  "success": true,
  "data": {
    "expire_time": 1775793600
  }
}
```

次数类型应用：

```json
{
  "code": 0,
  "message": "success",
  "success": true,
  "data": {
    "remain_count": 10
  }
}
```

---

## 2. 检查设备有效性

- 路径：`POST /reg/check`
- 路径：`GET /reg/check`
- 说明：检查当前设备是否仍然有效。

### 请求参数

| 字段        | 类型   | 必填 | 说明     |
| ----------- | ------ | ---- | -------- |
| `app_key`   | string | 是   | 应用密钥 |
| `device_id` | string | 是   | 设备 ID  |

### 请求示例

```json
{
  "app_key": "demo-app-key",
  "device_id": "device-001"
}
```

### 成功响应示例

时间类型应用，`expire_time` 为 Unix 时间戳（秒）：

```json
{
  "code": 0,
  "message": "success",
  "success": true,
  "data": {
    "expire_time": 1775793600
  }
}
```

次数类型应用：

```json
{
  "code": 0,
  "message": "success",
  "success": true,
  "data": {
    "remain_count": 8
  }
}
```

---

## 3. 扣减使用次数

- 路径：`POST /reg/usecount`
- 说明：仅适用于次数类型应用；如果剩余次数不足则返回失败。

### 请求参数

| 字段        | 类型        | 必填 | 说明                        |
| ----------- | ----------- | ---- | --------------------------- |
| `app_key`   | string      | 是   | 应用密钥                    |
| `device_id` | string      | 是   | 设备 ID                     |
| `use_count` | number      | 是   | 本次扣减次数，必须大于 0    |
| `use_info`  | object/null | 否   | 本次使用附加信息，支持 JSON |

### 请求示例

```json
{
  "app_key": "demo-app-key",
  "device_id": "device-001",
  "use_count": 2,
  "use_info": {
    "scene": "chapter-1",
    "seconds": 30
  }
}
```

### 成功响应示例

`time` 为 Unix 时间戳（秒）：

```json
{
  "code": 0,
  "message": "success",
  "success": true,
  "data": {
    "remain_count": 6
  }
}
```

### 失败响应示例

```json
{
  "code": 1,
  "message": "device remaining count is 0",
  "success": false,
  "data": null
}
```

---

## 4. 查询设备使用记录

- 路径：`GET /reg/use_records`
- 说明：外部应用按 `app_key + device_id` 查询使用记录，支持分页。

### 请求参数

| 字段        | 类型   | 必填 | 说明                |
| ----------- | ------ | ---- | ------------------- |
| `app_key`   | string | 是   | 应用密钥            |
| `device_id` | string | 是   | 设备 ID             |
| `page`      | number | 否   | 页码，默认 `1`      |
| `page_size` | number | 否   | 每页数量，默认 `20` |

### 请求示例

请求示例：

```text
/api/reg/use_records?app_key=demo-app-key&device_id=device-001&page=1&page_size=10
```

### 成功响应示例

```json
{
  "code": 0,
  "message": "success",
  "success": true,
  "data": {
    "list": [
      {
        "id": 1,
        "app_id": 2,
        "device_id": "device-001",
        "use_count": 2,
        "use_info": {
          "scene": "chapter-1",
          "seconds": 30
        },
        "time": 1741685400 // Unix 时间戳（秒）
      }
    ],
    "page": 1,
    "total": 1
  }
}
```

---

## 5. 错误说明

常见失败消息：

- `apps not found`：`app_key` 无效
- `reg_code not found`：注册码不存在
- `device not found`：设备不存在
- `device expired`：时间类型设备已过期
- `device remaining count is 0`：次数已用完
- `device remaining count is not enough: remain=x, required=y`：剩余次数不足
- `app code type is not count`：当前应用不是次数类型，不能调用 `usecount`

---

## 6. 路由清单

对应代码位置：`server/src/core/router.rs:186`

- `POST /api/reg/bind`
- `GET /api/reg/bind`
- `POST /api/reg/check`
- `GET /api/reg/check`
- `POST /api/reg/usecount`
- `GET /api/reg/use_records`
