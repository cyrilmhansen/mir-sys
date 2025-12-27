The Interpreter: Walking the Graph
==================================

The MIR Interpreter (`mir-interp.c`) is the immediate execution engine. It allows code to run the moment it is generated, without waiting for the heavier JIT compilation process.

The Execution Model
-------------------
The interpreter operates directly on the `MIR_insn_t` linked list (see :ref:`anatomy`), but it performs an initial "compilation" to an internal threaded code representation for speed. The core logic is in `mir/mir-interp.c`.

1. Preprocessing (Internal Code Generation)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
Before execution, `generate_icode` flattens the linked list of MIR instructions into a linear array for faster access.

- **Implementation**: `mir/mir-interp.c` -> `generate_icode`.
- **Process**:
    - Flattens `MIR_insn_t` nodes into a `code_varr` array of `MIR_val_t`.
    - Resolves labels to absolute array indices.
    - Pre-calculates certain operand data to minimize runtime overhead.
- **Key Data Structures**:
    - `code_varr`: A `VARR(MIR_val_t)` representing the linearized instruction stream (Internal Code).
- **Complexity**:
    - **Time**: $O(N)$ where $N$ is the number of instructions.
    - **Memory**: $O(N)$ for the instruction array.

2. Stack Allocation
~~~~~~~~~~~~~~~~~~~
The interpreter uses the host C stack for MIR stack frames, ensuring fast allocation and natural recursion.

- **Implementation**: `MIR_interp` entry point.
- **Mechanism**:
    - A `MIR_val_t` array is allocated on the host stack (often implicitly via `alloca` or recursive calls).
    - **Base Pointer (`bp`)**: This pointer acts as the base for the virtual register file of the current function frame.
    - All local variables and virtual registers are accessed relative to `bp`.
- **Complexity**:
    - **Time**: $O(1)$ (stack pointer adjustment).
    - **Memory**: $O(F)$ where $F$ is the size of the function's stack frame (locals + temporaries).

3. Dispatch Loop
~~~~~~~~~~~~~~~~
The heart of the interpreter is the dispatch loop, which executes the linearized code.

- **Direct Threaded Code** (GCC/Clang):
    - Uses the "Labels as Values" extension (`&&label`).
    - `dispatch_label_tab` maps MIR opcodes to the address of the C label handling that instruction.
    - Execution jumps directly from one handler to the next: `goto *pc->a`.
- **Switch Dispatch** (Standard C):
    - A standard `while(1) { switch(*pc) { ... } }` loop.
    - Slower due to bounds checking and branch prediction misses.
- **Data Structures**:
    - `dispatch_label_tab`: Array of code pointers.
    - `pc`: Program counter pointing into `code_varr`.
- **Complexity**:
    - **Time**: $O(1)$ overhead per instruction executed.
    - **Memory**: Constant overhead for the dispatch table.

API Reference
-------------
.. doxygenfunction:: MIR_interp
   :project: MIR

Performance Characteristics
---------------------------
- **Startup**: Instant. No code generation phase.
- **Throughput**: significantly slower than JITed code (approx 10x-50x slower) due to dispatch overhead and lack of register allocation.
- **Use Case**: Bootstrapping, rare execution paths, or environments where JIT (W^X memory) is restricted.