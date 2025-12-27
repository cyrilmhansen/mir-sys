The Medium Intermediate Representation
======================================

Types
-----
The type system is simplified to fit into 64-bit registers.

The public enum ``MIR_type_t`` in ``mir/mir.h`` defines the handful of primitive kinds MIR understands: ``MIR_T_I64``, ``MIR_T_U64``, ``MIR_T_F``, ``MIR_T_D``, ``MIR_T_LD``, ``MIR_T_P``, and block payloads (``MIR_T_BLK``/``MIR_T_RBLK``). Smaller integer types are widened automatically during lowering.

Instructions
------------
Instructions are defined by their opcodes.

The opcode enum ``MIR_insn_code_t`` (also in ``mir/mir.h``) lists every instruction MIR accepts—moves, arithmetic, comparisons, branches, calls, and the ``MIR_UNSPEC`` escape hatch for target-specific encodings. See the enum table in the header for the exhaustive list.
