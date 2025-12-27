#!/bin/bash
# Convert all SVGs in _static/diagrams to PDF for LaTeX compatibility

DIAG_DIR="_static/diagrams"
mkdir -p "$DIAG_DIR"

# Convert mir-gen.svg, mir3.svg, etc.
find "$DIAG_DIR" -name "*.svg" | while read svg_file; do
    pdf_file="${svg_file%.svg}.pdf"
    echo "Converting $svg_file to $pdf_file..."
    rsvg-convert -f pdf -o "$pdf_file" "$svg_file"
done
