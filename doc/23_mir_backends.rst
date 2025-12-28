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

.. _gen_stack_frame:

1.5 Advanced Machinization: The Stack Frame and Prologue
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Before a function can execute, it must secure its "Living Quarters" in memory. ``target_make_prolog_epilog`` is the **Civil Engineer** that builds the stack frame.

*   **Frame Layout**: The RISC-V frame is 16-byte aligned. It contains:
    1.  **Saved Registers**: Callee-saved registers (like ``S0-S11``) if they are used.
    2.  **Pseudo-registers**: MIR virtual registers that didn't fit in silicon.
    3.  **Return Address (RA)** and **Old Frame Pointer (FP)**.
*   **The Squeeze**: If the frame is small (< 2KB), a single ``ADDI SP, SP, -size`` instruction suffices. If it's larger, the backend must use a temporary register to calculate the new stack pointer.

.. _gen_pattern_matcher:

1.6 The Pattern Matcher: Instruction Selection
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

The heart of ``target_translate`` is a declarative **Pattern Matcher**. Instead of writing thousands of ``if`` statements, the backend uses a `struct pattern` table.

*   **The Grammar**: Patterns use a shorthand:
    *   ``"r r r"``: Three registers (e.g., ``ADD rd, rs1, rs2``).
    *   ``"r r i"``: Two registers and a 12-bit immediate (e.g., ``ADDI rd, rs1, imm``).
    *   ``"C"``: A "Compressed" register (available in the RV64C extension).
*   **The Replacement**: Each pattern has a machine code template (e.g., ``"O33 F0 rd0 rs1 rS2"``). The translator fills in the holes (``rd0``, ``rs1``) with the actual register numbers from the MIR instruction.

.. _gen_branch_relax:

1.7 The Arcane Art: Branch Relaxation and Patching
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

RISC-V instructions are 32 bits wide, and their branch offsets are limited. A standard branch (``BEQ``) can only jump +/- 4KB.

*   **The Problem**: What happens if your function is 100KB long and you need to jump from the start to the end?
*   **The Arcane Solution**: In ``target_translate``, MIR performs **Branch Relaxation**. 
    *   If an offset fits in 12 bits, it emits a single ``BEQ``.
    *   If it doesn't, it "relaxes" the instruction into a complex sequence:
        .. code-block:: text
           BNE next;  # Inverse the condition
           JAL target; # Jump-and-Link (20-bit offset, +/- 1MB)
           next:
    *   If even 1MB isn't enough, it uses an ``AUIPC`` + ``JALR`` sequence to reach the entire 64-bit address space.

.. note::
   **Historical Lore: The Fixed-Width Prison**
   
   In the early days of RISC (like MIPS and early Alpha), compilers had to be extremely clever because every instruction *had* to be exactly 32 bits. This created the "Branch Distance" problem. Modern architectures like RISC-V solve this by allowing the compiler to "relax" a single instruction into a multi-instruction trampoline. MIR automates this tedious process, ensuring that the programmer never has to worry about the physical size of their code.

.. _gen_x86_deep:

5. Deep Dive: The Veteran's CISC Struggle (x86_64)
-------------------------------------------------

While RISC-V is a clean slate, x86_64 is a city built on top of ancient ruins. The implementation in ``mir-gen-x86_64.c`` is a masterpiece of adaptation.

5.1 The Variable-Length Maze
~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Unlike RISC-V's 4-byte grid, x86 instructions can be anywhere from 1 to 15 bytes.

*   **Instruction Density**: A ``RET`` is 1 byte (``0xC3``). A complex move with a 64-bit immediate and a 4-byte displacement can be 10+ bytes.
*   **Time Complexity**: Finding the length of an x86 instruction is :math:`O(L)` where :math:`L` is the length in bytes. Since $L$ is capped at 15, it remains constant-time, but the logic is far more complex than RISC-V's bit-shifting.

5.2 The 2-Operand Conversion (Machinization)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

The x86 backend must perform a high-complexity pass during machinize to handle the 3-to-2 operand conversion.

*   **Memory Complexity**: This pass may double the number of instructions in the IR buffer (:math:`O(2N)`).
*   **The Lore of the Accumulator**: Historically, x86 was an "Accumulator" architecture (everything happened in ``EAX``). Modern x86_64 allows most registers to act as accumulators, but the "Result = Input1 op Input2" restriction persists. MIR's backend hides this 40-year-old limitation from the user.

5.3 The Red Zone and the Stack
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

x86_64 (specifically the System V ABI used on Linux) has a feature called the **Red Zone**.

*   **The Safe Haven**: Functions can use 128 bytes of memory *below* the stack pointer (``RSP``) without allocating it.
*   **The Benefit**: For small leaf functions, MIR can skip the ``SUB RSP, size`` instruction entirely, resulting in extremely fast, "prologue-less" code execution.

.. _gen_relocation_lore:

6. The Relocation Lore: Patching the Universe
---------------------------------------------

Both the Generator and the Interpreter must eventually deal with **Relocations**.

*   **The Absolute Truth**: When you call a function like ``printf``, the JIT doesn't know its final address until the code is actually placed in memory.
*   **The Relocation Table**: MIR maintains a list of "Holes" in the machine code.
*   **Complexity**: ``target_rebase`` is :math:`O(R)` where :math:`R` is the number of relocations. For each entry, it calculates the final destination and "patches" the bytes.

