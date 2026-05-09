# nginx 配置
要在同一服务器上部署小管家网站（子域名 `xgj.upshare.fun`），你需要添加一个新的 Nginx server block 配置。以下是完整的配置方案：

## 步骤 1：创建前端代码目录

首先，在服务器上创建小管家前端的部署目录：

```bash
# 创建小管家前端目录
sudo mkdir -p /var/www/home_manager
```

然后将小管家前端构建后的代码上传到 `/var/www/home_manager` 目录。
```bash
# 修改权限
sudo chown -R www-data:www-data ./home_manager/
```
## 步骤 2：修改 Nginx 配置

在现有的 Nginx 配置文件中添加以下 server block：

```nginx
server {
    listen 80;
    server_name xgj.upshare.fun;
    
    # 小管家前端目录
    root /var/www/home_manager;
    index index.html;

    # 小管家后端 API 代理（假设后端运行在 3000 端口）
    location /api/ {
        proxy_pass http://127.0.0.1:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
    }

    # 前端路由（支持 React Router）
    location / {
        try_files $uri $uri/ /index.html;
    }

    # 静态文件缓存
    location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg|woff|woff2|ttf|eot)$ {
        expires 1y;
        add_header Cache-Control "public, immutable";
    }
}
```
## 步骤 4：重启 Nginx 并配置 SSL（可选但推荐）

```bash
# 测试配置是否正确
sudo nginx -t

# 重启 Nginx
sudo systemctl restart nginx
```
## systemctl 配置
将小管家后端设置为系统服务非常简单，只需创建一个 systemd 服务配置文件。以下是详细步骤：

## 步骤 1：确保后端已构建

首先，确保你已经在服务器上构建了 Rust 后端项目：

```bash
# 进入项目目录
cd /path/to/home_manager/backend

# 构建生产版本
cargo build --release

# 确认可执行文件存在
ls -la target/release/home-manager-backend
```

## 步骤 2：创建 systemd 服务文件

创建一个 `.service` 文件：

```bash
sudo nano /etc/systemd/system/home-manager.service
```

粘贴以下内容（根据你的实际路径修改）：

```ini
[Unit]
Description=Home Manager Backend Service
After=network.target
After=postgresql.service  # 如果使用 PostgreSQL，确保数据库先启动

[Service]
User=www-data           # 运行服务的用户（推荐使用 www-data 或创建专用用户）
Group=www-data          # 用户组
WorkingDirectory=/path/to/home_manager/backend  # 项目根目录
ExecStart=/path/to/home_manager/backend/target/release/home-manager-backend  # 可执行文件路径
Restart=always          # 服务崩溃时自动重启
RestartSec=5           # 重启间隔（秒）
Environment=RUST_LOG=info  # 设置日志级别
EnvironmentFile=/path/to/home_manager/backend/.env
StandardOutput=journal+console
StandardError=journal+console
SyslogIdentifier=home-manager-backend

[Install]
WantedBy=multi-user.target
```

**请务必修改以下路径**：
- `WorkingDirectory`：你的后端项目目录
- `ExecStart`：可执行文件的完整路径
- `EnvironmentFile`：环境变量文件的完整路径

## 步骤 3：启用并启动服务

```bash
# 重新加载 systemd 配置
sudo systemctl daemon-reload

# 设置开机自启
sudo systemctl enable home-manager

# 启动服务
sudo systemctl start home-manager

# 查看服务状态
sudo systemctl status home-manager
```

## 步骤 4：常用命令

```bash
# 启动服务
sudo systemctl start home-manager

# 停止服务
sudo systemctl stop home-manager

# 重启服务
sudo systemctl restart home-manager

# 查看服务状态
sudo systemctl status home-manager

# 查看日志（实时）
sudo journalctl -u home-manager -f

# 查看最近日志
sudo journalctl -u home-manager --since "10 minutes ago"
```

## 步骤 5：确保数据库连接正常

如果你的后端使用 PostgreSQL，确保：
1. PostgreSQL 服务已启动：`sudo systemctl start postgresql`
2. 数据库配置正确（数据库名称、用户名、密码）
3. 数据库已创建并初始化

## 配置示例

假设你的项目路径是 `/var/www/home_manager/backend`，配置文件如下：

```ini
[Unit]
Description=Home Manager Backend Service
After=network.target
After=postgresql.service

[Service]
User=www-data
Group=www-data
WorkingDirectory=/var/www/home_manager/backend
ExecStart=/var/www/home_manager/backend/target/release/home-manager-backend
Restart=always
RestartSec=5
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
```

## 验证服务是否正常运行

1. 检查服务状态：
   ```bash
   sudo systemctl status home-manager
   ```

2. 查看日志确认没有错误：
   ```bash
   sudo journalctl -u home-manager -f
   ```

3. 测试 API 是否正常：
   ```bash
   curl http://localhost:3000/api/health
   ```

这样配置后，小管家后端服务就会在系统启动时自动运行，并在崩溃时自动重启。