#!/bin/bash
# Convert all SVGs in _static/diagrams to PDF for LaTeX compatibility

DIAG_DIR="_static/diagrams"
mkdir -p "$DIAG_DIR"

# Convert mir-gen.svg, mir3.svg, etc.
find "$DIAG_DIR" -name "*.svg" | while read svg_file; do
    pdf_file="${svg_file%.svg}.pdf"
    echo "Converting $svg_file to $pdf_file..."
    rsvg-convert -f pdf -o "$pdf_file" "$svg_file"
    # Down-convert to PDF 1.5 to match the TeX output version and avoid warnings.
    if command -v gs >/dev/null 2>&1; then
        tmp_pdf="${pdf_file}.tmp"
        gs -sDEVICE=pdfwrite -dCompatibilityLevel=1.5 -dNOPAUSE -dBATCH -dQUIET \
            -sOutputFile="$tmp_pdf" "$pdf_file"
        mv "$tmp_pdf" "$pdf_file"
    fi
done
