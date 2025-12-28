Diagrams
========

This appendix collects the architectural diagrams found in the codebase.

Project Overview
----------------

**Current State of MIR (The Present)**

This diagram shows the existing modular structure: MIR Core, the Interpreter, and the Generator backends for multiple architectures.

.. image:: _static/diagrams/mir3.*
   :align: center
   :alt: Current MIR Architecture

**Future Vision (The Horizon)**

The long-term plan for MIR, including additional optimization passes and broader architecture support.

.. image:: _static/diagrams/mirall.*
   :align: center
   :alt: Future MIR Architecture

JIT Internal Pipeline
---------------------

The following diagram illustrates the transformation phases within `mir-gen.c`. It highlights the path from abstract IR to native machine code, including SSA form, optimization loops, and the "Allocation War" of register assignment.

.. image:: _static/diagrams/mir-gen.*
   :align: center
   :alt: MIR Generator Pipeline

C to MIR Compiler
-----------------

Structure of the `c2mir` compiler. This diagram shows the four-pass alchemy: Lexer/Preprocessor, Recursive Descent Parser, Semantic Context Pass, and the final emission of MIR instructions.

.. image:: _static/diagrams/c2mir.*
   :align: center
   :alt: C2MIR Compiler Structure

Interactive Diagrams
--------------------

(Use Mermaid syntax here for new diagrams)

.. graphviz::

   digraph G {
      node [fontname="Helvetica", shape=box];
      edge [fontname="Helvetica", fontsize=10];

      src [label="C Code"];
      bin [label="MIR Binary"];
      exec [label="Execution", shape=diamond];
      interp [label="Interpret\n(Slow / Immediate)"];
      jit [label="JIT\n(Fast / Warmup)"];

      src -> bin [label="c2m"];
      bin -> exec [label="mir-interp"];
      exec -> interp [label="Interpret"];
      exec -> jit [label="JIT"];
   }

.. include:: generated_diagrams.rst
