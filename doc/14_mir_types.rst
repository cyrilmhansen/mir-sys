MIR Data Type Reference Guide
============================

This guide provides an elaboration on the ``MIR_type_t`` enumeration and related definitions from the ``mir.h`` file. It covers MIR's data types, their properties, and how they relate to the underlying hardware and ABI.

1. Introduction to MIR Data Types
---------------------------------

MIR (Medium Intermediate Representation) defines a set of fundamental data types designed to closely map to hardware capabilities, ensuring efficient JIT compilation. These types are used across MIR instructions, function prototypes, variable declarations, and memory operations.

The ``MIR_type_t`` enumeration is the core of MIR's type system, generated using X-macros for brevity and maintainability.

.. code-block:: c

    typedef enum {
      REP8 (TYPE_EL, I8, U8, I16, U16, I32, U32, I64, U64), // Integer types of different size:
      REP3 (TYPE_EL, F, D, LD),                             // Float or (long) double type
      REP2 (TYPE_EL, P, BLK),                               // Pointer, memory blocks
      TYPE_EL (RBLK) = TYPE_EL (BLK) + MIR_BLK_NUM,         // return block
      REP2 (TYPE_EL, UNDEF, BOUND),
    } MIR_type_t;

1.1. Naming Convention for ``MIR_type_t``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Each enum member starts with ``MIR_T_`` followed by an abbreviation:

*   **``I<N>``**: Signed integer of *N* bits (e.g., ``MIR_T_I8`` for ``int8_t``).
*   **``U<N>``**: Unsigned integer of *N* bits (e.g., ``MIR_T_U8`` for ``uint8_t``).
*   **``F``**: Single-precision floating-point (``float``).
*   **``D``**: Double-precision floating-point (``double``).
*   **``LD``**: Long double floating-point (``long double``).
*   **``P``**: Pointer type.
*   **``BLK``**: Generic memory block.
*   **``RBLK``**: Return memory block (a specialized ``BLK`` type).
*   **``UNDEF``**: Undefined type (used for internal error states or uninitialized values).
*   **``BOUND``**: Marks the end of the enum, for internal use.

2. Integer Types (``MIR_T_I8`` to ``MIR_T_U64``)
------------------------------------------------

MIR provides explicit control over integer sizes and signedness, directly reflecting common CPU integer registers and memory sizes.

*   **Signed Integers**: ``MIR_T_I8``, ``MIR_T_I16``, ``MIR_T_I32``, ``MIR_T_I64``
    *   Map to ``int8_t``, ``int16_t``, ``int32_t``, ``int64_t`` respectively.
*   **Unsigned Integers**: ``MIR_T_U8``, ``MIR_T_U16``, ``MIR_T_U32``, ``MIR_T_U64``
    *   Map to ``uint8_t``, ``uint16_t``, ``uint32_t``, ``uint64_t`` respectively.

**Utility Functions:**

*   ``static inline int MIR_int_type_p (MIR_type_t t)``: Returns ``TRUE`` if ``t`` is any integer type (including pointer).

3. Floating-Point Types (``MIR_T_F``, ``MIR_T_D``, ``MIR_T_LD``)
----------------------------------------------------------------

MIR supports standard floating-point precision levels, though ``long double`` has platform-specific considerations.

*   **``MIR_T_F``**: Single-precision float (``float``).
*   **``MIR_T_D``**: Double-precision float (``double``).
*   **``MIR_T_LD``**: Long double (``long double``).
    *   **Platform Dependency**: The size and precision of ``long double`` vary by target. It might be 80-bit, 128-bit, or even identical to ``double`` (64-bit). The MIR generator might convert ``MIR_T_LD`` operations to ``MIR_T_D`` if the target's ``long double`` is effectively a ``double`` (e.g., on some Windows ABIs).

**Utility Functions:**

*   ``static inline int MIR_fp_type_p (MIR_type_t t)``: Returns ``TRUE`` if ``t`` is any floating-point type.

