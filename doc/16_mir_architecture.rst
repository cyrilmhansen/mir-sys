MIR Instruction Architecture & Data Structures
============================================

This chapter details the internal C structures used to represent the MIR Intermediate Representation. Understanding these structures is essential for traversing the IR, writing optimization passes, or interacting with the generator internals.

It also describes the **Module Item System**. In MIR, a **Module** is essentially a container for a doubly-linked list of **Items**. An **Item** is a polymorphic entity: it can be a function, a variable definition (data), a memory reservation (bss), a function prototype, or a symbol declaration (import/export).

1. MIR Instruction Architecture
-------------------------------

1.1 Operands (``MIR_op_t``)
~~~~~~~~~~~~~~~~~~~~~~~~~~~

In MIR, every instruction consists of an opcode and a list of operands. An operand is a tagged union that can hold a register, a memory reference, a constant, or a label.

The ``mode`` field in ``MIR_op_t`` determines which field of the union is valid.

.. list-table::
   :header-rows: 1
   :widths: 20 20 60

   * - Mode Enumerator
     - C Type in Union
     - Description
   * - **``MIR_OP_UNDEF``**
     - N/A
     - Undefined or uninitialized operand.
   * - **``MIR_OP_REG``**
     - ``MIR_reg_t reg``
     - A named register (variable) created by the user.
   * - **``MIR_OP_VAR``**
     - ``MIR_reg_t var``
     - **Internal Use.** A pseudo-register or hard-register used during code generation.
   * - **``MIR_OP_INT``**
     - ``int64_t i``
     - 64-bit signed integer immediate.
   * - **``MIR_OP_UINT``**
     - ``uint64_t u``
     - 64-bit unsigned integer immediate.
   * - **``MIR_OP_FLOAT``**
     - ``float f``
     - Single-precision floating point immediate.
   * - **``MIR_OP_DOUBLE``**
     - ``double d``
     - Double-precision floating point immediate.
   * - **``MIR_OP_LDOUBLE``**
     - ``long double ld``
     - Long double immediate.
   * - **``MIR_OP_REF``**
     - ``MIR_item_t ref``
     - Reference to a MIR Item (Function, Proto, Data, Import, etc.).
   * - **``MIR_OP_STR``**
     - ``MIR_str_t str``
     - A string literal immediate.
   * - **``MIR_OP_MEM``**
     - ``MIR_mem_t mem``
     - A complex memory address (``base + index * scale + disp``).
   * - **``MIR_OP_VAR_MEM``**
     - ``MIR_mem_t var_mem``
     - **Internal Use.** Memory operand using internal variable indices.
   * - **``MIR_OP_LABEL``**
     - ``MIR_label_t label``
     - Reference to a label instruction (jump target).

The Operand Structure
^^^^^^^^^^^^^^^^^^^^^

.. code-block:: c

    typedef struct {
      void *data;                 /* Aux data for optimizer/generator (e.g. liveness info) */
      MIR_op_mode_t mode : 8;     /* The active mode (discriminator) */
      MIR_op_mode_t value_mode : 8; /* Internal: Used to track value types during generation */
      union {
        MIR_reg_t reg;            /* MIR_OP_REG */
        int64_t i;                /* MIR_OP_INT */
        /* ... other union members mapping to modes ... */
        MIR_mem_t mem;            /* MIR_OP_MEM */
        MIR_label_t label;        /* MIR_OP_LABEL */
      } u;
    } MIR_op_t;

1.2 Instructions (``MIR_insn_t``)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

MIR instructions are stored as a **Doubly-Linked List**. This allows for O(1) insertion and deletion of instructions, which is critical for optimization passes.

The Structure
^^^^^^^^^^^^^

The ``MIR_insn`` structure utilizes the "Flexible Array Member" C idiom. While ``ops`` is declared as size ``1``, the actual memory allocated for an instruction is ``sizeof(MIR_insn) + sizeof(MIR_op_t) * (nops - 1)``.

.. code-block:: c

    struct MIR_insn {
      void *data;                  /* Aux data (e.g., basic block info, analysis results) */
      DLIST_LINK (MIR_insn_t) insn_link; /* Pointers to prev/next instructions */
      MIR_insn_code_t code : 32;   /* The Opcode (e.g., MIR_ADD, MIR_MOV) */
      unsigned int nops : 32;      /* The number of operands */
      MIR_op_t ops[1];             /* The start of the operand array */
    };

Labels (``MIR_label_t``)
^^^^^^^^^^^^^^^^^^^^^^^^

