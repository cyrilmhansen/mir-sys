Embedding MIR: The Orchestrator's Guide
================================================================================

Welcome to the **Conductor's Podium**.

Embedding MIR into your application is not just about linking a library; it is about building a dynamic, self-evolving organism within your process. This chapter explores how to initialize the MIR universe, populate it with logic, and bridge the gap between static host code and dynamic JIT-compiled instructions.

1. The Primal Spark: ``MIR_init``
--------------------------------------------------------------------------------

Before any instruction can be executed, you must create a **Context**. The context is the container for all MIR modules, strings, and global state.

.. code-block:: c

   MIR_context_t ctx = MIR_init();

*   **Isolation**: Each context is independent. You can run multiple MIR instances in different threads without them interfering with each other.
*   **Memory Management**: MIR uses its own internal allocators within the context to ensure high-speed allocations for tiny objects (like IR nodes).

2. Creating the World: Modules and Functions
--------------------------------------------------------------------------------

Once the context is ready, you build your logic hierarchy: **Module** -> **Function** -> **Instruction**.

.. code-block:: c

   MIR_module_t module = MIR_new_module(ctx, "my_logic");
   MIR_item_t func_item = MIR_new_func(ctx, "add", 1, &res_type, 2, 
                                       MIR_T_I64, "a", MIR_T_I64, "b");
   
   // Get the function object to add instructions
   MIR_func_t func = func_item->u.func;
   MIR_reg_t a = MIR_reg(ctx, "a", func);
   MIR_reg_t b = MIR_reg(ctx, "b", func);
   MIR_reg_t res = MIR_new_func_reg(ctx, func, MIR_T_I64, "res");

   MIR_append_insn(ctx, func_item, 
                   MIR_new_insn(ctx, MIR_ADD, MIR_new_reg_op(ctx, res), 
                                MIR_new_reg_op(ctx, a), MIR_new_reg_op(ctx, b)));
   MIR_append_insn(ctx, func_item, 
                   MIR_new_insn(ctx, MIR_RET, MIR_new_reg_op(ctx, res)));
   
   MIR_finish_func(ctx);
   MIR_finish_module(ctx);

3. The Bridge: Foreign Function Interface (FFI)
--------------------------------------------------------------------------------

A JIT compiler is useless if it cannot talk to the host. MIR provides a robust **FFI** mechanism.

3.1 Importing Host Functions
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

To call a C function (like ``printf``) from MIR:

1.  Create an **Import** item in your MIR module.
2.  Use ``MIR_load_external`` to map the MIR name to the physical C function address.

.. code-block:: c

   MIR_new_import(ctx, "printf");
   MIR_load_external(ctx, "printf", &printf);

3.2 Calling MIR from the Host
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

To call a dynamic MIR function from your static C code, you have two choices:

1.  **The Interpreter**: Slow but simple.

    .. code-block:: c

       MIR_val_t results[1], args[2];
       args[0].i = 10; args[1].i = 20;
       MIR_interp(ctx, func_item, results, 2, args[0], args[1]);

2.  **The JIT Pointer**: Fast.

    .. code-block:: c

       typedef int64_t (*add_func_t)(int64_t, int64_t);
       add_func_t fn_ptr = (add_func_t)MIR_gen(ctx, func_item);
       int64_t sum = fn_ptr(10, 20);

.. _embedding_arcane:

4. Arcane Machinery: Shims and Thunks
--------------------------------------------------------------------------------

When the host calls a dynamic function, it isn't just a simple jump. There is an invisible layer of **Arcane Machinery** at work.

4.1 The Interpreter Shim
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

If you call an interpreted function through a physical function pointer, MIR must generate an **Interpreter Shim** on the fly.

*   **The Problem**: The CPU expects a standard ABI (stack frames, register values). The interpreter needs a ``MIR_val_t`` array.
*   **The Solution**: ``_MIR_get_interp_shim`` (e.g., in ``mir-x86_64.c``) emits raw machine code bytes that:
    1.  Save all caller registers to a temporary buffer.
    2.  Call the C-based ``MIR_interp`` function.
    3.  Load the results back into the correct ABI return registers.
    4.  Return to the caller as if nothing happened.

4.2 The Lazy Thunk
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

For lazy JIT compilation, MIR uses **Thunks**.

*   **The Ghost**: When a module is loaded but not yet compiled, its function pointers point to a "Thunk"—a tiny trampoline.
*   **The Trap**: When called, the thunk "traps" execution and calls the JIT generator.
*   **The Patch**: The generator compiles the function and then **patches** the thunk (using ``_MIR_change_code``) to jump directly to the new native code. All subsequent calls bypass the generator entirely.

.. note::
   **Historical Lore: Self-Modifying Code**
   
   In the 1980s, self-modifying code was common in games but is now highly restricted by modern OS security (W^X - Write XOR Execute). MIR navigates this by carefully managing memory protections and using architecture-specific "Fence" instructions (like ``ISB`` on ARM or ``MFENCE`` on x86) to flush the CPU's instruction cache after a patch.

5. Computational Complexity
--------------------------------------------------------------------------------

*   **Context Init (``MIR_init``)**: :math:`O(1)`.
*   **Instruction Append**: :math:`O(1)` (Adding to a doubly-linked list).
*   **Linking (``MIR_link``)**: :math:`O(I + E)` where :math:`I` is the number of imports and :math:`E` is the number of exports. It involves multiple hash table lookups to resolve names.
*   **Memory**:
    *   **IR Storage**: Proportional to the number of instructions.
    *   **String Pool**: :math:`O(S)` where :math:`S` is the total length of all unique identifiers and string literals.

Summary: Embedding MIR allows you to turn your application into a high-performance compiler host. Whether you are building a scripting language runtime or an optimized database query engine, MIR provides the physical foundation for your virtual dreams.
