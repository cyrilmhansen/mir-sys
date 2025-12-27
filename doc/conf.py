import os
import sys

# -- Project information -----------------------------------------------------
project = 'MIR'
copyright = '2025, Vladimir Makarov'
author = 'Cyril M. Hansen'
release = '0.2'

# -- General configuration ---------------------------------------------------
extensions = [
    'sphinx_rtd_theme',
    'sphinx.ext.graphviz',
]

# Optional: Mermaid diagrams via sphinxcontrib-mermaid (skip gracefully if missing)
try:
    import sphinxcontrib.mermaid  # type: ignore
    extensions.append('sphinxcontrib.mermaid')
except ImportError:
    mermaid_missing = True
else:
    mermaid_missing = False

templates_path = ['_templates']
exclude_patterns = ['_build', 'Thumbs.db', '.DS_Store']

# -- Options for HTML output -------------------------------------------------
html_theme = 'sphinx_rtd_theme'
html_static_path = ['_static']

html_last_updated_fmt = '%Y-%m-%d'

# Mermaid rendering (via sphinxcontrib-mermaid)
mermaid_output_format = 'raw'

# -- Options for LaTeX/PDF output ---------------------------------------------
latex_elements = {
    # The paper size ('letterpaper' or 'a4paper').
    'papersize': 'a4paper',

    # The font size ('10pt', '11pt' or '12pt').
    'pointsize': '10pt',

    # Force base LaTeX fonts to avoid missing tgtermes/tgheros packages.
    'fontpkg': r'',
    'fontenc': r'',

    # Literal blocks (code) often overflow margins in PDFs. 
    # This snippet creates a smaller font environment for code blocks.
    'preamble': r'''
        \sloppy
        \fvset{fontsize=\small}
        \RecustomVerbatimEnvironment{Verbatim}{Verbatim}{fontsize=\small}
    ''',
}

# Grouping the document tree into LaTeX files. List of tuples:
# (source start file, target name, title, author, documentclass [howto, manual, or own class]).
latex_documents = [
    ('index', 'MIR_Technical_Manual.tex', 'The Anatomy of a C JIT',
     'Cyril M. Hansen', 'manual'),
]

# Use Tectonic for reproducible PDF builds with bundled fonts.
latex_engine = 'pdflatex'
