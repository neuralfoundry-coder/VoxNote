#!/bin/bash
# VoxNote 개발 디버그 실행 스크립트
# - 프론트엔드(Vite HMR) + Rust 백엔드 동시 실행
# - 프론트엔드 변경: 즉시 HMR 반영
# - Rust 변경: 자동 재컴파일 + 앱 재시작
# - 기본 로그 레벨: DEBUG (RUST_LOG 환경변수로 오버라이드 가능)
#
# Usage:
#   ./scripts/dev.sh              # 기본 (전체 DEBUG)
#   ./scripts/dev.sh --trace      # 전체 TRACE
#   ./scripts/dev.sh --quiet      # voxnote만 DEBUG, 나머지 WARN

set -e
cd "$(dirname "$0")/.."

YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

# 로그 레벨 설정
case "${1:-}" in
    --trace)
        export RUST_LOG="${RUST_LOG:-trace}"
        ;;
    --quiet)
        export RUST_LOG="${RUST_LOG:-voxnote=debug,warn}"
        ;;
    --help|-h)
        echo "Usage: $0 [--trace|--quiet]"
        echo ""
        echo "Options:"
        echo "  (default)   전체 DEBUG 로그"
        echo "  --trace     전체 TRACE 로그 (매우 상세)"
        echo "  --quiet     voxnote=debug, 나머지=warn"
        echo ""
        echo "환경변수 오버라이드:"
        echo "  RUST_LOG=info ./scripts/dev.sh"
        exit 0
        ;;
    "")
        export RUST_LOG="${RUST_LOG:-debug}"
        ;;
    *)
        echo "Unknown option: $1 (use --help)"
        exit 1
        ;;
esac

echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${YELLOW} VoxNote Dev Mode${NC}"
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e " RUST_LOG  = ${RUST_LOG}"
echo -e " Frontend  = http://localhost:1420 (Vite HMR)"
echo -e " Rust 변경 = 자동 재컴파일"
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# 기존 포트 점유 프로세스 정리
cleanup_port() {
    local port=$1
    local pids
    pids=$(lsof -ti :"$port" 2>/dev/null || true)
    if [ -n "$pids" ]; then
        echo -e "${YELLOW} Port $port 점유 프로세스 종료 중... (PID: $pids)${NC}"
        echo "$pids" | xargs kill -9 2>/dev/null || true
        sleep 1
    fi
}

cleanup_port 1420

# 종료 시 정리
trap 'echo ""; echo -e "${YELLOW} Shutting down...${NC}"; cleanup_port 1420; wait 2>/dev/null' EXIT INT TERM

# cargo tauri dev가 Vite도 함께 시작 (tauri.conf.json의 beforeDevCommand)
echo -e " Starting Tauri dev (frontend + backend)..."
cargo tauri dev
