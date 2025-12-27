Modules & Linking
=================

MIR is designed to support modular compilation, similar to C translation units or Java classes. Code is organized into **Modules**, which must be **Linked** together before they can be executed.

The Module Structure
--------------------
A `MIR_module_t` is a named container for MIR items. It acts as a namespace and a unit of compilation.

.. code-block:: c

   MIR_module_t m = MIR_new_module(ctx, "my_module");
   // ... add functions, data, globals ...
   MIR_finish_module(ctx);

Types of Items
~~~~~~~~~~~~~~
A module contains a list of items (`MIR_item_t`):

1.  **Functions** (`MIR_func_item`): Executable code.
2.  **Data** (`MIR_data_item`): Static data (like `.data` or `.rodata` sections).
3.  **Imports** (`MIR_import_item`): Symbols defined elsewhere that this module needs.
4.  **Exports** (`MIR_export_item`): Symbols this module provides to the outside world.
5.  **Forward Declarations** (`MIR_forward_item`): Handling recursion within a module.

Linking
-------
Before code can run, dependencies must be resolved. The `MIR_link` function performs this task.

.. code-block:: c

   MIR_load_module(ctx, m); // Queue module for linking
   MIR_link(ctx, MIR_set_interp_interface, my_import_resolver);

The linking process involves several passes:

1.  **Resolution**: For every `MIR_import_item`, the linker searches for a matching `MIR_export_item` in other loaded modules or the special **Environment Module**.
2.  **External Resolution**: If a symbol isn't found internally, the user-provided `import_resolver` callback is invoked (often wrapping `dlsym` or `GetProcAddress`).
3.  **Simplification**: The linker runs the simplification pass (see :doc:`04_jit_pipeline`) on all functions in the module.
4.  **Interface Setting**: It sets the execution entry point (Interpreter or JIT) for functions.

The Environment Module
----------------------
MIR maintains a hidden internal module called the **Environment**. This acts as a global scope for:

-   Builtin functions.
-   Symbols registered via `MIR_load_external`.

When you call `MIR_load_external(ctx, "printf", printf)`, you are essentially exporting the C `printf` function from the Environment module, making it available to any MIR module that imports "printf".

Complexity Analysis
-------------------
-   **Symbol Lookup**: Uses hash tables (`mir-htab.h`), so resolution is generally $O(1)$ per import.
-   **Linking Time**: $O(N)$ where $N$ is the total size of the modules being linked (dominated by the simplification pass).
-   **Memory**: Proportional to the number of exported symbols (hash table entries).

API Reference
-------------
.. doxygenfunction:: MIR_new_module
   :project: MIR

.. doxygenfunction:: MIR_link
   :project: MIR

.. doxygenfunction:: MIR_load_external
   :project: MIR
