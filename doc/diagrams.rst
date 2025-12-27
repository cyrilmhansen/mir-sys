Diagrams
========

This appendix collects the automatically generated diagrams and hand-written sketches.

Doxygen (automatic)
-------------------
The Doxygen HTML output now includes class diagrams, call graphs, and caller graphs for public APIs. Build the docs and open ``_build/doxygen-html/index.html``; graph-rich pages include inline SVGs you can zoom and click through.

Sphinx Mermaid (manual)
-----------------------
Use Mermaid blocks to document control flow or pipelines directly in the narrative. Install ``sphinxcontrib-mermaid`` (already listed in ``requirements.txt``) and then write::

   .. mermaid::
      flowchart TD
        parse[Parse C] --> mir[Emit MIR]
        mir --> ssa[SSA + Opts]
        ssa --> ra[Register Allocate]
        ra --> mach[Machine Code]
        mach --> exec[JIT / Interp]

Graphviz (manual)
-----------------
The built-in ``graphviz`` directive remains available if you prefer DOT syntax.