4. Pointer Type (``MIR_T_P``)
-----------------------------

The ``MIR_T_P`` type represents memory addresses. Its underlying size depends on the target architecture.

*   **Size Dependency**:
    *   ``#define MIR_PTR32 1`` if ``UINTPTR_MAX == 0xffffffff`` (32-bit pointers).
    *   ``#define MIR_PTR64 1`` if ``UINTPTR_MAX == 0xffffffffffffffffu`` (64-bit pointers).
    *   The system checks for 32-bit or 64-bit architectures at compile time.

5. Block Types (``MIR_T_BLK``, ``MIR_T_RBLK``)
----------------------------------------------

These types are crucial for handling aggregate data (structs, arrays) in function calls according to specific ABI rules. They do **not** represent values directly, but rather how blocks of memory are *passed* or *returned*.

*   **``MIR_T_BLK``**: General memory block type for arguments.
    *   **Variants**: ``MIR_BLK_NUM`` (defined as 5) indicates that there are multiple ways a block can be passed (e.g., partially in registers, entirely on stack, by reference). These variations (``MIR_T_BLK + 0`` to ``MIR_T_BLK + MIR_BLK_NUM - 1``) allow the generator to select the correct ABI handling.
*   **``MIR_T_RBLK``**: Return memory block type.
    *   A specialized block type used specifically for function return values, typically when the return value is an aggregate that doesn't fit in standard return registers and must be passed by reference (caller-allocated memory).

**Utility Functions:**

*   ``static inline int MIR_blk_type_p (MIR_type_t t)``: Returns ``TRUE`` if ``t`` is a general block type (``MIR_T_BLK`` to ``MIR_T_RBLK - 1``).
*   ``static inline int MIR_all_blk_type_p (MIR_type_t t)``: Returns ``TRUE`` if ``t`` is any block type (including ``MIR_T_RBLK``).

6. Miscellaneous Type Definitions
---------------------------------

*   **``MIR_type_t`` (``MIR_T_UNDEF``, ``MIR_T_BOUND``)**:
    *   ``MIR_T_UNDEF``: Represents an undefined or uninitialized type, often used for error states or placeholders during compilation.
    *   ``MIR_T_BOUND``: An internal marker indicating the end of the ``MIR_type_t`` enumeration.
*   **``MIR_scale_t``**:
    *   ``typedef uint8_t MIR_scale_t;``
    *   Represents the **scale factor** for an index register in memory addressing (e.g., in ``[base + index * scale + disp]``). Typical values are 1, 2, 4, 8, reflecting element sizes.
    *   ``#define MIR_MAX_SCALE UINT8_MAX`` defines the maximum possible scale value.
*   **``MIR_disp_t``**:
    *   ``typedef int64_t MIR_disp_t;``
    *   Represents a signed 64-bit **displacement** (offset) used in memory addressing.
*   **``MIR_reg_t``**:
    *   ``typedef uint32_t MIR_reg_t;``
    *   Represents a **register number**. In MIR, this can be a pseudo-register (virtual register managed by the compiler) or a physical hardware register. Register numbers ``> MAX_HARD_REG`` are pseudos.
    *   ``#define MIR_MAX_REG_NUM UINT32_MAX`` and ``#define MIR_NON_VAR MIR_MAX_REG_NUM`` are used for internal representation, where ``MIR_NON_VAR`` signifies the absence of a register (e.g., an operand slot that doesn't use a register).
*   **``MIR_imm_t``**:
    *   ``typedef union { int64_t i; uint64_t u; float f; double d; long double ld; } MIR_imm_t;``
    *   A union type to hold immediate (constant) values, allowing flexibility across different literal types.
*   **``MIR_alias_t``**:
    *   ``typedef uint32_t MIR_alias_t;``
    *   A unique identifier used to represent an **alias name**. This is critical for alias analysis in optimization passes, allowing the compiler to determine if different memory access instructions might refer to the same physical memory location.
