Glossary & Index
==================

This glossary provides definitions for key terms used throughout the MIR documentation, with cross-references to the chapters where they are discussed in detail.

.. contents::
   :local:
   :depth: 1

Fundamental Concepts
--------------------

MIR (Medium Internal Representation)
    The core intermediate language used by this project. It is a strongly-typed, register-based IR designed to be close to machine code but abstract enough for portability.
    *See:* :doc:`01_philosophy`, :doc:`02_the_ir`.

Context (``MIR_context_t``)
    The "universe" in which a compilation session exists. It holds memory allocators, error handlers, string tables, and the list of loaded modules.
    *See:* :doc:`17_mir_context_modules`, :doc:`19_mir_implementation` (Section 3).

Module (``MIR_module_t``)
    A container for code and data, analogous to a C translation unit. It acts as a namespace for functions, globals, and type definitions.
    *See:* :doc:`10_modules_and_linking`, :doc:`17_mir_context_modules`, :doc:`19_mir_implementation` (Section 18).

Item (``MIR_item_t``)
    The fundamental unit of content within a module. Items can be functions, global variables (data/bss), prototypes, or symbol declarations (import/export).
    *See:* :doc:`16_mir_architecture`, :doc:`19_mir_implementation` (Section 20).

Data Types & Memory
-------------------

Type (``MIR_type_t``)
    An enumeration defining the data types supported by the VM (e.g., ``MIR_T_I64``, ``MIR_T_D``). MIR types map directly to hardware registers.
    *See:* :doc:`14_mir_types`.

Operand (``MIR_op_t``)
    An argument to an instruction. It can be a register, a constant (immediate), a memory reference, or a label.
    *See:* :doc:`16_mir_architecture`, :doc:`19_mir_implementation` (Section 44).

Memory Operand (``MIR_mem_t``)
    A complex operand representing a memory address using the form ``type: disp(base, index, scale)``.
    *See:* :doc:`15_mir_memory`.

BSS (Block Started by Symbol)
    A section of memory reserved for uninitialized global variables, which are automatically zeroed at runtime.
    *See:* :doc:`19_mir_implementation` (Section 23).

Execution & Optimization
------------------------

Interpreter
    The engine that executes MIR code directly without compiling it to machine code. Useful for debugging and rapid iteration.
    *See:* :doc:`03_interpreter`.

Generator (``mir-gen``)
    The component that translates MIR IR into native machine code for the host architecture.
    *See:* :doc:`04_jit_pipeline`.

Simplification
    A transformation pass that canonicalizes MIR code (e.g., splitting memory-to-memory moves) to make it suitable for the generator.
    *See:* :doc:`19_mir_implementation` (Section 38 & 52).

Linking (``MIR_link``)
    The process of resolving symbolic references (imports/exports) between modules and the host environment.
    *See:* :doc:`10_modules_and_linking`, :doc:`19_mir_implementation` (Section 37).

Thunk
    A small piece of code that acts as a bridge or placeholder. In MIR, thunks are used for unlinked functions (redirecting to error handlers) or for ABI adaptation.
    *See:* :doc:`11_thunks_and_shims`.

Internal Mechanics
------------------

String Interning
    The process of storing only one copy of each distinct string (identifier) to save memory and allow fast integer-based comparisons.
    *See:* :doc:`19_mir_implementation` (Section 9).

Value Numbering (VN)
    An optimization technique used to identify and eliminate redundant computations (Common Subexpression Elimination) by hashing their components.
    *See:* :doc:`19_mir_implementation` (Section 55).

Alias Analysis
    The mechanism for determining if two pointers might refer to the same memory location. MIR uses ``alias`` and ``nonalias`` sets to guide this.
    *See:* :doc:`15_mir_memory`, :doc:`19_mir_implementation` (Section 10).

ISA (Instruction Set Architecture)
    The specific set of opcodes (e.g., ``MIR_ADD``, ``MIR_MOV``) defined by the MIR virtual machine.
    *See:* :doc:`13_mir_header`.

Tools & Ecosystem
-----------------

C2MIR
    A tool that compiles C code into MIR text or binary format.
    *See:* :doc:`08_c2mir`.

Scan/Parse
    The process of reading textual MIR and converting it into the internal AST.
    *See:* :doc:`19_mir_implementation` (Section 53).

Binary IO
    The serialization format for saving compiled MIR modules to disk for later loading.
    *See:* :doc:`19_mir_implementation` (Section 50).
