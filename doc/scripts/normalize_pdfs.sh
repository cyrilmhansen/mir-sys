#!/bin/bash
# Down-convert Graphviz PDFs to PDF 1.5 for TeX compatibility.

set -euo pipefail

OUT_DIR="${1:-_build/latex}"

if ! command -v gs >/dev/null 2>&1; then
    exit 0
fi

if [ ! -d "$OUT_DIR" ]; then
    exit 0
fi

find "$OUT_DIR" -name "graphviz-*.pdf" | while read -r pdf_file; do
    tmp_pdf="${pdf_file}.tmp"
    gs -sDEVICE=pdfwrite -dCompatibilityLevel=1.5 -dNOPAUSE -dBATCH -dQUIET \
        -sOutputFile="$tmp_pdf" "$pdf_file"
    mv "$tmp_pdf" "$pdf_file"
done