In MIR, a Label is **not** a separate data type. It is simply a ``MIR_insn`` where ``code == MIR_LABEL``.

*   ``typedef struct MIR_insn *MIR_label_t;``
*   Variables of type ``MIR_label_t`` point to specific instructions in the stream.
*   Operands of mode ``MIR_OP_LABEL`` hold pointers to these instructions.

1.3 Variables and Strings
~~~~~~~~~~~~~~~~~~~~~~~~~

``MIR_var_t``
^^^^^^^^^^^^^

This structure defines a local variable or argument within a function signature. It is not used inside the instruction stream (where ``MIR_reg_t`` is used instead), but rather for defining Function and Prototype interfaces.

*   ``type``: The MIR type (e.g., ``MIR_T_I32``).
*   ``name``: The string name of the variable.
*   ``size``: Used only if the type is ``MIR_T_BLK`` (block size in bytes).

``MIR_str_t``
^^^^^^^^^^^^^

A simple string wrapper carrying the length to avoid repeated ``strlen`` calls during processing.

*   ``len``: Length of string.
*   ``s``: Pointer to character data.

1.4 Helper Macros
~~~~~~~~~~~~~~~~~

The header defines several macros to assist with generic type generation (X-Macros).

*   ``DEF_VARR(T)``: Defines a Variable Array (Vector) type for type T.
*   ``DEF_DLIST_LINK(T)``: Defines the ``prev``/``next`` pointers for a Doubly Linked List of type T.
*   ``DEF_DLIST(T, link)``: Defines the container struct (head/tail) and helper functions for the list.

**Example:**
``DEF_VARR(MIR_var_t)`` creates a type ``VARR(MIR_var_t)`` which acts as a dynamic array ``std::vector<MIR_var_t>`` in C++.

2. Module Item Architecture
---------------------------

This section details the **Module Item System**.

2.1 The Polymorphic Container: ``MIR_item``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

The ``MIR_item`` structure is the fundamental unit of code or data within a module. It acts as a wrapper that holds metadata (linkage, visibility, type) and contains a union holding the specific data for that type.

Structure Definition
^^^^^^^^^^^^^^^^^^^^

.. code-block:: c

    struct MIR_item {
      void *data;                 /* Generic user data / auxiliary data */
      MIR_module_t module;        /* Back-pointer to the owning module */
      DLIST_LINK (MIR_item_t) item_link; /* Pointers for the module's item list */
      MIR_item_type_t item_type;  /* Discriminator: What kind of item is this? */

      /* Linkage Fields */
      MIR_item_t ref_def;         /* Points to the actual definition (used for Imports/Forwards) */
      void *addr;                 /* Runtime address (machine code or data memory) */

      /* Flags */
      char export_p;              /* True if this item is visible to other modules */
      char section_head_p;        /* True if this item starts a memory section (Data/BSS) */

      /* The Payload */
      union {
        MIR_func_t func;
        MIR_proto_t proto;
        MIR_name_t import_id;
        MIR_name_t export_id;
        MIR_name_t forward_id;
        MIR_data_t data;
        MIR_ref_data_t ref_data;
        MIR_lref_data_t lref_data;
        MIR_expr_data_t expr_data;
        MIR_bss_t bss;
      } u;
    };

Item Types (``MIR_item_type_t``)
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

The ``item_type`` field determines which member of the union ``u`` is valid.

.. list-table::
   :header-rows: 1
   :widths: 25 25 50

   * - Enum Value
     - Payload Field
     - Description
   * - **``MIR_func_item``**
     - ``u.func``
     - A function definition (code).
   * - **``MIR_proto_item``**
     - ``u.proto``
     - A function prototype (signature).
   * - **``MIR_data_item``**
     - ``u.data``
     - Initialized data (arrays, scalars).
   * - **``MIR_bss_item``**
     - ``u.bss``
     - Uninitialized zeroed memory.
   * - **``MIR_import_item``**
     - ``u.import_id``
     - A symbol required from another module.
   * - **``MIR_export_item``**
     - ``u.export_id``
     - A symbol made visible to other modules.
   * - **``MIR_forward_item``**
     - ``u.forward_id``
     - A forward declaration of a symbol within this module.
   * - **``MIR_ref_data_item``**
     - ``u.ref_data``
     - Data initialized with the address of another item.
   * - **``MIR_lref_data_item``**
     - ``u.lref_data``
     - Data initialized with the address of a label.
   * - **``MIR_expr_data_item``**
     - ``u.expr_data``
     - Data initialized by the result of a MIR function.

2.2 Executable Items
~~~~~~~~~~~~~~~~~~~~

