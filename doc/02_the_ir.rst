The Ideal C Machine
===================

The MIR (Medium Internal Representation) defines a virtual machine that closely mimics the capabilities of a modern 64-bit CPU, but abstracts away the specific encodings and register names. It is designed to be the "perfect target" for a C compiler.

The Type System
---------------
Unlike higher-level IRs that might have complex struct definitions, the MIR VM operates on a simplified set of primitives. Every value in the MIR VM fits into a 64-bit register (or 128-bit for `long double` on some platforms).

.. doxygenenum:: MIR_type_t
   :project: MIR

MIR_type_t quick reference
~~~~~~~~~~~~~~~~~~~~~~~~~~
MIR keeps the type set intentionally small. Each enumerator below maps to a
machine-width interpretation used by the interpreter, register allocator, and
ABI lowering.

MIR_T_I8
  Signed 8-bit integer (promoted to 64-bit in registers).
MIR_T_U8
  Unsigned 8-bit integer (zero-extended when promoted).
MIR_T_I16
  Signed 16-bit integer.
MIR_T_U16
  Unsigned 16-bit integer.
MIR_T_I32
  Signed 32-bit integer.
MIR_T_U32
  Unsigned 32-bit integer.
MIR_T_I64
  Signed 64-bit integer.
MIR_T_U64
  Unsigned 64-bit integer.
MIR_T_F
  32-bit IEEE-754 float.
MIR_T_D
  64-bit IEEE-754 double.
MIR_T_LD
  C `long double`. The size is target-ABI dependent: 80-bit extended precision on
  some x86_64 ABIs, 128-bit on some AArch64/RISC-V configurations, or 64-bit on
  targets that alias it to double.

  This means ABI handling, register allocation, and stack slot sizing must
  query the target for the actual width. You will see dedicated paths in the
  backends to handle 16-byte `long double` where required.
MIR_T_P
  Pointer value, sized to the host address width (64-bit on supported targets).
MIR_T_BLK
  By-value aggregate argument. This is used for C structs/unions passed by value.

  The argument carries a `size` so the ABI layer can decide whether it is passed
  in registers, by reference, or via a hidden copy.
MIR_T_RBLK
  By-value aggregate result (returned block). This represents functions that
  return a struct/union by value.

  Most ABIs lower this to a hidden first argument (a pointer to the return
  buffer) and change the logical return type to `void`. MIR models that directly
  so the backend can materialize the ABI-specific calling sequence.
MIR_T_UNDEF
  Unknown or not-yet-determined type used internally while constructing MIR.
MIR_T_BOUND
  Sentinel marking the end of the enum; used for sizing tables.

The Instruction Set
-------------------
The MIR ISA is a RISC-like load/store architecture with a rich set of 2-operand and 3-operand instructions.

.. doxygenenum:: MIR_insn_code_t
   :project: MIR

.. _anatomy:

The Anatomy of Code
-------------------
A MIR function is a doubly-linked list of instructions. Each instruction contains an opcode and a variable number of operands.

.. doxygenstruct:: MIR_insn
   :project: MIR
   :members:

.. doxygenstruct:: MIR_op_t
   :project: MIR
   :members:
