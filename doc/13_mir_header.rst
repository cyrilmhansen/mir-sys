MIR Header Analysis
===================

Based on the ``mir/mir.h`` file content provided in the codebase, here are the detailed answers to your inquiries.

C and C++ Compilation Compatibility
-----------------------------------

**Is it correct that `mir.h` allows C and CPP compilation?**

**Yes.**
The file contains standard "extern C" guards:

.. code-block:: c

    #ifdef __cplusplus
    extern "C" {
    #endif
    // ... definitions ...
    #ifdef __cplusplus
    }
    #endif

This tells a C++ compiler to disable name mangling for the enclosed functions, allowing C++ code to link against the compiled C library.

**Does this change has consequences to the API and/or functionality?**

**No functional change, but it ensures interoperability.**

*   **API:** The API remains C-based. It does not use C++ features (classes, templates, references). It uses standard C types and structs.
*   **Functionality:** The underlying logic does not change.
*   **Consequence:** It allows the MIR library to be used seamlessly in C++ projects (like LLVM or generic game engines) without the user having to manually wrap headers in ``extern "C"``.

Standard C Imports
------------------

You asked to specify exactly what is imported by the lines provided in your snippet.

.. code-block:: c

    #include <stdio.h>
    #include <stdint.h>
    #include <assert.h>

These headers provide the following definitions used throughout ``mir.h``:

*   **``<stdio.h>``**:
    *   **``FILE``**: Used in API functions for input/output, such as ``MIR_output(..., FILE *f)``, ``MIR_write``, and ``MIR_read``.
    *   *(Note: Implementation files like ``mir.c`` use ``fprintf``, ``vfprintf``, ``fopen``, etc., but the header primarily relies on the ``FILE`` type definition).*
*   **``<stdint.h>``**:
    *   **Fixed-width integer types**: Used extensively to ensure the JIT works identically across platforms. Specifically: ``int64_t``, ``uint64_t``, ``uint8_t``, ``uint32_t``, ``int8_t``, ``int16_t``, ``uint16_t``, ``int32_t``, ``uintptr_t``.
