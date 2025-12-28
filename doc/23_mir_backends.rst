Target Backend Architecture
===========================

We have explored the generator's analytical mind; now we look at its **Body**.

Each target architecture (RISC-V, ARM64, x86_64) is implemented as a set of platform-specific functions that fulfill the generic generator's requests. These files are typically found in ``mir/mir-gen-*.c``.

1. The Target Case Study: RISC-V (``mir-gen-riscv64.c``)
-------------------------------------------------------

RISC-V is the "Simplest Civilization" in the MIR multiverse. Its clean, load/store architecture and fixed-size instructions make it an ideal starting point for understanding how MIR meets silicon.

1.1 The Register Roster (The Citizenry)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Every backend defines its **Hard Registers**. In RISC-V, these are the ``X`` (integer) and ``F`` (floating-point) registers.

*   **``hard_reg_alloc_order``**: The Register Allocator doesn't pick registers at random. This array defines the preference. It typically starts with "caller-saved" (temporary) registers (``R8-R15``, ``F8-F15``).
    *   *Why?* If we use a temporary register, we don't have to save it to the stack when the function starts. We only use "callee-saved" registers if the temporaries are all full.
*   **``target_call_used_hard_reg_p``**: This is the "Clobber" map. It tells the allocator which registers are safe to hold data across a function call. In RISC-V, ``A0-A7`` are always assumed to be "burned" by a call.

1.2 The Local Customs: ``target_machinize`` (The Bureaucracy)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

This is the most critical phase for **ABI (Application Binary Interface)** compliance. It's where the abstract IR learns about the laws of the land.

*   **Calling Convention**: The RISC-V ABI mandates that integer arguments go in ``A0-A7`` and floating point in ``FA0-FA7``.
*   **The Transformation**: ``machinize_call`` rewrites generic ``CALL`` instructions.
    *   **The Shuffle**: It moves arguments into the specific ``A`` or ``FA`` registers.
    *   **The Overflow**: If a function has 10 arguments, the first 8 go in registers, and the remaining 2 are "spilled" onto the stack (``SP`` relative).
*   **The Signedness Quirk**: A unique law of RISC-V is that 32-bit values in registers must be **sign-extended** even if they are logically unsigned. ``get_ext_code`` ensures that an unsigned 32-bit MIR type is passed as a signed 32-bit value to satisfy the ABI.

1.3 Complex Operations: Built-in Shims (The Outsourcing)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Some operations are too complex for a simple RISC-V instruction (e.g., ``long double`` math or block moves). Instead of emitting hundreds of raw instructions, MIR "outsources" the work to C.

*   **``mir_blk_mov``**: A small C function that performs optimized memory copies.
*   **``get_builtin``**: This function identifies complex MIR opcodes like ``MIR_LDADD`` (Long Double Add).
    *   **The Hook**: It creates a prototype for a C helper function (e.g., ``mir_ldadd``).
    *   **The Call**: It replaces the ``LDADD`` opcode with a standard ``CALL`` to that C helper.
    *   This keeps the machine code generator simple while supporting the full range of C types.

1.4 The Instruction Forge: ``target_translate`` (The Smithy)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

This is the final step where MIR opcodes are hammered into raw machine bytes.

*   **Instruction Selection**: A ``MIR_ADD`` might become an ``ADD`` (reg-reg) or an ``ADDI`` (reg-imm) instruction depending on its operands.
*   **Branch Relaxing**: RISC-V branch instructions have a limited range. If a jump is too far, the backend must "relax" it—replacing a simple branch with a jump-and-link (``JAL``) instruction that can reach the entire address space.


2. Evolution of Complexity: ARM64 and x86_64
--------------------------------------------

While the structure remains the same, other architectures introduce new challenges.

2.1 ARM64 (AArch64)
~~~~~~~~~~~~~~~~~~~

*   **Immediate Ranges**: ARM64 instructions have strict limits on the size of constant values (immediates). If a constant is too large, the backend must split the move into multiple instructions or use a "literal pool" in memory.
*   **Vector Power**: ARM64 has extensive support for NEON (SIMD) registers, requiring more complex register class management.

2.2 x86_64 (CISC)
~~~~~~~~~~~~~~~~~

*   **The 2-Operand Constraint**: Most x86 instructions are of the form ``A = A op B``. MIR is 3-operand: ``A = B op C``. The x86 backend must frequently inject ``MOV`` instructions to satisfy this constraint.
*   **Addressing Modes**: x86 has the complex SIB (Scale-Index-Base) addressing. The backend's ``target_insn_ok_p`` must decide if a complex MIR memory operand can be handled by a single x86 instruction or if it must be broken down.
*   **Variable Length**: Unlike RISC-V's 4-byte instructions, x86 instructions can be anywhere from 1 to 15 bytes long. The code emitter must be a precision engineer.

3. The Endianness Challenge
---------------------------

MIR is designed to be **Endian-Independent**.

*   **The Register Slot**: Internally, MIR treats registers as 64-bit containers.
*   **The Offset Magic**: On a **Little Endian** machine (x86, ARM64), the first byte of a 64-bit value is at offset 0. On a **Big Endian** machine (s390x, ARM64-BE), it's at offset 7.
*   **``_MIR_addr_offset``**: This core function dynamically detects the host's endianness and provides the necessary offset.

.. note::
   **Running ARM64 in Big Endian mode?**
   
   To support ARM64 BE, the backend would need to:
   1. Use the correct ``_MIR_addr_offset`` for byte/short loads.
   2. Ensure the ``target_translate`` phase emits instructions in the correct byte order (ARM64 instructions are always little-endian even in BE mode, but data access follows the BE rules).
   3. Correctly handle the ABI's specific rules for stack layout and argument passing, which often reverse the order of fields in aggregates.

**Summary**: Backends are where the abstract meets the physical. By starting with the simple RISC-V model and layer on complexity for ARM and x86, MIR maintains a clean separation of concerns while achieving high performance across the entire spectrum of modern computing.
