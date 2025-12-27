# Makefile for Sphinx documentation

SPHINXOPTS    ?=
SPHINXBUILD   ?= sphinx-build
SOURCEDIR     = .
BUILDDIR      = _build

help:
	@$(SPHINXBUILD) -M help "$(SOURCEDIR)" "$(BUILDDIR)" $(SPHINXOPTS) $(O)

.PHONY: help Makefile doxygen html pdf tectonic

# 1. Run Doxygen to generate XML
doxygen:
	doxygen Doxyfile

# 2. Build HTML (Narrative + Code)
html: doxygen
	@$(SPHINXBUILD) -M html "$(SOURCEDIR)" "$(BUILDDIR)" $(SPHINXOPTS) $(O)

# 3. Build PDF (Requires LaTeX installed)
# We use -M latexpdf which generates the .tex files and runs pdflatex
pdf: doxygen
	@TEXMFVAR="$(abspath $(BUILDDIR)/texmf-var)" \
	  $(SPHINXBUILD) -M latexpdf "$(SOURCEDIR)" "$(BUILDDIR)" $(SPHINXOPTS) $(O)

# 4. Build via Tectonic (requires `tectonic` binary installed)
tectonic: doxygen
	@$(SPHINXBUILD) -b latex "$(SOURCEDIR)" "$(BUILDDIR)/latex" $(SPHINXOPTS) $(O)
	@cd "$(BUILDDIR)/latex" && tectonic "MIR_Technical_Manual.tex" --outdir . --keep-logs
	@echo "Tectonic PDF written to $(BUILDDIR)/latex/MIR_Technical_Manual.pdf"
