Diagrams
========

This appendix collects the architectural diagrams found in the codebase.

Project Overview
----------------

**Current State of MIR**

.. image:: _static/diagrams/mir3.*
   :align: center
   :alt: Current MIR Architecture

**Future Vision**

.. image:: _static/diagrams/mirall.*
   :align: center
   :alt: Future MIR Architecture

JIT Internal Pipeline
---------------------

The following diagram illustrates the transformation phases within `mir-gen.c`.

.. image:: _static/diagrams/mir-gen.*
   :align: center
   :alt: MIR Generator Pipeline

C to MIR Compiler
-----------------

Structure of the `c2mir` compiler.

.. image:: _static/diagrams/c2mir.*
   :align: center
   :alt: C2MIR Compiler Structure

Interactive Diagrams
--------------------

(Use Mermaid syntax here for new diagrams)

.. mermaid::

    graph TD
        A[C Code] -->|c2m| B(MIR Binary)
        B -->|mir-interp| C{Execution}
        C -->|Interpret| D[Slow / Immediate]
        C -->|JIT| E[Fast / Warmup]

.. include:: generated_diagrams.rst
