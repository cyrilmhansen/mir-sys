The JIT: Flattening the Graph
=============================

The Just-In-Time (JIT) compiler (`mir-gen.c`) transforms the abstract MIR graph into native machine code. This is where the "speed" of MIR comes from.

The Pipeline
------------
The transformation from `MIR_item_t` to `void*` (function pointer) involves several sophisticated passes.
The primary implementation resides in `mir/mir-gen.c`.

1. Simplification
~~~~~~~~~~~~~~~~~
This is a lowering phase that transforms complex MIR instructions into simpler, canonical forms. It removes "syntactic sugar" to simplify subsequent analysis. This step happens **both** in the JIT and during `MIR_link` for the interpreter.

- **Implementation**: `mir/mir.c` -> `simplify_func`.
- **Core Logic**:
    - **Memory Operand Lowering**: Complex memory operands (e.g., `base + index * scale + disp`) are decomposed into explicit arithmetic instructions (`MIR_ADD`, `MIR_MUL`) using temporary registers.
    - **CSE**: Local Value Numbering is performed to reuse existing calculations for address components.
- **Key Data Structures**:
    - `struct simplify_ctx`: Holds state.
    - `val_tab` (Hash Table): Maps values (instruction/operand combinations) to temporary registers for Value Numbering.
- **Complexity**:
    - **Time**: $O(N)$ where $N$ is the number of instructions.
    - **Memory**: $O(N)$ for temporary instructions and hash table entries.

2. SSA Construction
~~~~~~~~~~~~~~~~~~~
The JIT converts the Control Flow Graph (CFG) into **Static Single Assignment** form, where every variable is assigned exactly once.

- **Implementation**: `mir/mir-gen.c` -> `build_ssa`.
- **Algorithm**: **Braun et al.** "Simple and Efficient Construction of Static Single Assignment Form". This algorithm constructs SSA directly during CFG traversal without computing dominance frontiers, making it highly efficient.
- **Steps**:
    - **Worklist**: Basic blocks are processed in Reverse Post-Order (RPO).
    - **Phi Insertion**: `MIR_PHI` nodes are inserted recursively by searching predecessor blocks when a variable is used but not defined locally.
    - **Renaming**: Variables are versioned (e.g., `reg1` -> `reg1_1`, `reg1_2`) to enforce SSA properties.
- **Key Data Structures**:
    - `curr_cfg` (`struct func_cfg`): The Control Flow Graph.
    - `def_tab`: A hash table mapping variables to their defining instructions during SSA construction.
- **Complexity**:
    - **Time**: $O(N)$ (linear in the size of the program).
    - **Memory**: $O(V)$ where $V$ is the number of variables.

3. Register Allocation (RA)
~~~~~~~~~~~~~~~~~~~~~~~~~~~
MIR uses a **Priority-Based Linear Scan** allocator. This prioritizes compilation speed over the absolute optimality of graph coloring (used by GCC/LLVM), which is a suitable trade-off for a JIT.

- **Implementation**: `mir/mir-gen.c` -> `reg_alloc`.
- **Algorithm**: Linear Scan with **Live Range Splitting**.
- **Steps**:
    - **Live Analysis**: `build_live_ranges` calculates live intervals for virtual registers.
    - **Sorting**: Registers are sorted by priority (loop depth, usage frequency).
    - **Assignment**: Iterates through sorted registers, assigning hardware registers or spilling to stack slots.
    - **Splitting**: Unique to MIR's fast RA, it splits live ranges where a variable may reside in a register for part of its life and on the stack for another, minimizing spill costs.
    - **Rewrite**: Virtual registers are replaced with physical ones.
- **Key Data Structures**:
    - `live_ranges` (`VARR(live_range_t)`): Stores start/end points for variables.
    - `sorted_regs`: Priority-sorted list of registers.
    - `conflict_locs`: Bitmaps tracking hardware register occupancy.