Function Definition (``MIR_func``)
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

The heavyweight structure representing a function's logic and signature.

.. code-block:: c

    typedef struct MIR_func {
      const char *name;             /* Function name */
      MIR_item_t func_item;         /* Back-pointer to the container item */

      /* Instruction Streams */
      DLIST (MIR_insn_t) insns;          /* The current instruction list (may be optimized) */
      DLIST (MIR_insn_t) original_insns; /* The original instructions (before optimization) */

      /* Signature */
      uint32_t nres;                /* Number of return values */
      uint32_t nargs;               /* Number of arguments */
      MIR_type_t *res_types;        /* Array of return types */
      char vararg_p;                /* Is this a vararg function? */
      VARR (MIR_var_t) * vars;      /* List of arguments and local variables */

      /* Generator Internals */
      uint32_t last_temp_num;       /* Counter for temporary register generation */
      uint32_t n_inlines;           /* Counter for inlined calls */
      void *machine_code;           /* Pointer to generated native code */
      void *call_addr;              /* Entry point address (may differ from machine_code) */
      struct MIR_lref_data *first_lref; /* Linked list of label references in this func */

      /* Flags & Aux */
      char expr_p;                  /* True if usable as a linker expression */
      char jret_p;                  /* True if uses JRET (Jump Return) */
      VARR (MIR_var_t) * global_vars; /* Global registers tied to hard regs */
      void *internal;               /* Internal optimizer/generator data */
    } *MIR_func_t;

Function Prototype (``MIR_proto``)
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

Defines a call signature. This is used by ``CALL`` instructions to know how to pass arguments and retrieve return values.

.. code-block:: c

    typedef struct MIR_proto {
      const char *name;             /* Prototype name */
      uint32_t nres;                /* Number of results */
      MIR_type_t *res_types;        /* Result types array */
      char vararg_p;                /* Is it vararg? */
      VARR (MIR_var_t) * args;      /* Argument definitions (names may be NULL) */
    } *MIR_proto_t;

2.3 Static Data Items
~~~~~~~~~~~~~~~~~~~~~

Initialized Data (``MIR_data``)
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

Represents binary data embedded in the module (like C ``static int x = 5;``).

.. code-block:: c

    typedef struct MIR_data {
      const char *name;             /* Symbol name (can be NULL) */
      MIR_type_t el_type;           /* Type of elements (e.g., MIR_T_U8) */
      size_t nel;                   /* Number of elements */
      union {
        long double d;              /* Used for alignment */
        uint8_t els[1];             /* The actual data bytes */
      } u;
    } *MIR_data_t;

Uninitialized Data (``MIR_bss``)
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

Represents the BSS section (Block Started by Symbol). Memory is reserved but not stored in the file; it is zero-initialized at runtime.

.. code-block:: c

    typedef struct MIR_bss {
      const char *name;             /* Symbol name (can be NULL) */
      uint64_t len;                 /* Length in bytes */
    } *MIR_bss_t;

2.4 Relocatable Data Items
~~~~~~~~~~~~~~~~~~~~~~~~~~

These structures handle data initialization that depends on addresses known only at runtime or link-time.

Item Reference (``MIR_ref_data``)
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

Initializes memory with the address of another MIR item.

*   **Formula:** ``*load_addr = &ref_item + disp``

.. code-block:: c

    typedef struct MIR_ref_data {
      const char *name;             /* Name of this data item */
      MIR_item_t ref_item;          /* The item whose address we need */
      int64_t disp;                 /* Displacement added to the address */
      void *load_addr;              /* Where to store the resolved address */
    } *MIR_ref_data_t;

Label Reference (``MIR_lref_data``)
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

Initializes memory with the address of a label inside a function (supporting computed gotos or "Labels as Values").

*   **Formula:** ``*load_addr = label_addr [- label2_addr] + disp``

.. code-block:: c

    typedef struct MIR_lref_data {
      const char *name;
      MIR_label_t label;            /* The target label */
      MIR_label_t label2;           /* Optional subtractor label (for relative offsets) */
      int64_t disp;
      void *load_addr;
      /* ... internals ... */
    } *MIR_lref_data_t;

Expression Reference (``MIR_expr_data``)
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

Initializes memory by running a specific MIR function during linking.

*   **Usage:** Complex initialization logic that cannot be expressed by simple relocations.

.. code-block:: c

    typedef struct MIR_expr_data {
      const char *name;
      MIR_item_t expr_item;         /* The function to execute */
      void *load_addr;              /* Where to store the function's return value */
    } *MIR_expr_data_t;
