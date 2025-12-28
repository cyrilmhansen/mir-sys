The Optimization Forge: Simplification & Inlining
=================================================

We have crossed the threshold into the realm of **Optimization**. The code documented here is the engine of techniques like **Value Numbering** and **Simplification**, which prepare the MIR for efficient machine code generation.

1. The Optimization Forge: Value Numbering (VN)
-----------------------------------------------

The core of local optimization in MIR is **Value Numbering**.

*   **The Idea:** If I calculate ``a + b`` and store it in ``x``, and then I calculate ``a + b`` again later (without ``a`` or ``b`` changing), I shouldn't re-do the math. I should just reuse ``x``.
*   **The Mechanism:**
    *   ``val_t``: This struct represents a "computation". It holds the opcode (e.g., ``ADD``), the type (e.g., ``I64``), and the operands (``op1``, ``op2``).
    *   ``val_hash`` / ``val_eq``: These allow us to store computations in a hash table (``val_tab``).
    *   ``vn_add_val``: This is the broker. You give it a computation (``a + b``).
        *   If it's already in the table, it returns the *existing* temporary register holding the result.
        *   If it's new, it creates a *new* temporary register, records the computation, and returns the new register.

This simple mechanism automatically eliminates redundant calculations (**Common Subexpression Elimination** or CSE) within a basic block.

2. The Simplifier's Core: ``simplify_func``
-------------------------------------------

This function is the **Standardizer**. It walks through every instruction and operand, enforcing strict rules for the backend.

2.1 Operand Canonicalization
~~~~~~~~~~~~~~~~~~~~~~~~~~~~

*   **Immediate-to-Immediate Moves:** If you try to move a constant ``10`` into another constant ``20``, it asserts failure. You can't change the laws of physics.
*   **Memory-to-Memory Moves:** Most CPUs cannot move data directly from RAM to RAM. ``simplify_func`` detects ``MOV mem, mem`` and injects a temporary register: ``MOV temp, mem; MOV mem, temp``.
*   **Complex Addressing:** It decomposes complex memory operands (``base + index * scale + disp``) into simpler arithmetic instructions if the target architecture or the specific instruction doesn't support them natively.
*   **String Literals:** If it sees a string literal operand (``MIR_OP_STR``), it:
    1.  Creates a new global string data item (``MIR_new_string_data``).
    2.  Replaces the operand with a **Reference** to that global (``MIR_new_ref_op``).
    This moves string data out of the code stream and into the data segment.

2.2 Control Flow Optimization
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

*   **Dead Code Elimination:** It removes trivial jumps to the immediately following instruction (e.g., ``JMP L1; L1: ...``).
*   **Branch Reversal:** It optimizes sequences like ``BCond L1; JMP L2; L1: ...`` into ``BNCond L2; L1: ...`` to improve code locality.
*   **Label Sweeping:** ``remove_unused_and_enumerate_labels`` prunes labels that are never targeted by a jump, keeping the label namespace clean and dense.

2.3 Stack Optimization (Alloca)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

``func_alloca_features`` analyzes stack usage.
*   **Consolidation:** Adjacent ``ALLOCA`` instructions with constant sizes are merged into a single allocation. Why grow the stack ten times by 8 bytes when you can grow it once by 80 bytes?
*   **Alignment:** It ensures allocations respect the natural alignment of the machine (``natural_alignment``).

3. The Inlining Engine
----------------------

MIR supports function inlining to eliminate call overhead.

*   **Limits:** It respects ``MIR_MAX_INSNS_FOR_INLINE`` to prevent code bloat.
*   **Mechanism:** ``process_inlines`` recursively inlines functions marked with ``MIR_INLINE`` or ``MIR_call_code_p``.
    *   It renames registers in the callee to avoid collisions with the caller (``rename_regs``).
    *   It maps arguments to the caller's variables.
    *   It handles ``RET`` instructions by turning them into jumps to the end of the inlined block.

4. The Return Fixer: ``make_one_ret``
-------------------------------------

Functions in MIR can have multiple exit points (``RET``). But many physical machine ABIs and internal analyses prefer a **Single Point of Exit**.

*   **The Transformation:**
    1.  It creates a new label: ``L_return``.
    2.  It finds every ``RET`` instruction in the function.
    3.  It replaces the ``RET val`` with ``MOV ret_reg, val; JMP L_return``.
    4.  It places the single, true ``RET ret_reg`` instruction at ``L_return``.

This canonicalization simplifies register allocation and liveness analysis.

**Summary:** We are seeing the "middle-end" of the compiler in action. It is scrubbing, polishing, and restructuring the code. It eliminates redundancy, enforces structural invariants, and prepares the IR for the final translation to machine code.
