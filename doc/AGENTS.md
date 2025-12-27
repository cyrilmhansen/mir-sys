Detailed Chapter Breakdowns

Here is what should go into each file to achieve that "Literate Programming" feel.
Book I: The Abstract Machine

01_philosophy.rst - The Context

    The Universal Module: How MIR_module_t mimics a C translation unit.

    The Context State: Why MIR_context_t exists (isolation, thread-safety) and how it manages memory allocators.

    The Dual-Mode Promise: The design decision to support both immediate Interpretation (start fast) and JIT Compilation (run fast).

02_the_ir_spec.rst - The Instruction Set

    The Type System: Why MIR only needs i64, f, d, and p. How simpler types (i8, i16) are implicitly promoted.

    Memory vs. Registers: Explaining the operand types (MIR_OP_REG vs MIR_OP_MEM).

    Instruction Classes:

        Data Movement (MOV, EXT)

        Arithmetic (Integer vs Float)

        Control Flow & Branches (The compare-and-branch architecture).

    The "Unspec" Instruction: The escape hatch for target-specific assembly.

03_c_frontend.rst - C2MIR (The Parser)

    Macro-Expansion Strategy: How the preprocessor in mir/c2mir/ works.

    AST Construction: Mapping C syntax trees to MIR instructions.

    Headers & Environment: How MIR simulates a C standard library environment in memory.

Book II: Execution & Optimization

04_the_interpreter.rst - Immediate Execution

    Direct Threaded Code: Analyzing mir-interp.c. How the interpreter flattens the graph for speed.

    The Value Union: How MIR_val_t acts as the generic container for all data types.

    The Thunk/Shim Bridge:

        The Problem: C compiled code calling Interpreted MIR code (and vice versa).

        The Solution: _MIR_get_interp_shim and how it marshals arguments dynamically.

05_jit_pipeline.rst - The Generator (mir-gen.c)

    Simplification Pass: Removing sugar. Converting complex memory operands into explicit address calculations.

    Control Flow Graph (CFG): Basic block discovery and edge creation.

    SSA Construction: The conversion to Single Static Assignment form (Phis and renaming).

    Optimization Passes:

        Dead Code Elimination (DCE).

        Global Value Numbering (GVN).

        Loop Invariant Code Motion (LICM).

06_register_allocation.rst - The RA Beast

    Prioritized Linear Scan: Why MIR uses this over Graph Coloring (speed vs. quality trade-off).

    Spilling Strategy: How the generator decides what goes to the stack when registers run out.

    Coalescing: Merging virtual registers to reduce moves.

Book III: The 64-bit Trinity

07_target_architecture.rst - x86_64, ARM64, RISC-V

    The Machinize Phase: The final transformation where abstract MIR instructions become target-specific instructions.

    Instruction Selection:

        x86_64 CISC approach: Complex addressing modes.

        RISC-V/ARM64 approach: Load/Store architecture constraints.

    Prologues & Epilogues: Visualizing the stack frame for each architecture.

        Leaf function optimizations.

        Red Zone handling (x86_64).

08_abi_and_varargs.rst - The ABI Nightmare

    Calling Conventions:

        System V (Linux) vs MS ABI (Windows).

        The specific registers used for argument passing on the three targets.

    The Varargs Problem:

        Why va_arg is the hardest thing to JIT.

        x86_64: The complex register save area (floating point vs integer split).

        RISC-V: The simplified approach.

        Implementation: Deep dive into VA_LIST_IS_ARRAY_P and the builtin trampolines.

Book IV: Appendices

09_missing_links.rst - Future Directions

    Concurrency Primitives:

        The lack of ATOMIC_CAS, ATOMIC_LOAD, and FENCE.

        Why this prevents implementing a Go or Java runtime efficiently on MIR today.

    Endianness:

        How MIR currently assumes the host endianness.

        The challenges of cross-compilation (e.g., generating s390x bytecode on x86).

    Stack Unwinding:

        Lack of .eh_frame and DWARF CFI generation.

        Impact on C++ exceptions and debugging tools.

    Vectorization: Lack of SIMD (AVX/NEON/RVV) support.

api_reference.rst

    The raw Doxygen output (classes, structs, functions) linked via the narrative above.

Suggested Action

If this structure looks good to you, you can start populating 08_abi_and_varargs.rst first, as that contains the most complex logic in the codebase (mir-gen-x86_64.c vs mir-gen-aarch64.c) and usually requires the most explanation for new contributors.
