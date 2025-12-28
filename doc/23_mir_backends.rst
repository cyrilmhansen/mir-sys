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
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Some operations are too complex for a simple RISC-V instruction (e.g., ``long double`` math or block moves). Instead of emitting hundreds of raw instructions, MIR "outsources" the work to C.

*   **``mir_blk_mov``**: A small C function that performs optimized memory copies.
*   **``get_builtin``**: This function identifies complex MIR opcodes like ``MIR_LDADD`` (Long Double Add).
    *   **The Hook**: It creates a prototype for a C helper function (e.g., ``mir_ldadd``).
    *   **The Call**: It replaces the ``LDADD`` opcode with a standard ``CALL`` to that C helper.

.. _gen_stack_frame:

1.4 Advanced Machinization: The Stack Frame and Prologue
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Before a function can execute, it must secure its "Living Quarters" in memory. ``target_make_prolog_epilog`` is the **Civil Engineer** that builds the stack frame.

*   **Frame Layout**: The RISC-V frame is 16-byte aligned. It contains:
    1.  **Saved Registers**: Callee-saved registers (like ``S0-S11``) if they are used.
    2.  **Pseudo-registers**: MIR virtual registers that didn't fit in silicon.
    3.  **Return Address (RA)** and **Old Frame Pointer (FP)**.
*   **The Squeeze**: If the frame is small (< 2KB), a single ``ADDI SP, SP, -size`` instruction suffices. If it's larger, the backend must use a temporary register to calculate the new stack pointer.

.. _gen_pattern_matcher:

1.5 The Pattern Matcher: Instruction Selection
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

The heart of ``target_translate`` is a declarative **Pattern Matcher**. Instead of writing thousands of ``if`` statements, the backend uses a `struct pattern` table.

*   **The Grammar**: Patterns use a shorthand (e.g., ``"r r r"`` for three registers, ``"r r i"`` for two registers and a 12-bit immediate).
*   **The Replacement**: Each pattern has a machine code template (e.g., ``"O33 F0 rd0 rs1 rS2"``). The translator fills in the holes (``rd0``, ``rs1``) with the actual register numbers from the MIR instruction.

.. _gen_branch_relax:

1.6 The Arcane Art: Branch Relaxation and Patching
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

RISC-V instructions are 32 bits wide, and their branch offsets are limited. A standard branch (``BEQ``) can only jump +/- 4KB.

*   **The Problem**: Large functions may need to jump further than the hardware allows.
*   **The Arcane Solution**: In ``target_translate``, MIR performs **Branch Relaxation**.
    *   If an offset fits in 12 bits, it emits a single ``BEQ``.
    *   If it doesn't, it "relaxes" the instruction into a complex sequence (e.g., inverting the condition and using a 20-bit ``JAL``).
    *   If even 1MB isn't enough, it uses an ``AUIPC`` + ``JALR`` sequence to reach the entire 64-bit address space.

.. note::
   **Historical Lore: The Fixed-Width Prison**
   
   In the early days of RISC (like MIPS and early Alpha), compilers had to be extremely clever because every instruction *had* to be exactly 32 bits. This created the "Branch Distance" problem. Modern architectures like RISC-V solve this by allowing the compiler to "relax" a single instruction into a multi-instruction trampoline.

2. The Elite Guard: ARM64 (AArch64)
------------------------------------

ARM64 is the dominant architecture of the mobile world. Its implementation in ``mir-gen-aarch64.c`` reflects a modern, efficient RISC design.

2.1 The Register Wealth
~~~~~~~~~~~~~~~~~~~~~~~

ARM64 provides 31 general-purpose registers and a dedicated **Zero Register** (``ZR``). It also uses a **Link Register** (``LR``) for return addresses, which saves memory traffic for "leaf" functions.

2.2 Apple Silicon vs. The World
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

The ARM64 backend must handle the divide between the **Standard ABI** and the **Apple ABI**.
*   **Variadic Arguments**: Apple passes variadics on the stack; Linux uses a complex register-save area. MIR's backend handles this with extensive conditional compilation.

3. The Veteran: x86_64 (CISC)
----------------------------

The x86_64 backend (``mir-gen-x86_64.c``) is a masterpiece of adaptation, turning MIR's 3-operand RISC IR into idiosyncratic CISC instructions.

3.1 The Arcane Puzzle: SIB Byte Encoding
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

The most arcane part of the x86 backend is encoding the **SIB (Scale-Index-Base)** byte.
*   **The Trap**: Certain registers (like ``RSP`` and ``RBP``) have special meanings in the ModRM/SIB bytes.
*   **The Workaround**: ``setup_mem`` contains highly specific logic to detect these hardware quirks and inject zero-displacements to satisfy the silicon logic.

.. note::
   **Historical Lore: The 1978 Ghost**
   
   The ModRM byte was designed for the Intel 8086 in 1978. Every subsequent expansion (386, x86_64) had to work around its 8-register legacy. MIR's backend is effectively communicating with logic designed over 45 years ago.

4. The Endianness Challenge
---------------------------

MIR achieves **Architecture Independence** by treating all registers as 64-bit containers.

*   **Internal Compass**: ``_MIR_addr_offset`` dynamically detects if the host is Little Endian or Big Endian.
*   **The Offset**: On Big Endian systems, it provides the necessary displacement (e.g., 7 bytes for an ``I8``) so that byte-level operations target the correct "lane" in the 64-bit register.

.. _gen_complexity:

5. Computational Complexity
---------------------------

MIR's backend is designed for high-performance, linear-time generation.

5.1 Time Complexity
~~~~~~~~~~~~~~~~~~~

*   **Machinization**: :math:`O(N)` where :math:`N` is the number of instructions.
*   **Translation**: :math:`O(N \times P)` where :math:`P` is the (constant) number of patterns per opcode.
*   **Branch Relaxation**: :math:`O(L)` where :math:`L` is the number of labels.
*   **Rebasing**: :math:`O(R)` where :math:`R` is the number of relocations.

5.2 Memory Complexity
~~~~~~~~~~~~~~~~~~~~~

*   **Code Buffer**: :math:`O(N)` bytes.
*   **Relocation Table**: :math:`O(J)` where :math:`J` is the number of jumps.

.. _gen_relocation_lore:

6. The Relocation Lore: Patching the Universe
---------------------------------------------

MIR performs micro-second linking inside process memory.
*   **The Table**: It tracks "Holes" in the machine code.
*   **The Rebase**: Once the code's final address is known, ``target_rebase`` patches these holes.

.. note::
   **Historical Lore: The Linker's Burden**
   
   In the 1970s, Linkers were separate programs that took minutes to run. MIR performs the same task in microseconds, allowing for the extreme "Lazy" compilation modes that give the project its speed.

**Summary**: Backends are the bridge between the logical and the physical. MIR provides a unified interface to the hardware while respecting the deep historical legacies of every supported architecture.