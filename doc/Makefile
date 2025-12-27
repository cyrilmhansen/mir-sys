# Makefile for Sphinx documentation

SPHINXOPTS    ?=
SPHINXBUILD   ?= sphinx-build
SOURCEDIR     = .
BUILDDIR      = _build

help:
	@$(SPHINXBUILD) -M help "$(SOURCEDIR)" "$(BUILDDIR)" $(SPHINXOPTS) $(O)

.PHONY: help Makefile doxygen html pdf tectonic

# 0. Clean
clean:
	@rm -rf $(BUILDDIR) _static/diagrams generated_diagrams.rst

# 1. Run Doxygen to generate XML
doxygen:
	doxygen Doxyfile

# 1b. Copy Diagrams & Import Doxygen Graphs
copy-diagrams:
	@mkdir -p _static/diagrams
	@cp ../mir/*.svg _static/diagrams/ || true
	@cp ../mir/c2mir/*.svg _static/diagrams/ || true
	@python3 scripts/import_diagrams.py

# 1c. Convert SVGs to PDF for LaTeX
convert-svgs: copy-diagrams
	@bash scripts/convert_svgs.sh

# 2. Build HTML (Narrative + Code)
html: doxygen copy-diagrams
	@$(SPHINXBUILD) -M html "$(SOURCEDIR)" "$(BUILDDIR)" $(SPHINXOPTS) $(O)

# 3. Build PDF (Requires LaTeX installed)
pdf: doxygen convert-svgs
	@$(SPHINXBUILD) -M latexpdf "$(SOURCEDIR)" "$(BUILDDIR)" $(SPHINXOPTS) $(O)
	@bash scripts/normalize_pdfs.sh "$(BUILDDIR)/latex"

# 4. Build PDF using Tectonic (Reproducible)
tectonic: doxygen convert-svgs
	@$(SPHINXBUILD) -M latex "$(SOURCEDIR)" "$(BUILDDIR)" $(SPHINXOPTS) $(O)
	@bash scripts/normalize_pdfs.sh "$(BUILDDIR)/latex"
	@cd $(BUILDDIR)/latex && tectonic MIR_Technical_Manual.tex
