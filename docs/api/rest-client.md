# REST Client 使用指南

## 🚀 快速开始

### 1. 安装插件

在 VSCode 中安装 REST Client 插件：

1. 打开 VSCode 扩展面板（`Ctrl+Shift+X` 或 `Cmd+Shift+X`）
2. 搜索 "REST Client"
3. 安装 "REST Client by Huachao Mao"

### 2. 使用接口测试文件

项目已经准备好了接口测试文件：`docs/api.http`

#### 基本使用步骤：

1. **打开测试文件**：在 VSCode 中打开 `docs/api.http`
2. **发送请求**：点击请求上方的 "Send Request" 按钮
3. **查看响应**：响应会在新标签页中显示

#### 示例截图说明：

```http
### 1. 健康检查
GET {{baseUrl}}/health
Accept: application/json
```

点击 `GET {{baseUrl}}/health` 上方的 **"Send Request"** 链接即可发送请求。

## 🔧 功能特性

### 环境变量

文件顶部定义了环境变量：

```http
@baseUrl = http://localhost:3001
@apiUrl = {{baseUrl}}/api
```

- `{{baseUrl}}` - 服务器基础地址
- `{{apiUrl}}` - API 基础路径

### 请求分组

使用 `###` 分隔不同的请求：

```http
### 用户管理 - 获取用户列表
GET {{apiUrl}}/sys/user
Accept: application/json

### 角色管理 - 获取角色列表
GET {{apiUrl}}/sys/role
Accept: application/json
```

### 注释说明

- 使用 `#` 开头的行作为注释
- 使用 `###` 作为请求分隔符和标题

## 📋 当前可用接口

| 接口名称 | 方法 | 路径            | 说明          |
| -------- | ---- | --------------- | ------------- |
| 健康检查 | GET  | `/health`       | 检查服务状态  |
| 根路径   | GET  | `/`             | 获取 API 信息 |
| 用户列表 | GET  | `/api/sys/user` | 获取用户列表  |
| 角色列表 | GET  | `/api/sys/role` | 获取角色列表  |
| 菜单列表 | GET  | `/api/sys/menu` | 获取菜单列表  |
| 字典列表 | GET  | `/api/sys/dict` | 获取字典列表  |
| 日志列表 | GET  | `/api/sys/log`  | 获取日志列表  |

## 🛠️ 高级用法

### 1. 设置请求头

```http
GET {{apiUrl}}/sys/user
Accept: application/json
Authorization: Bearer your-jwt-token
Content-Type: application/json
```

### 2. POST 请求示例

```http
POST {{apiUrl}}/sys/user
Content-Type: application/json

{
  "username": "testuser",
  "email": "test@example.com",
  "password": "password123"
}
```

### 3. 批量测试

在同一个文件中定义多个请求，可以逐个测试或批量执行。

### 4. 保存响应

右键点击响应内容可以：

- 复制响应
- 保存为文件
- 格式化 JSON

## 🔍 调试技巧

### 1. 查看请求详情

REST Client 会显示：

- 完整的请求 URL
- 请求头信息
- 响应状态码
- 响应头
- 响应体

### 2. 错误排查

常见问题：

- **连接失败**：检查后端服务是否启动
- **404 错误**：检查 API 路径是否正确
- **500 错误**：检查后端日志输出

### 3. 环境切换

可以在 `.vscode/settings.json` 中配置多个环境：

```json
{
  "rest-client.environmentVariables": {
    "development": {
      "baseUrl": "http://localhost:3001"
    },
    "production": {
      "baseUrl": "https://your-domain.com"
    }
  }
}
```

## 📝 最佳实践

1. **路径规范**：统一使用不带尾部斜杠的路径（如 `/api/sys/user`）
2. **注释清晰**：为每个请求添加清晰的说明
3. **分组管理**：使用 `###` 对相关接口进行分组
4. **环境变量**：使用变量避免硬编码 URL
5. **版本控制**：将 `.http` 文件提交到 Git，方便团队共享

## 🆚 与其他工具对比

| 特性     | REST Client    | Postman     | Hoppscotch  |
| -------- | -------------- | ----------- | ----------- |
| 集成度   | ✅ VSCode 内置 | ❌ 独立应用 | ❌ 网页工具 |
| 版本控制 | ✅ 文件形式    | ❌ 需要导出 | ❌ 需要导出 |
| 团队协作 | ✅ Git 共享    | ✅ 云端同步 | ❌ 本地存储 |
| 学习成本 | ✅ 简单        | ❌ 功能复杂 | ✅ 简单     |
| 离线使用 | ✅ 完全离线    | ❌ 需要登录 | ❌ 需要网络 |

对于个人开发和项目初期，REST Client 是最佳选择！