.. note::
   **Historical Lore: The Linker's Burden**
   
   In the 1970s, "Linkers" were separate, heavy programs that ran for minutes to resolve these addresses. MIR performs this same "Linking" process in microseconds, right inside your process memory. It is a testament to how far Moore's Law and algorithmic efficiency have come.

**Summary**: Backends are the bridge between the logical and the physical. Whether it's navigating the fixed-width constraints of RISC-V or the ancient operand rules of x86, MIR provides a unified, high-performance interface to the hardware.



2. The Elite Guard: ARM64 (AArch64)
------------------------------------

ARM64 is the dominant architecture of the mobile world and increasingly the data center. Its implementation in ``mir-gen-aarch64.c`` reflects a modern, efficient RISC design with a few "Elite" twists.

2.1 The Register Wealth
~~~~~~~~~~~~~~~~~~~~~~~

ARM64 provides a generous 31 general-purpose 64-bit registers (``R0-R30``).

*   **Zero Register**: ``R31`` is the "Zero Register" (``ZR``). It always returns 0 when read and discards data when written. MIR uses this to simplify many logic operations.
*   **The Link Register**: ``R30`` (``LR``) holds the return address for function calls. Unlike x86, which pushes the return address to the stack, ARM64 keeps it in a register, saving a memory access for "leaf" functions (functions that don't call other functions).

2.2 Apple Silicon vs. The World
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

The ARM64 backend must navigate a major cultural divide: **Standard ARM64 ABI** vs. **Apple macOS/iOS ABI**.

*   **Variadic Arguments**: On standard Linux/Android ARM64, variadic arguments are passed in a complex structure that includes both integer and floating-point register save areas. On Apple platforms, variadic arguments are **always passed on the stack**.
*   **Stack Alignment**: Apple is stricter about the 16-byte stack alignment. ``mir-gen-aarch64.c`` contains numerous ``#if defined(__APPLE__)`` blocks to handle these bureaucratic differences.

2.3 Immediate Constraints
~~~~~~~~~~~~~~~~~~~~~~~~~

ARM64 instructions are 32 bits wide. This means you cannot fit a 64-bit constant inside a single instruction.

*   **The Solution**: If a constant is small, it uses a simple ``MOV``. If it's large, the backend must use a sequence of ``MOVK`` (Move Keep) instructions to build the value 16 bits at a time, or load it from a **Literal Pool** (a small data area embedded in the code).

3. The Veteran: x86_64 (CISC)
----------------------------

The x86_64 backend (``mir-gen-x86_64.c``) is the most complex, as it must adapt MIR's clean RISC-like IR to the dense, idiosyncratic world of CISC.

3.1 The 2-Operand Straitjacket
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

MIR instructions are 3-operand (``A = B + C``). Most x86 instructions are 2-operand (``A = A + B``).

*   **The Injection**: The x86 backend must frequently inject an extra ``MOV`` to preserve the inputs:
    .. code-block:: assembly
       MOV RAX, RBX  ; A = B
       ADD RAX, RCX  ; A = A + C
*   **The Optimization**: If the Register Allocator was smart enough to assign ``A`` and ``B`` to the same hardware register, the backend identifies the ``MOV RAX, RAX`` and deletes it.

3.2 Complex Addressing (SIB)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~

x86 can calculate complex addresses in a single instruction: ``[base + index * scale + disp]``.

*   **``target_insn_ok_p``**: This function is the "Inspector." It looks at a MIR memory operand and decides if it fits into x86's addressing hardware. If the displacement is too large or the scale is not 1, 2, 4, or 8, the inspector rejects it, forcing the simplifier to break the math down into multiple steps.

4. The Endianness Challenge
---------------------------

MIR's core philosophy is **Architecture Independence**. A major part of this is handling **Endianness**—the order in which bytes are stored in memory.

4.1 The 64-bit Container
~~~~~~~~~~~~~~~~~~~~~~~~

Internally, MIR treats all registers as 64-bit containers. When you load a single byte (``I8``) into a register, it occupies a specific "lane" in that 64-bit slot.

*   **Little Endian (x86, ARM64, RISC-V)**: The byte lives at the **lowest** address (offset 0).
*   **Big Endian (s390x, PowerPC)**: The byte lives at the **highest** address (offset 7).

4.2 The Reality Anchor: ``_MIR_addr_offset``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

This function is MIR's "Internal Compass." It performs a runtime check:
.. code-block:: c
   int v = 1; if (*(char *)&v != 0) // I am Little Endian!

*   **The Adjustment**: If the host is Big Endian, ``_MIR_addr_offset`` returns the necessary displacement (7 for ``I8``, 6 for ``I16``, 4 for ``I32``) to ensure that code generated for a byte load actually points to the correct byte in the 64-bit register.

4.3 Porting to a New Endianness
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

To run MIR on a **Big Endian ARM64** system, the developer would:
1.  Verify the ``_MIR_addr_offset`` logic in the core.
2.  Ensure that the ``target_translate`` phase in the ARM64 backend emits instructions in the correct byte order (ARM64 instructions themselves are almost always Little Endian, even when data is Big Endian!).
3.  Adjust the aggregate passing logic in the ABI handler, as many BE systems reverse the alignment of small structs within a register.

**Summary**: By abstracting away these physical "quirks" into target-specific files while maintaining a unified analytical core, MIR achieves the rare feat of being both highly portable and extremely fast.