*   **``<assert.h>``**:
    *   **``assert``**: Used in the ``mir_assert`` macro definition (``#define mir_assert(cond) assert (cond)``). This allows the library to perform internal sanity checks during debug builds.

Responsibility of Internal Headers
----------------------------------

These files are internal utilities that provide data structures and memory management for the MIR core.

*   **``mir-dlist.h`` (Doubly Linked Lists)**
    *   **Responsibility:** Provides generic macros to create and manipulate typed doubly linked lists. Used for managing lists of instructions (``MIR_insn_t``), basic blocks, modules, and items where efficient insertion/deletion in the middle of the sequence is required.
*   **``mir-varr.h`` (Variable Arrays / Vectors)**
    *   **Responsibility:** Provides generic macros for dynamic arrays (vectors) that grow automatically. Used for storing operands, variables, and other sequential data where index-based access is needed.
*   **``mir-htab.h`` (Hash Tables)**
    *   **Responsibility:** Provides a generic hash table implementation. Used for symbol tables (mapping names to registers or items), optimization passes (GVN, CSE), and quick lookups.
*   **``mir-alloc.h`` (General Allocation)**
    *   **Responsibility:** Abstraction layer for heap memory management. It wraps ``malloc``, ``calloc``, ``realloc``, and ``free``. It allows the user to pass a custom allocator context (``MIR_alloc_t``), enabling custom memory strategies (e.g., arenas or pools) for the compiler data.
*   **``mir-code-alloc.h`` (Code Memory Allocation)**
    *   **Responsibility:** specialized allocator for **executable memory**.
    *   It handles OS-specific operations (like ``mmap``, ``VirtualAlloc``, ``mprotect``) required to allocate memory that has ``PROT_EXEC`` (execute) permissions. This is distinct from ``mir-alloc.h`` because standard ``malloc`` memory is usually not executable for security reasons (W^X protection).

Configuration Macros (MIR_NO_IO, MIR_NO_SCAN)
---------------------------------------------

These are **build-time feature flags** used to reduce the footprint of the library if certain features are not needed.

*   **``MIR_NO_IO``**:
    *   **Effect:** If defined, functions related to binary serialization/deserialization (``MIR_write``, ``MIR_read``, ``MIR_write_module``, etc.) are excluded from compilation.
    *   **Use Case:** An embedded JIT environment that generates code in memory and executes it immediately, never needing to save the IR to disk.
*   **``MIR_NO_SCAN``**:
    *   **Effect:** If defined, functions related to parsing textual MIR (``MIR_scan_string``) are excluded.
    *   **Use Case:** A language frontend that builds MIR using API calls (``MIR_new_insn``, etc.) programmatically rather than parsing text strings.

MIR_UNUSED
----------

.. code-block:: c

    #ifdef __GNUC__
    #define MIR_UNUSED __attribute__ ((unused))
    #else
    #define MIR_UNUSED
    #endif

*   **Elaboration:** This is a compiler hint (specifically for GCC and Clang).
*   **Purpose:** It suppresses "unused variable" or "unused parameter" warnings during compilation.
*   **Context:** In generic macros (like the ``REP`` macros or ``HTAB`` definitions), some generated code might declare variables that specific instances don't actually use. Without this attribute, the build would be noisy or fail if ``-Werror`` is enabled.

REP Macros and MIR_error_type
-----------------------------

.. code-block:: c

    #define REP2(M, a1, a2) M (a1) REP_SEP M (a2)
    // ... (recursive definitions up to REP8)
    #define REP_SEP ,

*   **Elaboration:** These are **X-Macros** helpers used to reduce code repetition when defining enumerations or lists.
*   **Mechanism:** ``REP8(ERR_EL, no, syntax, ...)`` takes a macro ``M`` (here ``ERR_EL``) and a list of arguments. It applies ``M`` to every argument, separating them with commas.

.. code-block:: c

    #define ERR_EL(e) MIR_##e##_error
    typedef enum MIR_error_type {
      REP8 (ERR_EL, no, syntax, ...),
      // ...
    } MIR_error_type_t;

*   **Result:** This expands to a standard C enum definition:

    .. code-block:: c

        typedef enum MIR_error_type {
           MIR_no_error,
           MIR_syntax_error,
           // ... and so on
        } MIR_error_type_t;

*   **Benefit:** Allows defining the prefixes (``MIR_``) and suffixes (``_error``) in one place. If the naming convention changes, only the ``ERR_EL`` macro needs to change, not the whole enum list.

MIR_NO_RETURN
-------------

.. code-block:: c

    #ifdef __GNUC__
    #define MIR_NO_RETURN __attribute__ ((noreturn))
    #else
    #define MIR_NO_RETURN
    #endif

*   **Elaboration:** This indicates to the compiler/optimizer that the function **never returns control** to the point where it was called.
*   **Usage:** Used for ``MIR_error_func_t`` (see below).
*   **Behavior:** When an error function is called, it is expected to ``exit()``, ``abort()``, or ``longjmp()`` away. It prevents the compiler from generating warning messages about unreachable code appearing after the error call, or missing return values in branches that trigger errors.

MIR_error_func_t and INSN_EL
----------------------------

**``MIR_error_func_t``**

.. code-block:: c

    typedef void MIR_NO_RETURN (*MIR_error_func_t) (MIR_error_type_t error_type, const char *format, ...);

*   **Elaboration:** This defines a function pointer type for error handling.
    *   It takes an error code (``error_type``).
    *   It takes a ``printf``-style format string and variable arguments (``...``).
    *   It is marked ``MIR_NO_RETURN``, meaning the implementation must terminate the current execution flow (e.g., exit program).

**``INSN_EL``**

.. code-block:: c

    #define INSN_EL(i) MIR_##i

*   **Elaboration:** This macro is used in conjunction with the ``REP`` macros (similar to ``ERR_EL`` above) to define the **Instruction Codes** enum (``MIR_insn_code_t``).
*   **Example:**
    Inside the enum definition later in the file:
    ``REP4 (INSN_EL, MOV, FMOV, DMOV, LDMOV)``
    Expands to:
    ``MIR_MOV, MIR_FMOV, MIR_DMOV, MIR_LDMOV``
*   **Purpose:** Defines the opcodes for the MIR virtual machine (Move, Float Move, Double Move, etc.).

MIR Instruction Set Reference Guide
===================================

This header file defines the **Instruction Set Architecture (ISA)** of the MIR virtual machine. Because the list of instructions is extensive and generated via macros, understanding it requires breaking it down into a structured framework.

Here is a **structured documentation approach** designed to be expanded over time. We will categorize the instructions by functionality and explain the naming conventions used to generate them.

1. Definition Mechanism (The "How")
-----------------------------------

The ``MIR_insn_code_t`` enumeration is generated using **X-Macros** (``REP`` macros) and a concatenation macro (``INSN_EL``).

*   **Macro:** ``#define INSN_EL(i) MIR_##i``
*   **Expansion Logic:** When the preprocessor encounters ``REP4(INSN_EL, MOV, FMOV, DMOV, LDMOV)``, it expands to:

    .. code-block:: c

        MIR_MOV, MIR_FMOV, MIR_DMOV, MIR_LDMOV,

This technique ensures that adding a new instruction only requires adding it to the list in ``mir.h``, and the enum values are automatically assigned.

2. Naming Conventions (The "Grammar")
-------------------------------------

MIR opcodes follow a strict naming schema denoting data types and sizes. Understanding these prefixes and suffixes allows you to deduce the behavior of almost any instruction without looking up its specific documentation.

.. list-table::
   :header-rows: 1
   :widths: 15 20 25 40

   * - Affix
     - Meaning
     - Context
     - Data Type
   * - **(None)**
     - 64-bit Integer
     - Default (e.g., ``ADD``)
     - ``int64_t`` / ``uint64_t``
   * - **S** (Suffix)
     - Short (32-bit)
     - Suffix (e.g., ``ADDS``)
     - ``int32_t`` / ``uint32_t``
   * - **U** (Prefix)
     - Unsigned
     - Prefix (e.g., ``UDIV``)
     - Unsigned Integers
   * - **F** (Prefix)
     - Float
     - Prefix (e.g., ``FADD``)
     - ``float`` (IEEE 754 Single)
   * - **D** (Prefix)
     - Double
     - Prefix (e.g., ``DADD``)
     - ``double`` (IEEE 754 Double)
   * - **LD** (Prefix)
     - Long Double
     - Prefix (e.g., ``LDADD``)
     - ``long double`` (Platform specific)

3. Instruction Categories (The Framework)
-----------------------------------------

We will structure the documentation into **8 Logic Groups**. This structure allows us to document specific behaviors (like rounding modes or overflow handling) per group in the future.

Group A: Data Movement
~~~~~~~~~~~~~~~~~~~~~~

Instructions responsible for moving data between registers, memory, and immediate values.

*   **Base Ops:** ``MOV`` (Move)
*   **Variants:** ``FMOV``, ``DMOV``, ``LDMOV``.
*   **Address Loading:** ``ADDR`` (Load effective address), ``LADDR`` (Load Label Address).

Group B: Type Conversion & Extension
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Instructions that change the bit-width or format of data.

*   **Extensions:** ``EXT`` (Sign-extend), ``UEXT`` (Zero-extend). Available for 8, 16, and 32 bits.
*   **Int <-> Float:** ``I2F`` (Int to Float), ``F2I`` (Float to Int), and variations for Double/Long Double.

Group C: Arithmetic Operations
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Standard mathematical operations. Most have 3 operands (``DEST = SRC1 op SRC2``).

*   **Core Ops:** ``ADD``, ``SUB``, ``MUL``, ``DIV`` (Divide), ``MOD`` (Modulus), ``NEG`` (Negate - 2 operands).
*   **Variants:** Signed vs Unsigned (``UMUL``), 32-bit vs 64-bit (``ADDS``), Floating point (``FADD``).
*   **Overflow Handling:** ``ADDO``, ``SUBO``, ``MULO``. These set an internal overflow flag used by branch instructions in Group F.

Group D: Bitwise & Shift Operations
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Logical operations performed on integer registers.

*   **Logical:** ``AND``, ``OR``, ``XOR``.
*   **Shifts:** ``LSH`` (Left Shift), ``RSH`` (Right Shift Arithmetic/Sign-extending), ``URSH`` (Unsigned Right Shift/Zero-filling).

Group E: Comparisons
~~~~~~~~~~~~~~~~~~~~

These instructions compare two values and set a result register.

*   **Core Ops:** ``EQ`` (Equal), ``NE`` (Not Equal), ``LT`` (Less Than), ``LE`` (Less Equal), ``GT`` (Greater Than), ``GE`` (Greater Equal).
*   **Variants:** Float/Double comparisons (``FEQ``, ``DEQ``) handle NaN (Not a Number) logic specific to IEEE 754.

Group F: Control Flow (Branching)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Instructions that manipulate the Instruction Pointer (IP).

1.  **Unconditional:** ``JMP`` (Jump to label), ``JMPI`` (Indirect Jump to address in register).
2.  **Branch on Value:** ``BT`` (Branch if True/Non-Zero), ``BF`` (Branch if False/Zero).
3.  **Branch on Comparison:** ``BEQ`` (Branch Equal), ``BNE``, ``BLT``, etc. These combine a comparison and a jump in one instruction.
4.  **Branch on Overflow:** ``BO`` (Branch Overflow), ``BNO`` (Branch No Overflow). Used immediately after Group C Overflow instructions.

Group G: Function Calls & ABI
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Instructions managing function invocation and argument passing.

*   **Invocation:** ``CALL``, ``INLINE`` (Hint to inline if possible), ``RET`` (Return).
*   **JIT Optimization:** ``JCALL`` / ``JRET`` (Jump Call/Return for tail-call optimizations or specific interpreter dispatching).
*   **Variable Arguments:** ``VA_START``, ``VA_ARG``, ``VA_END``, ``VA_BLOCK_ARG``. Used to implement C-style ``varargs``.

Group H: Stack & Memory Management
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

*   **Allocation:** ``ALLOCA`` (Allocate memory on the stack frame).
*   **Scope Management:** ``BSTART``, ``BEND``. Used to mark block scopes for stack unwinding or memory reclamation.

Group I: Meta-Instructions & Properties
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Instructions used by the generator or optimizer that do not necessarily map one-to-one to machine code.

*   **Properties:** ``PRSET`` (Property Set), ``PRBEQ`` (Property Branch Equal). Used for optimizing dynamic typing or specialized versions of code.
*   **Internal:** ``USE`` (Mark variable as live), ``PHI`` (Phi node for SSA form), ``UNSPEC`` (Unspecified target-specific behavior).

4. Documentation Expansion Plan
-------------------------------

To enrich this documentation in the future, we should tackle one **Group** at a time.

**Example Template for Group C (Arithmetic):**

    **Instruction:** ``ADD`` / ``ADDS``

    *   **Format:** ``ADD dest, src1, src2``
    *   **Description:** Computes ``src1 + src2`` and stores result in ``dest``.
    *   **Variants:**
        *   ``ADD``: 64-bit integer addition.
        *   ``ADDS``: 32-bit integer addition (result sign-extended to 64-bit in register).
    *   **Edge Cases:** Overflow wraps around (standard two's complement). For overflow detection, use ``ADDO``.

We can iterate through the groups defined in Section 3 to build the complete manual.