- **Complexity**:
    - **Time**: $O(R \log R)$ due to sorting, where $R$ is the number of virtual registers.
    - **Memory**: $O(R)$ for live ranges.

4. Code Emission
~~~~~~~~~~~~~~~~
The final phase translates the register-allocated MIR instructions into native machine code.

- **Implementation**: `mir/mir-gen.c` -> `target_translate`.
- **Process**:
    - **Machinization**: `target_machinize` handles ABI details and instruction splitting.
    - **Encoding**: A massive switch statement (often in `mir-gen-<target>.c`) maps MIR opcodes to machine byte sequences (e.g., x86_64 ModR/M).
    - **Memory Management**: `_MIR_publish_code` allocates executable memory (using `mmap` with `PROT_WRITE`, then `PROT_EXEC` for W^X security).
- **Key Data Structures**:
    - `code_buffer`: Dynamic byte array for accumulating machine code.
- **Complexity**:
    - **Time**: $O(N)$ where $N$ is the number of machine instructions.
    - **Memory**: $O(M)$ where $M$ is the size of the generated machine code.

Optimization Levels
-------------------
The MIR JIT supports different optimization levels, trading off compilation time for generated code quality.

Level 1 (-O1)
~~~~~~~~~~~~~
At this level, the JIT enables basic optimizations that are relatively cheap to compute but provide significant speedups over `-O0`.

1. **Code Selection (Combine)**
   The `combine` pass merges multiple abstract MIR instructions into a single machine instruction where supported by the target architecture (e.g., merging a load with an arithmetic operation).

   - **Implementation**: `mir/mir-gen.c` -> `combine`.
   - **Logic**: Iterates forward through basic blocks. It tracks variable definitions in `var_refs`. If an instruction uses a variable defined by a previous instruction (and that variable is not used elsewhere), it attempts to merge them (e.g., `mov r1, [mem]; add r0, r0, r1` -> `add r0, r0, [mem]`). It also swaps operands of commutative instructions to find better matches.
   - **Data Structures**: `struct combine_ctx` containing `var_refs` (tracking last definition of variables).
   - **Complexity**:
       - **Time**: $O(N)$ (forward pass).
       - **Memory**: $O(R)$ for tracking variable definitions.

2. **Dead Code Elimination (DCE)**
   Removes instructions whose results are never used and which have no side effects.

   - **Implementation**: `mir/mir-gen.c` -> `dead_code_elimination`.
   - **Logic**: Performs a backward pass over instructions in each basic block. It maintains a `live` bitmap of currently needed variables. Instructions defining variables not in the `live` set (and not flagged as having side effects like calls or memory stores) are deleted.
   - **Data Structures**: `live` (bitmap) representing the set of live variables at the current point.
   - **Complexity**:
       - **Time**: $O(N)$ (backward pass).
       - **Memory**: $O(R)$ for the liveness bitmap.

3. **Loop Analysis**
   The JIT builds a **Loop Tree** to understand the nesting structure of loops.

   - **Implementation**: `mir/mir-gen.c` -> `build_loop_tree`.
   - **Purpose**: While heavy loop optimizations (like LICM) are reserved for `-O2`, the loop tree at `-O1` allows the Register Allocator to prioritize variables used inside loops, minimizing spills in hot paths.
   - **Complexity**:
       - **Time**: $O(N)$ to detect loops.
       - **Memory**: $O(L)$ where $L$ is the number of loops.

The Generator Context
---------------------
The generator maintains its own state, separate from the main `MIR_context_t`. This allows the JIT to be torn down or re-initialized independently.

.. doxygenfunction:: MIR_gen_init
   :project: MIR
.. doxygenfunction:: MIR_gen_finish
   :project: MIR

Target Support
--------------
The JIT backend is modular. It supports:
- **x86_64**: extensive optimization.
- **AArch64** (ARM64): full support.
- **RISC-V** (64-bit): growing support.
- **PPC64 / s390x**: Big-iron support.