The Bridge: Thunks & Shims
==========================

One of MIR's most powerful features is its ability to interact seamlessly with compiled C code. This interoperability is handled by two low-level mechanisms: **Thunks** and **Shims**.

The Interface Problem
--------------------
Compiled C code expects to call a function via a standard machine address (a function pointer) using the host's Calling Convention (ABI). However, MIR functions might be:
1.  **Interpreted**: Requiring argument marshaling into a virtual register file.
2.  **Uncompiled**: Requiring lazy JIT generation upon the first call.
3.  **Relocated**: Moving in memory after a re-compilation.

Thunks: Stable Entry Points
---------------------------
A **Thunk** is a tiny, mutable piece of machine code that acts as a permanent proxy for a MIR function.

- **Purpose**: When you create a MIR function, MIR allocates a thunk address. Even if the function moves or changes from interpreted to JITed, the thunk address remains constant.
- **Mechanism**:
    - `_MIR_get_thunk`: Allocates a small executable buffer containing a jump instruction.
    - `_MIR_redirect_thunk`: Atomically updates the jump target in the thunk.
- **Workflow**:
    1.  C code calls the **Thunk Address**.
    2.  The thunk performs an immediate jump to the **Actual Implementation** (Interpreter shim or JIT code).

Shims: The Interpreter Bridge
-----------------------------
Because the Interpreter (`mir-interp.c`) is a C function, it cannot be called directly by other C code using the target's standard ABI (as it needs to know which MIR function to run and where the virtual stack is).

The **Interpreter Shim** bridges this gap:
- **Implementation**: `_MIR_get_interp_shim` (target-specific, e.g., `mir-x86_64.c`).
- **Process**:
    1.  **Save State**: The shim saves all C argument registers to the stack.
    2.  **Marshal**: it packs these registers into an array of `MIR_val_t`.
    3.  **Invoke**: It calls the internal interpreter function with the `ctx`, `func_item`, and the value array.
    4.  **Return**: It extracts the result from `MIR_val_t` back into the appropriate C return register (e.g., `RAX` or `X0`).

Wrappers: Lazy JIT Generation
-----------------------------
MIR supports **Lazy Compilation**, where a function is only JITed the first time it is called. This is implemented via **Wrappers**.

- **Mechanism**:
    1.  Initially, the function's Thunk is redirected to a **Wrapper**.
    2.  When called, the Wrapper invokes a C handler (e.g., `generate_func_and_redirect`).
    3.  The handler triggers the JIT compiler (`MIR_gen`).
    4.  The handler then uses `_MIR_redirect_thunk` to point the thunk directly to the new machine code.
    5.  Finally, the handler executes the code. Subsequent calls bypass the wrapper and go straight to the machine code.

Complexity Analysis
-------------------

- **Memory**:
    - **Thunk**: ~5-16 bytes per function.
    - **Shim**: ~50-200 bytes per function (only if interpreted).
- **Time**:
    - **JIT Call**: $O(1)$ - adds exactly one machine jump (`jmp`) overhead.
    - **Interpreted Call**: $O(1)$ - adds marshaling overhead (proportional to the number of arguments).
    - **Lazy JIT**: $O(N)$ on the *first* call (where $N$ is compilation time), then $O(1)$ thereafter.

API Reference
-------------
.. doxygenfunction:: _MIR_get_thunk
   :project: MIR

.. doxygenfunction:: _MIR_redirect_thunk
   :project: MIR

.. doxygenfunction:: _MIR_get_interp_shim
   :project: MIR
