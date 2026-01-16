1) 推荐的总体模型：RBAC + Permission（资源-动作）+（可选）多租户/应用维度
核心概念
User（用户）
可以拥有多个 Role（你现在就是 user_roles 多对多）
Role（角色）
一组 Permission 的集合（role_permissions）
Permission（权限）
推荐用 (resource, action) 表达
你 SQL 里也是 resource + action，很好
建议的权限命名规范
resource：业务资源（建议和 API/页面模块对齐）
例如：users, roles, apps, reg_codes, devices
action：动作
例如：READ, CREATE, UPDATE, DELETE
name：唯一 key（用于代码/配置引用）
例如：users:read, users:create
这样后端判断权限、前端显示菜单，都可以围绕同一套 key 做一致化。

2) 后端：鉴权最佳落点（建议放在“路由层/handler层”的统一 middleware）
你现在 router 上已经有：

/api/admin/* 统一 hoop auth_middleware::auth
说明 JWT 鉴权（认证）已在做
下一步是做 授权（Authorization），推荐思路：

(A) 每个 endpoint 声明它需要的权限
例如：

GET /api/admin/users/list -> users:read
POST /api/admin/users -> users:create
PUT /api/admin/users/{id} -> users:update
DELETE /api/admin/users/{id} -> users:delete
(B) Middleware/Hoop 统一检查
做一个类似 require_permission("users", "READ") 的 hoop（或者宏/包装函数），这样每个路由加一行即可。

(C) 权限判断数据来源
两种选择：

JWT 内带 role_ids（你现在 token 就是 role_ids）
请求时根据 role_ids 查 role_permissions -> permissions
可加 Redis 缓存：role_id -> permissions[]

3) 数据库初始化与演进策略（很关键）
你现在 migration 里插入了：

roles: admin/user/guest
permissions: 只有 all * *

建议改成“三层策略”
(1) 超级权限 all:* 仅 admin 拥有
(2) 给 user/guest 分配最小权限
user：比如 reg_codes:read、apps:read 等
guest：可能只允许公开接口，不走 admin API
(3) 权限项由代码维护 / 自动同步（可选，但很好用）
每次启动时扫描一份权限定义表（常量列表），把缺失的 permission 自动 upsert 到 DB
避免“写了接口忘了加权限项”的问题
4) 前端（admin）：菜单/按钮显示建议用“权限驱动”，不要硬编码 role
前端最常见的坑是：

写死 if role === admin 显示按钮，后期角色一多就崩
或者完全不控制显示，靠后端报 403（体验差）
推荐做法
登录后调用 GET /api/admin/me
再提供一个接口：GET /api/admin/me/permissions（或者把 permissions 一并返回）
前端拿到 Set<string> 的权限 key，比如：
users:read, users:create, ...
然后：
菜单项显示：需要 xxx:read
新增按钮：需要 xxx:create
删除按钮：需要 xxx:delete
这样你新增角色/调整权限，前端无需改逻辑。

你现在前端已经有 permissions 页面，但后端还没实现对应 API。等你后端把权限管理 API 做出来，前端这块就能真正闭环。

5) 结合你当前项目的一个“落地蓝图”（最少改动版本）
按你目前后端已有模块（users/roles/apps/reg_codes/devices），可以这样定义权限：

users
users:read/create/update/delete
roles
roles:read/create/update/delete
apps
apps:read/create/update/delete
reg_codes
reg_codes:read/create/update/delete
devices
devices:read
admin 默认拥有全部；user 可能只拥有部分（比如只读 apps + 读写 reg_codes 之类）。
