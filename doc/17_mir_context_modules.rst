MIR Context and Modules
=======================

This section of the code defines the top-level organizational structures of the MIR environment. To understand how MIR works, one must understand the hierarchy: **Context contains Modules, and Modules contain Items.**

1. The MIR Module (``struct MIR_module``)
-----------------------------------------

A **Module** in MIR serves as a container for related code and data, conceptually similar to a "Translation Unit" in C or a "Module" in LLVM. It creates a scope for functions and global variables.

### Structure Breakdown

.. code-block:: c

    struct MIR_module {
      void *data;                  /* Generic user data */
      const char *name;            /* Unique identifier for the module */
      DLIST (MIR_item_t) items;    /* The list of functions, globals, etc. */
      DLIST_LINK (MIR_module_t) module_link; /* Linkage to other modules */
      uint32_t last_temp_item_num; /* Internal counter for anonymous items */
    };

1.  **``void *data``**: A generic pointer allowing the API user (the compiler writer) to attach custom metadata to a module. MIR itself does not touch this; it is for your use (e.g., tracking source file paths or frontend-specific AST nodes).
2.  **``const char *name``**: The name of the module. This is used for debugging dumps and error messages.
3.  **``DLIST (MIR_item_t) items``**: This is the core content of the module. It is a doubly-linked list containing **Items**. These items are the functions, globals, prototypes, imports, and exports that belong to this module.
4.  **``DLIST_LINK (MIR_module_t) module_link``**: This contains the ``prev`` and ``next`` pointers that allow this module to exist inside a list of modules managed by the **Context**.
5.  **``uint32_t last_temp_item_num``**: An internal counter used by the generator. When the compiler needs to create an anonymous string literal or a temporary storage slot during compilation, it uses this counter to generate a unique internal name (e.g., ``.lc0``, ``.lc1``) to prevent name collisions within the module.

2. List Generation (``DEF_DLIST``)
----------------------------------

.. code-block:: c

    /* Definition of double list of MIR_module_t type elements */
    DEF_DLIST (MIR_module_t, module_link);

MIR relies heavily on C macros to generate type-safe data structures.

*   The ``DEF_DLIST`` macro expands into a full struct definition and helper functions for a **Doubly-Linked List of Modules**.
*   It tells the system: "Create a list type where the payload is ``MIR_module_t``, and the linkage pointers are stored in the field named ``module_link`` within that payload."
*   This allows the ``MIR_context`` to hold a list of all loaded modules.

3. The MIR Context (``MIR_context``)
------------------------------------

.. code-block:: c

    struct MIR_context;
    typedef struct MIR_context *MIR_context_t;

The **Context** is the "Universe" in which MIR operates. It represents the global state of the library instance.

*   **Opaque Handle**: The code provides a forward declaration (``struct MIR_context``) and a pointer type (``MIR_context_t``). This encapsulates the internals, forcing users to interact via the API (e.g., ``MIR_init``, ``MIR_new_module``) rather than accessing struct fields directly.
*   **Responsibilities**:
    1.  **Memory Management**: The context holds the ``MIR_alloc_t`` allocator. All modules, items, and instructions created within a context are allocated using this allocator.
    2.  **Module Registry**: It maintains the doubly-linked list of all loaded ``MIR_module``s.
    3.  **Error Handling**: It stores the error handling callback function (``MIR_error_func_t``).
    4.  **Environment**: It acts as the container for "Environment" modules (built-ins) and manages the state of the parser and generator.

Hierarchy Summary
-----------------

1.  **``MIR_context_t``** (The Universe)
    *   Contains a list of **``MIR_module_t``**
2.  **``MIR_module_t``** (The Translation Unit)
    *   Contains a list of **``MIR_item_t``**
3.  **``MIR_item_t``** (The Code/Data)
    *   Contains the actual Functions, Globals, or Prototypes.
