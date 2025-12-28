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

Builtin
    A predefined function or prototype provided by the MIR environment to handle essential operations (like memory moves) or to bridge the gap to native host functions.
    *See:* :doc:`19_mir_implementation` (Section 60 & 61).

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

ICode (Internal Code)
    A flattened, array-based representation of MIR instructions used by the interpreter for high-speed execution. It replaces the linked-list AST with dense value arrays.
    *See:* :doc:`22_mir_interpreter` (Section 2).

Direct Threaded Dispatch
    An interpretation technique where each instruction handler ends with a direct jump to the next handler's address, bypassing the overhead of a central loop or switch statement.
    *See:* :doc:`22_mir_interpreter` (Section 3).

Shim
    In the context of the interpreter, a tiny piece of machine code that acts as a bridge between the host's native ABI and the interpreter's internal execution model.
    *See:* :doc:`22_mir_interpreter` (Section 6).

FFI (Foreign Function Interface) Bridge
    The mechanism that allows MIR code (running in the interpreter or JIT) to call native C functions and vice versa, handling the complex translation of data between the VM and the host ABI.
    *See:* :doc:`22_mir_interpreter` (Section 5).

Generator (``mir-gen``)
    The component that translates MIR IR into native machine code for the host architecture.
    *See:* :doc:`04_jit_pipeline`, :doc:`19_mir_implementation` (Section 2).

Disassembly
    The process of converting raw machine code bytes back into human-readable assembly instructions. MIR uses system tools like ``objdump`` via pipes to provide this capability for debugging.
    *See:* :doc:`19_mir_implementation` (Section 69).

Target Inclusion
    The architectural pattern of including target-specific C files directly into the main implementation to create a monolithic, highly-coupled backend for each CPU architecture.
    *See:* :doc:`19_mir_implementation` (Section 70).

Simplification
    A transformation pass that canonicalizes MIR code (e.g., splitting memory-to-memory moves) to make it suitable for the generator.
    *See:* :doc:`19_mir_implementation` (Section 38 & 52), :doc:`20_mir_optimizations`.

CFG (Control Flow Graph)
    A representation of all paths that might be traversed through a program during its execution. In MIR, the CFG consists of Basic Blocks connected by edges.
    *See:* :doc:`04_jit_pipeline`.

SSA (Single Static Assignment)
    An intermediate form where every variable is assigned exactly once. This form simplifies many optimizations by making data flow explicit.
    *See:* :doc:`04_jit_pipeline`.

Register Allocation (RA)
    The process of assigning virtual MIR registers to a limited number of physical CPU registers. MIR uses a priority-based linear scan allocator.
    *See:* :doc:`04_jit_pipeline`.

Linking (``MIR_link``)
    The process of resolving symbolic references (imports/exports) between modules and the host environment.
    *See:* :doc:`10_modules_and_linking`, :doc:`19_mir_implementation` (Section 37).

Thunk
    A small piece of code that acts as a bridge or placeholder. In MIR, thunks are used for unlinked functions (redirecting to error handlers) or for ABI adaptation.
    *See:* :doc:`11_thunks_and_shims`.

Internal Mechanics
------------------

Basic Block Instruction (``bb_insn``)
    A wrapper for ``MIR_insn_t`` that stores generator-specific metadata, such as liveness information and indices used for data-flow bitmaps.
    *See:* :doc:`21_mir_generator` (Section 4 & 9).

Death Note (``dead_var_t``)
    A metadata record attached to an instruction identifying a register whose value is no longer needed after that point. Crucial for register reuse and liveness analysis.
    *See:* :doc:`21_mir_generator` (Section 10).

Object Cache (Pool)
    A memory management technique used for small, high-frequency objects (like ``dead_var_t``) where used objects are kept in a free list for reuse instead of being returned to the system heap.
    *See:* :doc:`21_mir_generator` (Section 10).

Leader
    The first instruction of a basic block. Leaders are typically labels, jump targets, or instructions immediately following a branch.
    *See:* :doc:`21_mir_generator` (Section 5).

Preheader
    A basic block inserted immediately before a loop entry. It provides a guaranteed single execution point for code moved out of the loop by LICM.
    *See:* :doc:`21_mir_generator` (Section 20).

Dominators
    A block A "dominates" block B if every path from the start to B must go through A. Used for loop detection and code motion.
    *See:* :doc:`04_jit_pipeline`.

Phi Function (``PHI``)
    A special instruction used in SSA form to merge values from different control flow paths.
    *See:* :doc:`02_the_ir`, :doc:`04_jit_pipeline`.

Liveness Analysis
    The process of determining which variables are "alive" (needed for future use) at each point in the program. This is critical for register allocation.
    *See:* :doc:`04_jit_pipeline`.

Spilling
    The act of moving a variable from a register to a memory slot (on the stack) when the number of active variables exceeds the available physical registers.
    *See:* :doc:`04_jit_pipeline`.

Machinize
    The target-specific pass that transforms generic MIR instructions into forms that directly map to the host CPU's instruction set and ABI. This includes converting to 2-operand format and handling calling conventions.
    *See:* :doc:`21_mir_generator` (Section 30).

Pressure Relief
    An optimization that moves instructions to reduce the peak number of simultaneously active variables, helping to avoid register spilling.
    *See:* :doc:`21_mir_generator` (Section 18.5).

Rebasing
    The process of patching absolute memory addresses in generated machine code once the final destination of the code block is known.
    *See:* :doc:`21_mir_generator` (Section 30.3).

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
    *See:* :doc:`19_mir_implementation` (Section 50 & 65-66).

Serialization
    The process of converting internal data structures (like modules and instructions) into a linear stream of bytes for storage or transmission.
    *See:* :doc:`19_mir_implementation` (Section 65).

W^X (Write XOR Execute)
    A security policy that prevents memory from being simultaneously writable and executable. MIR navigates this during code generation by toggling permissions.
    *See:* :doc:`19_mir_implementation` (Section 63).

Cache Flush
    The operation of invalidating the CPU's instruction cache to ensure that newly generated machine code is correctly loaded from memory.
    *See:* :doc:`19_mir_implementation` (Section 63).

Patching
    The process of modifying generated machine code in place, often used to resolve jump targets or link function calls after the initial code emission.
    *See:* :doc:`19_mir_implementation` (Section 64).
