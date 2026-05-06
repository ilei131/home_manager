# Home Manager

一个基于 Rust + React 的家庭事务管理系统，帮助家庭成员高效管理日常事务、任务分配和财务记录。

## 功能特性

- **用户管理**：注册、登录、JWT 认证、角色权限控制
- **任务管理**：创建、分配、跟踪家庭任务，支持优先级和截止日期
- **财务记录**：收支记录、分类统计、月度报表
- **家庭成员**：多成员管理，角色分配（管理员/普通成员）
- **通知提醒**：任务到期提醒、新任务通知
- **数据统计**：可视化仪表盘，任务完成率、财务趋势图表

## 技术栈

| 层级 | 技术 |
|------|------|
| 后端 | Rust (Actix-web / Axum), SQLx, PostgreSQL, JWT |
| 前端 | React 18, TypeScript, Vite, Tailwind CSS |
| 数据库 | PostgreSQL 16 |
| 部署 | Docker, Docker Compose, Nginx |

## 项目结构

```
home-manager/
├── backend/                  # Rust 后端
│   ├── src/
│   │   ├── main.rs          # 入口文件
│   │   ├── handlers/        # API 处理器
│   │   ├── models/          # 数据模型
│   │   ├── services/        # 业务逻辑
│   │   └── middleware/      # 中间件（认证等）
│   ├── migrations/           # 数据库迁移
│   ├── Cargo.toml
│   └── Dockerfile
├── frontend/                 # React 前端
│   ├── src/
│   │   ├── components/      # 组件
│   │   ├── pages/           # 页面
│   │   ├── hooks/           # 自定义 Hooks
│   │   ├── services/        # API 调用
│   │   └── utils/           # 工具函数
│   ├── package.json
│   ├── nginx.conf
│   └── Dockerfile
├── scripts/
│   └── dev.sh               # 本地开发启动脚本
├── docker-compose.yml        # 生产环境编排
├── docker-compose.dev.yml    # 开发环境编排（仅数据库）
├── .gitignore
└── README.md
```

## 快速开始

### Docker 一键启动（推荐）

```bash
# 克隆项目
git clone <repository-url>
cd home-manager

# 启动所有服务（数据库 + 后端 + 前端）
docker compose up -d

# 查看日志
docker compose logs -f
```

启动后访问：
- 前端页面：http://localhost
- 后端 API：http://localhost:3000

### 本地开发

#### 前置要求

- Rust 1.75+
- Node.js 20+
- PostgreSQL 16+
- npm 或 pnpm

#### 步骤

```bash
# 1. 启动数据库
docker compose -f docker-compose.dev.yml up -d

# 2. 启动后端
cd backend
cp .env.example .env       # 配置环境变量
cargo run                  # 启动后端服务 (http://localhost:3000)

# 3. 启动前端（新终端）
cd frontend
npm install
npm run dev                # 启动前端开发服务器 (http://localhost:5173)

# 或者使用一键启动脚本
./scripts/dev.sh
```

## 环境变量说明

| 变量名 | 说明 | 默认值 |
|--------|------|--------|
| `DATABASE_URL` | PostgreSQL 连接字符串 | `postgres://postgres:postgres@localhost:5432/home_manager` |
| `JWT_SECRET` | JWT 签名密钥 | `change-this-secret-in-production` |
| `SERVER_HOST` | 服务监听地址 | `0.0.0.0` |
| `SERVER_PORT` | 服务监听端口 | `3000` |
| `RUST_LOG` | 日志级别 | `info` |

## API 接口文档

### 认证

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/auth/register` | 用户注册 |
| POST | `/api/auth/login` | 用户登录 |
| GET | `/api/auth/me` | 获取当前用户信息 |

### 用户管理

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/users` | 获取用户列表 |
| GET | `/api/users/:id` | 获取用户详情 |
| PUT | `/api/users/:id` | 更新用户信息 |
| DELETE | `/api/users/:id` | 删除用户 |

### 任务管理

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/tasks` | 获取任务列表 |
| POST | `/api/tasks` | 创建任务 |
| GET | `/api/tasks/:id` | 获取任务详情 |
| PUT | `/api/tasks/:id` | 更新任务 |
| DELETE | `/api/tasks/:id` | 删除任务 |
| PATCH | `/api/tasks/:id/complete` | 标记任务完成 |

### 财务记录

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/finance/records` | 获取财务记录列表 |
| POST | `/api/finance/records` | 创建财务记录 |
| GET | `/api/finance/summary` | 获取财务汇总 |
| GET | `/api/finance/categories` | 获取分类列表 |

### 统计数据

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/stats/dashboard` | 获取仪表盘数据 |

## 默认管理员账号

首次启动时系统会自动创建管理员账号：

- **用户名**：`admin`
- **密码**：`admin123`

> **重要**：请在首次登录后立即修改默认密码。

## 截图

<!-- TODO: 补充项目截图 -->
<!-- - 登录页面 -->
<!-- - 仪表盘 -->
<!-- - 任务管理 -->
<!-- - 财务记录 -->

## License

MIT
