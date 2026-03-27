#!/bin/bash
# VoxNote 전체 테스트 실행 스크립트
# Usage: ./scripts/test-all.sh [--all|--unit|--server|--frontend|--lint]

set -e
cd "$(dirname "$0")/.."

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

run_unit() {
    echo -e "${YELLOW}[1/4] Rust Core Tests${NC}"
    cargo test -p voxnote-core --no-default-features 2>&1
}

run_server() {
    echo -e "${YELLOW}[2/4] Rust Server Tests${NC}"
    cargo test -p voxnote-server 2>&1
}

run_frontend() {
    echo -e "${YELLOW}[3/4] Frontend Tests${NC}"
    cd frontend && pnpm vitest run 2>&1 && cd ..
}

run_lint() {
    echo -e "${YELLOW}[4/4] Lint & Format${NC}"
    cargo clippy -p voxnote-core --no-default-features -- -D warnings 2>&1
    cargo fmt --check 2>&1
    cd frontend && npx tsc --noEmit 2>&1 && cd ..
}

case "${1:-}" in
    --unit)    run_unit ;;
    --server)  run_server ;;
    --frontend) run_frontend ;;
    --lint)    run_lint ;;
    --all|"")
        run_unit
        run_server
        run_frontend
        echo ""
        echo -e "${GREEN}=== ALL TESTS PASSED ===${NC}"
        ;;
    *)
        echo "Usage: $0 [--all|--unit|--server|--frontend|--lint]"
        exit 1
        ;;
esac
