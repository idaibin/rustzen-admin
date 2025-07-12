# 部署

## 部署 rust

### 构建

方式一：`just build-backend`
方式二：采用 github 的 action 事件

### 拷贝文件到服务器

放到 `/opt/rustzen-admin` 目录下
设置权限 `sudo chmod +x /opt/rustzen-admin/rustzen-admin`

### 设置开机启动和统一管理

vi /opt/rustzen-admin/rustzen-admin.service

```bash
[Unit]
Description=Rust App Service
After=network.target

[Service]
Type=simple
ExecStart=/opt/rustzen-admin/rustzen-admin
Restart=always
RestartSec=5
User=root
Group=root
WorkingDirectory=/opt/rustzen-admin/
Environment=RUST_LOG=info
Environment=RUST_BACKTRACE=1

# 安全设置
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ReadWritePaths=/opt/rustzen-admin/

[Install]
WantedBy=multi-user.target
```

```bash
# 创建软链接（首次）
sudo ln -s /opt/rustzen-admin/rustzen-admin.service /etc/systemd/system/rustzen-admin.service

# 重载 systemd
sudo systemctl daemon-reexec

# 启用和启动服务
sudo systemctl enable rustzen-admin
sudo systemctl start rustzen-admin

# 查看状态
systemctl status rustzen-admin

# 查看日志
sudo journalctl -u rustzen-admin -n 20
```
