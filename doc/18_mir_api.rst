MIR Construction and Management API
===================================

This section of ``mir.h`` outlines the public API for manipulating MIR's AST (Abstract Syntax Tree) and generating code. This API is the primary interface for any frontend compiler or tool interacting with MIR.

Here is an overview of the key components and functions, structured by their role in the compilation lifecycle.

1. Context and Environment
--------------------------

The **MIR Context** (``MIR_context_t``) is the central object that holds all state for a compilation session.

*   **Initialization**:
    *   ``MIR_init()``: Creates a standard context.
    *   ``MIR_init2(alloc, code_alloc)``: Creates a context with custom memory allocators (for internal data) and executable memory allocators (for generated machine code). This is crucial for sandboxed environments or specific OS requirements.
*   **Finalization**:
    *   ``MIR_finish(ctx)``: Destroys the context and frees all associated memory.
*   **Error Handling**:
    *   ``MIR_set_error_func()``: Sets a custom callback for fatal errors.
*   **Version Check**: ``_MIR_get_api_version()`` ensures the header matches the library version.

2. Module Management
--------------------

MIR code is organized into **Modules**. A module acts as a translation unit (like a ``.c`` file).

*   ``MIR_new_module(ctx, name)``: Starts defining a new module.
*   ``MIR_finish_module(ctx)``: Finalizes the current module.
*   ``MIR_get_module_list(ctx)``: Returns the list of all loaded modules.

3. Item Creation (The AST Nodes)
--------------------------------

Modules contain **Items**. Items can be functions, data, prototypes, or symbol declarations.

3.1 Functions and Prototypes
~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Functions are the executable units. Prototypes define calling conventions.

*   **Functions**:
    *   ``MIR_new_func()`` / ``MIR_new_func_arr()``: Create a new function definition. ``_arr`` version takes arguments as an array struct, while the other is variadic.
    *   ``MIR_new_vararg_func()``: Create a variadic function.
    *   ``MIR_finish_func(ctx)``: Finalizes the current function body (checks validity of instructions).
*   **Prototypes**:
    *   ``MIR_new_proto()``: Defines a function signature (used for ``CALL`` instructions).

3.2 Data and Memory
~~~~~~~~~~~~~~~~~~~

*   ``MIR_new_data()``: Creates initialized binary data (e.g., ``static int x = 5``).
*   ``MIR_new_string_data()``: Creates a C-string constant.
*   ``MIR_new_bss()``: Creates uninitialized zeroed memory (BSS section).

3.3 Linkage and Symbols
~~~~~~~~~~~~~~~~~~~~~~~

*   ``MIR_new_import()``: Declares a symbol needed from another module.
*   ``MIR_new_export()``: Declares a symbol visible to other modules.
*   ``MIR_new_forward()``: Forward declaration within the module.

4. Instruction Construction
---------------------------

The body of a function is a list of **Instructions**.

*   ``MIR_new_insn(ctx, opcode, ...)``: Creates a new instruction with variadic operands.
*   ``MIR_new_insn_arr()``: Same, but takes an array of ``MIR_op_t``.
*   **Insertion**:
    *   ``MIR_append_insn()`` / ``MIR_prepend_insn()``: Add to start/end of function.
    *   ``MIR_insert_insn_after()`` / ``MIR_insert_insn_before()``: Insert relative to another instruction.
*   ``MIR_remove_insn()``: Deletes an instruction.

4.1 Specialized Instruction Constructors
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Some instructions have dedicated helpers for convenience:

*   ``MIR_new_label()``: Creates a branch target label.
*   ``MIR_new_call_insn()``: Creates a function call.
*   ``MIR_new_ret_insn()``: Creates a return instruction.

5. Operand Construction
-----------------------

Instructions operate on **Operands**.

*   **Registers**: ``MIR_new_reg_op()``.
*   **Integers**: ``MIR_new_int_op()`` (signed), ``MIR_new_uint_op()`` (unsigned).
*   **Floats**: ``MIR_new_float_op()``, ``MIR_new_double_op()``.
*   **Memory**: ``MIR_new_mem_op()`` (creates a memory reference ``[base + index*scale + disp]``).
*   **Labels/Refs**: ``MIR_new_label_op()`` (jump target), ``MIR_new_ref_op()`` (refer to a function/data item).

6. Registers and Variables
--------------------------

*   ``MIR_new_func_reg()``: Creates a new virtual register (local variable) for a function.
*   ``MIR_reg()``: Looks up a register by name.

7. Execution and Linking
------------------------

Once the AST is built:

*   **Linking**: ``MIR_load_module()`` loads a module into the context. ``MIR_link()`` resolves symbols between loaded modules.
*   **Generation**: ``MIR_gen()`` compiles a specific function item to machine code.
*   **Interpreter**: ``MIR_interp()`` executes a function using the built-in interpreter (slower but doesn't require code gen).

Internal & Helper Functions
~~~~~~~~~~~~~~~~~~~~~~~~~~~

The header also exposes several helper functions (prefixed with ``_MIR`` or utility functions like ``MIR_op_mode_name``) used for debugging, dumping MIR to text (``MIR_output``), or handling target-specific ABI details (thunks, wrappers). These are typically used by the generator backend rather than the frontend user.
