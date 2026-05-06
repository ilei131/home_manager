#!/usr/bin/env bash
set -euo pipefail

# ============================================
# Home Manager 本地开发启动脚本
# ============================================

PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
FRONTEND_DIR="$PROJECT_ROOT/frontend"
BACKEND_DIR="$PROJECT_ROOT/backend"

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info()  { echo -e "${GREEN}[INFO]${NC}  $*"; }
log_warn()  { echo -e "${YELLOW}[WARN]${NC}  $*"; }
log_error() { echo -e "${RED}[ERROR]${NC} $*"; }

# ============================================
# 1. 检查 PostgreSQL 是否运行
# ============================================
check_postgres() {
    log_info "检查 PostgreSQL 连接..."

    # 尝试通过 docker compose dev 配置启动数据库
    if ! docker compose -f "$PROJECT_ROOT/docker-compose.dev.yml" ps --status running 2>/dev/null | grep -q "db"; then
        log_warn "PostgreSQL 未运行，正在启动..."
        docker compose -f "$PROJECT_ROOT/docker-compose.dev.yml" up -d

        # 等待数据库就绪
        log_info "等待数据库就绪..."
        for i in $(seq 1 30); do
            if docker compose -f "$PROJECT_ROOT/docker-compose.dev.yml" exec -T db pg_isready -U postgres >/dev/null 2>&1; then
                log_info "数据库已就绪"
                return 0
            fi
            sleep 1
        done
        log_error "数据库启动超时，请手动检查"
        exit 1
    fi

    log_info "PostgreSQL 已在运行"
}

# ============================================
# 2. 检查依赖
# ============================================
check_dependencies() {
    # 检查 Rust
    if ! command -v cargo &>/dev/null; then
        log_error "未找到 cargo，请先安装 Rust: https://rustup.rs/"
        exit 1
    fi
    log_info "Rust $(cargo --version)"

    # 检查 Node.js
    if ! command -v node &>/dev/null; then
        log_error "未找到 node，请先安装 Node.js 20+: https://nodejs.org/"
        exit 1
    fi
    log_info "Node.js $(node --version)"

    # 检查 npm
    if ! command -v npm &>/dev/null; then
        log_error "未找到 npm"
        exit 1
    fi
    log_info "npm $(npm --version)"
}

# ============================================
# 3. 启动后端
# ============================================
start_backend() {
    log_info "启动后端服务..."
    cd "$BACKEND_DIR"

    # 检查 .env 文件
    if [ ! -f .env ]; then
        if [ -f .env.example ]; then
            cp .env.example .env
            log_warn "已从 .env.example 创建 .env 文件，请检查配置"
        else
            cat > .env <<'EOF'
DATABASE_URL=postgres://postgres:postgres@localhost:5432/home_manager
JWT_SECRET=dev-secret-change-in-production
SERVER_HOST=0.0.0.0
SERVER_PORT=3000
RUST_LOG=info
EOF
            log_warn "已创建默认 .env 文件"
        fi
    fi

    # 运行数据库迁移（如果存在 sqlx-cli）
    if command -v sqlx &>/dev/null && [ -d migrations ]; then
        log_info "运行数据库迁移..."
        sqlx migrate run 2>/dev/null || log_warn "数据库迁移跳过（可能已执行）"
    fi

    cargo run &
    BACKEND_PID=$!
    echo "$BACKEND_PID" > "$PROJECT_ROOT/.backend.pid"
    log_info "后端服务已启动 (PID: $BACKEND_PID) - http://localhost:3000"
}

# ============================================
# 4. 启动前端
# ============================================
start_frontend() {
    log_info "启动前端开发服务器..."
    cd "$FRONTEND_DIR"

    # 安装依赖
    if [ ! -d node_modules ]; then
        log_info "安装前端依赖..."
        npm install
    fi

    npm run dev &
    FRONTEND_PID=$!
    echo "$FRONTEND_PID" > "$PROJECT_ROOT/.frontend.pid"
    log_info "前端服务已启动 (PID: $FRONTEND_PID)"
}

# ============================================
# 5. 清理函数
# ============================================
cleanup() {
    log_info "正在停止服务..."

    if [ -f "$PROJECT_ROOT/.backend.pid" ]; then
        kill "$(cat "$PROJECT_ROOT/.backend.pid")" 2>/dev/null || true
        rm -f "$PROJECT_ROOT/.backend.pid"
    fi

    if [ -f "$PROJECT_ROOT/.frontend.pid" ]; then
        kill "$(cat "$PROJECT_ROOT/.frontend.pid")" 2>/dev/null || true
        rm -f "$PROJECT_ROOT/.frontend.pid"
    fi

    log_info "服务已停止"
}

trap cleanup EXIT INT TERM

# ============================================
# 主流程
# ============================================
main() {
    echo "========================================="
    echo "  Home Manager - 本地开发环境"
    echo "========================================="
    echo ""

    check_dependencies
    check_postgres

    echo ""
    start_backend

    # 等待后端启动
    log_info "等待后端服务启动..."
    for i in $(seq 1 30); do
        if curl -s http://localhost:3000/api/health >/dev/null 2>&1; then
            break
        fi
        sleep 1
    done

    start_frontend

    echo ""
    echo "========================================="
    log_info "所有服务已启动！"
    echo "========================================="
    echo "  后端 API:  http://localhost:3000"
    echo "  前端页面:  http://localhost:5173"
    echo "  数据库:    localhost:5432"
    echo "========================================="
    echo ""
    log_info "按 Ctrl+C 停止所有服务"

    # 等待子进程
    wait
}

main "$@"
