#!/usr/bin/env bash
# VoxNote 아키텍처 다이어그램 생성 스크립트
# Mermaid → PNG 변환 및 MD → PDF 변환
#
# 사전 요구사항:
#   brew install mermaid-cli pandoc
#   또는: npm install -g @mermaid-js/mermaid-cli
#
# 사용법:
#   ./generate-diagrams.sh [--png] [--pdf] [--all]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DIAGRAMS_DIR="$SCRIPT_DIR/diagrams"
PDF_DIR="$SCRIPT_DIR/pdf"

DO_PNG=false
DO_PDF=false

# 인자 파싱
for arg in "$@"; do
  case $arg in
    --png) DO_PNG=true ;;
    --pdf) DO_PDF=true ;;
    --all) DO_PNG=true; DO_PDF=true ;;
    *) echo "사용법: $0 [--png] [--pdf] [--all]"; exit 1 ;;
  esac
done

# 기본값: 전부
if ! $DO_PNG && ! $DO_PDF; then
  DO_PNG=true
  DO_PDF=true
fi

# ─── PNG 생성 (Mermaid → PNG) ───
if $DO_PNG; then
  echo "═══ Mermaid → PNG 변환 시작 ═══"
  mkdir -p "$DIAGRAMS_DIR"

  if ! command -v mmdc &>/dev/null; then
    echo "❌ mmdc(mermaid-cli) 미설치. 설치: npm install -g @mermaid-js/mermaid-cli"
    exit 1
  fi

  DIAGRAM_COUNT=0

  for md_file in "$SCRIPT_DIR"/*.md; do
    [ -f "$md_file" ] || continue
    basename=$(basename "$md_file" .md)

    # Mermaid 코드 블록 추출 (```mermaid ... ```)
    idx=0
    in_mermaid=false
    tmp_file=""

    while IFS= read -r line; do
      if [[ "$line" =~ ^\`\`\`mermaid ]]; then
        in_mermaid=true
        idx=$((idx + 1))
        tmp_file=$(mktemp /tmp/mermaid-XXXXXX.mmd)
        continue
      fi

      if $in_mermaid && [[ "$line" =~ ^\`\`\` ]]; then
        in_mermaid=false
        out_png="$DIAGRAMS_DIR/${basename}-diagram-$(printf '%02d' $idx).png"

        if mmdc -i "$tmp_file" -o "$out_png" -t neutral -w 1600 -H 1200 --scale 2 2>/dev/null; then
          echo "  ✅ $out_png"
          DIAGRAM_COUNT=$((DIAGRAM_COUNT + 1))
        else
          echo "  ⚠️  변환 실패: ${basename} 다이어그램 #${idx} (문법 오류 가능)"
        fi
        rm -f "$tmp_file"
        continue
      fi

      if $in_mermaid; then
        echo "$line" >> "$tmp_file"
      fi
    done < "$md_file"
  done

  echo "═══ PNG 변환 완료: ${DIAGRAM_COUNT}개 생성 ═══"
  echo ""
fi

# ─── PDF 생성 (MD → PDF) ───
if $DO_PDF; then
  echo "═══ Markdown → PDF 변환 시작 ═══"
  mkdir -p "$PDF_DIR"

  if ! command -v pandoc &>/dev/null; then
    echo "❌ pandoc 미설치. 설치: brew install pandoc"
    exit 1
  fi

  PDF_COUNT=0

  for md_file in "$SCRIPT_DIR"/*.md; do
    [ -f "$md_file" ] || continue
    basename=$(basename "$md_file" .md)
    out_pdf="$PDF_DIR/${basename}.pdf"

    # pandoc으로 PDF 생성 (한국어 지원을 위해 xelatex + 폰트 설정)
    if pandoc "$md_file" \
      -o "$out_pdf" \
      --pdf-engine=xelatex \
      -V mainfont="AppleGothic" \
      -V monofont="Menlo" \
      -V geometry:margin=2.5cm \
      -V fontsize=11pt \
      --toc \
      --toc-depth=3 \
      -V colorlinks=true \
      2>/dev/null; then
      echo "  ✅ $out_pdf"
      PDF_COUNT=$((PDF_COUNT + 1))
    else
      # xelatex 실패 시 weasyprint 또는 wkhtmltopdf 시도
      echo "  ⚠️  PDF 변환 실패: ${basename} (xelatex 필요. brew install basictex)"
    fi
  done

  echo "═══ PDF 변환 완료: ${PDF_COUNT}개 생성 ═══"
fi

echo ""
echo "결과물:"
[ -d "$DIAGRAMS_DIR" ] && echo "  PNG: $DIAGRAMS_DIR/ ($(ls "$DIAGRAMS_DIR"/*.png 2>/dev/null | wc -l | tr -d ' ')개)"
[ -d "$PDF_DIR" ] && echo "  PDF: $PDF_DIR/ ($(ls "$PDF_DIR"/*.pdf 2>/dev/null | wc -l | tr -d ' ')개)"
