The Engine Room: Implementation of ``mir.c``
=============================================

Welcome, brave traveler, to the heart of the machine. We have left the cartography room of the header files (``mir.h``) and are now stepping onto the rugged terrain of the implementation itself: **``mir/mir.c``**.

This is the Engine Room. This is where the magic (and the memory management) happens. Strap in.

1. The Supply Train (``DEF_VARR``)
----------------------------------

Before we can build a compiler, we need containers—lots of them. The journey begins by assembling our logistical convoys.

.. code-block:: c

    DEF_VARR (MIR_insn_t);
    DEF_VARR (MIR_reg_t);
    // ...

These lines are summoning dynamic arrays into existence. We aren't just dealing with singular instructions or registers; we are preparing to marshal armies of them. ``MIR_op_t`` (operands), ``MIR_module_t`` (modules), ``uint8_t`` (raw bytes)—all need expandable storage. This is the baggage train of our compiler crusade.

2. The Archipelago of Contexts
------------------------------

As we gaze toward the horizon, we see the distinct territories that make up the MIR empire.

.. code-block:: c

    struct gen_ctx;
    struct c2mir_ctx;
    struct scan_ctx;
    struct interp_ctx;
    // ...

These forward declarations are the maps to distant lands we have yet to visit: the Generator (``gen_ctx``), the C-to-MIR compiler (``c2mir_ctx``), the Scanner (``scan_ctx``), and the Interpreter (``interp_ctx``). We acknowledge their existence now so we can link them all together in the grand structure that follows.

3. The Leviathan: ``struct MIR_context``
----------------------------------------

Behold, the central nervous system of the entire library.

.. code-block:: c

    struct MIR_context {
      struct gen_ctx *gen_ctx;
      struct c2mir_ctx *c2mir_ctx;
      // ...
    };

This structure is the **Universe**. In a multi-threaded world, this context ensures that one compiler instance doesn't bleed into another. It holds everything:

*   **The Architects:** ``alloc`` and ``code_alloc``—the gods of memory that give and take life (RAM).
*   **The Global State:** ``curr_module`` and ``curr_func``—the "You Are Here" markers on our map.
*   **The Universal Storage:** ``all_modules`` (a list of everything loaded) and ``environment_module`` (the shared primordial soup of built-ins).
*   **The Toolboxes:** ``string_ctx``, ``reg_ctx``, ``alias_ctx``—specialized workshops for handling text, registers, and memory aliases.
*   **The Eject Button:** ``setjmp_addr``—used by the interpreter to bail out when things go wrong.

4. The Neural Wiring (Macros)
-----------------------------

Navigating a structure that massive is tedious. To save our fingers from typing ``ctx->`` a thousand times, we install the wiring.

.. code-block:: c

    #define error_func ctx->error_func
    #define curr_func ctx->curr_func
    // ...

These macros effectively "mount" the context's internal organs directly into the local namespace. It makes the code readable, though it implies that ``ctx`` must always be present in the local scope.

5. The Fail-Safes
-----------------

No adventure is without peril.

.. code-block:: c

    static void MIR_NO_RETURN default_error (MIR_error_type_t error_type, const char *format, ...)

This is the doom-sayer. If an allocation fails or the syntax is garbled, this function is invoked. It screams the error to ``stderr`` and pulls the ultimate plug: ``exit(1)``. It is the hard stop at the end of a failed expedition.

6. The Sentinels (Reserved Names)
---------------------------------

Finally, before we dive deeper, we encounter the gatekeepers.

.. code-block:: c

    #define HARD_REG_NAME_PREFIX "hr"
    #define TEMP_REG_NAME_PREFIX "t"

MIR has sacred names that mere mortals (users) are forbidden to use.

*   **``_MIR_reserved_name_p``**: This function stands guard at the gates of symbol creation. It checks if you are trying to name a variable something like ``hr5`` (Hardware Register) or ``t10`` (Temporary). If you try to usurp these system-level names, this sentinel will flag you down.

We have established our base of operations. The context is defined, the memory handlers are ready, and the guards are posted. Now, we proceed to the logic.

7. The Book of Spells (``insn_descs``)
-------------------------------------

We have now reached the heart of the lexicon. The ``insn_descs`` table is the Rosetta Stone that translates cryptic opcode enums into human-readable strings and defines the laws of their usage.

.. code-block:: c

    struct insn_desc {
      MIR_insn_code_t code;
      const char *name;
      unsigned char op_modes[5];
    };

    #define OUT_FLAG (1 << 7)

    static const struct insn_desc insn_descs[] = {
      {MIR_MOV, "mov", {MIR_OP_INT | OUT_FLAG, MIR_OP_INT, MIR_OP_BOUND}},
      {MIR_FMOV, "fmov", {MIR_OP_FLOAT | OUT_FLAG, MIR_OP_FLOAT, MIR_OP_BOUND}},
      {MIR_ADD, "add", {MIR_OP_INT | OUT_FLAG, MIR_OP_INT, MIR_OP_INT, MIR_OP_BOUND}},
      {MIR_ADDS, "adds", {MIR_OP_INT | OUT_FLAG, MIR_OP_INT, MIR_OP_INT, MIR_OP_BOUND}},
      {MIR_FEQ, "feq", {MIR_OP_INT | OUT_FLAG, MIR_OP_FLOAT, MIR_OP_FLOAT, MIR_OP_BOUND}},
      {MIR_JMP, "jmp", {MIR_OP_LABEL, MIR_OP_BOUND}},
      {MIR_CALL, "call", {MIR_OP_BOUND}},
      // ... and hundreds more
    };

Every instruction in the MIR arsenal is cataloged here.

*   **``code``**: The internal identifier (e.g., ``MIR_MOV``).
*   **``name``**: The chant used by programmers (e.g., ``"mov"``).
*   **``op_modes``**: The grammar of the spell. It dictates what ingredients are required.
    *   ``MIR_OP_INT | OUT_FLAG``: The first operand *must* be an integer, and it is an **output** (the result is written here).
    *   ``MIR_OP_INT``: The second operand *must* be an integer, and it is an **input** (read-only).
    *   ``MIR_OP_BOUND``: The terminator. This instruction takes exactly two operands.

Scanning this list reveals the capabilities of our machine:

*   **Arithmetic:** ``add``, ``sub``, ``mul``, ``div``.
*   **Logic:** ``and``, ``or``, ``xor``, shifts (``lsh``, ``rsh``).
*   **Comparison:** ``eq`` (equal), ``lt`` (less than), ``ne`` (not equal).
*   **Type Conversion:** ``i2f`` (int to float), ``d2i`` (double to int).
*   **Control Flow:** ``jmp`` (jump), ``bt`` (branch if true), ``call``, ``ret``.

This table is the single source of truth. If an instruction isn't here, it doesn't exist. If you try to feed a float to an ``add`` instruction (which expects ``MIR_OP_INT``), the validators using this table will strike you down.

It is a dense, repetitive list, but therein lies its power: consistency. It is the Periodic Table of Elements for the MIR universe.

8. Calibration and Orientation
------------------------------

Having established the laws of the universe (``insn_descs``), we now encounter the machinery that calibrates the engine and determines our orientation within the physical hardware.

8.1 The Equipment Check (``check_and_prepare_insn_descs``)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Before the compiler processes a single instruction, it must perform a self-diagnostic. This function is the **Supply Master** counting the crates.

*   **The Sanity Check:** ``mir_assert (insn_descs[i].code == i)`` ensures the "Book of Spells" is not corrupted. It verifies that the ``insn_descs`` array is perfectly indexed by the opcode enum. If an instruction is out of order, the compiler halts immediately.
*   **Pre-computation:** It iterates through the operand modes of every instruction to count them (``j``). Instead of counting operands every time an instruction is issued (which would be slow), it caches this "arity" in the ``insn_nops`` array. This is a classic "do it once, use it forever" optimization.

8.2 The Great Simplification (``type2mode``)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Here we see the compiler's internal alchemy. While the user cares about specific types like ``int8``, ``uint16``, or ``int64``, the machinery often only cares about the **Storage Class**.

This function transmutes granular types into elemental ``MIR_op_mode_t`` categories:

*   All integers (signed, unsigned, pointers) :math:`\rightarrow` ``MIR_OP_INT``.
*   Floating points retain their distinct nature (``FLOAT``, ``DOUBLE``, ``LDOUBLE``).

This tells us that internally, MIR treats all integer types as congruent 64-bit containers, handling sign-extension or truncation only when necessary.

8.3 The Compass (``_MIR_addr_offset``)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

This seemingly small function reveals a crucial capability: **Endianness Awareness**.

*   ``int v = 1; if (... *(char *) &v != 0)``: This is a runtime check to see if the machine is **Little Endian** (like x86 or ARM64) or **Big Endian** (like s390x or legacy PowerPC).
*   **The Big Endian Problem:** In a 64-bit register on a Big Endian machine, an 8-bit value lives at the *highest* address byte (offset 7), whereas on Little Endian, it lives at offset 0.
*   **The Adjustment:** If the machine is Big Endian, this function returns the necessary offset (7 for byte, 6 for short, 4 for int) to point to the correct data inside the 64-bit slot.

9. The Great Archives (String Interning)
----------------------------------------

Yes, this absolutely deserves its own section. We have stepped away from the grinding gears of the instruction machinery and entered the cool, quiet halls of the **String Interning System**.

9.1 The Law of Uniqueness
~~~~~~~~~~~~~~~~~~~~~~~~~

This snippet implements a mechanism known in computer science cartography as **String Interning**.

*   **The Problem:** A source program might use the identifier ``i`` or the function name ``printf`` thousands of times. Allocating memory for these characters thousands of times is wasteful; comparing them character-by-character is slow.
*   **The Solution (``string_store``):** When you bring a string to the Archives, the Scribe (``string_find``) checks the great index (the **Hash Table** ``string_tab``).
    *   If the text is already recorded, the Scribe hands you back a simple **Number** (``size_t num``). This is the catalog ID.
    *   If the text is new, the Scribe makes a **local copy** (``MIR_malloc`` + ``memcpy``), assigns it a new ID, and files it away in the Vault (the **Vector** ``strings``).

9.2 The Dual-Structure Strategy
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Notice the ``string_ctx``:

.. code-block:: c

    struct string_ctx {
      VARR (string_t) * strings;  /* The Vault: Indexed access (ID -> String) */
      HTAB (string_t) * string_tab; /* The Index: Lookup access (String -> ID) */
    };

This is a classic optimization pattern. We need two ways to access data:

1.  **Fast Insertion/Deduplication:** The Hash Table allows :math:`O(1)` checks to see if a string exists.
2.  **Fast Retrieval:** The Vector allows :math:`O(1)` access when we later need to print the name associated with ID #42.

9.3 The Sacred Zero (``string_init``)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Look closely at the initialization:

.. code-block:: c

    VARR_PUSH (string_t, *strs, string); /* don't use 0th string */

The Archives define a "Zero Entry." The string at index 0 is empty/null.

**Lesson:** In MIR, a string ID of ``0`` effectively means ``NULL``. Valid string IDs start at 1. This prevents the "null pointer problem" by ensuring that 0 is never a valid reference to a real identifier.

10. The Department of Aliases (Parallel Archives)
------------------------------------------------

Just as we leave the Great Archives of Strings, we stumble upon a smaller, adjacent building. It looks remarkably similar—the architecture, the filing systems, the scribes—everything mirrors the Archives we just visited. This is the **Department of Aliases**.

10.1 The Doppelgänger Mechanism
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

The code here reveals a fascinating efficiency in the MIR design philosophy: **Reuse**.

*   **The Observation:** ``struct alias_ctx`` is structurally identical to ``struct string_ctx``.
*   **The Action:** The function ``MIR_alias`` explicitly calls ``string_store``—the very same function used by the Great Archives.
*   **The Lesson:** Why reinvent the wheel? The mechanism for ensuring uniqueness (interning) is robust. The compiler simply creates a *separate instance* of this mechanism specifically for Aliases.

10.2 Why a Separate Vault?
~~~~~~~~~~~~~~~~~~~~~~~~~~

If they are both just storing strings, why not keep them in the main Archive?

This is a matter of **Type Safety and Semantic Isolation**.

*   In MIR, an ``alias`` is a specific concept used for memory optimization (telling the compiler which pointers might overlap and which definitely do not).
*   By keeping them in a separate table, an Alias ID of ``42`` is mathematically distinct from a String ID of ``42``.
*   This prevents a catastrophic mix-up where a variable named "x" (String ID 10) is accidentally treated as Alias Set 10, potentially causing the optimizer to generate invalid code.

10.3 The Retrieval Guard (``MIR_alias_name``)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

The retrieval function adds a layer of protection:

.. code-block:: c

    if (alias >= VARR_LENGTH (string_t, aliases))
        MIR_get_error_func (ctx) ...

The Department of Aliases checks your credentials. If you ask for the name of Alias #500, but the registry only goes up to #50, it triggers the alarm.

11. The Registry of Names (Function-Local Symbol Tables)
------------------------------------------------------

We have navigated the global archives and aliases, but now we descend into the specific quarters of a function. Each function is its own fiefdom, and ``func_regs_t`` is the local census bureau.

11.1 The Trinity of Lookup Tables
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Deep within ``struct func_regs``, we find three Hash Tables guarding the identity of registers:

1.  **``name2rdn_tab``**: Maps string names to indices in the ``reg_descs`` vector.
2.  **``reg2rdn_tab``**: Maps numeric register IDs back to their descriptors.
3.  **``hrn2rdn_tab``**: Tracks variables tied to specific hardware registers (e.g., ``rax``).

11.2 The Binding Ritual (``create_func_reg``)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

The function ``create_func_reg`` is the ceremony where a new variable is born.

*   **The Name Check:** Enforces that user variables don't usurp reserved machine names like ``hr5`` or ``t10``.
*   **The Hard Register Pact:** Validates that requested hardware registers exist, are not reserved (like the stack pointer), and match the data type.
*   **The Global Bitmap:** Marks tied hardware registers in the module's global bitmap so the compiler knows they are occupied.

11.3 The Zero Register
~~~~~~~~~~~~~~~~~~~~~~

By pushing a dummy entry at index 0, MIR ensures that register 0 remains invalid/reserved, aligning valid IDs with positive indices.

12. The Utility Belt and the Cartographer’s Tools
-------------------------------------------------

12.1 The Identifier (``MIR_item_name``)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

A universal function to extract names from polymorphic ``MIR_item`` objects (functions, data, prototypes).

12.2 The Safety Toggles (``func_redef_permission``)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Allows or disallows the redefinition of existing functions, a feature crucial for interactive environments like REPLs.

12.3 The Global Index (``module_item_tab``)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

A hash table providing :math:`O(1)` lookup for any item in any module by name.

12.4 The Standard Issue Gear (``#include *.c``)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

MIR includes default implementations for memory allocation (``mir-alloc-default.c``) and code allocation (``mir-code-alloc-default.c``) to be fully self-contained if custom allocators aren't provided.

13. The Alien Terrain: Android
------------------------------

Android lacks a standard console. MIR bridges this by using the **Android NDK Logging API** (``__android_log_print``) when ``MIR_ANDROID_TRACE`` is enabled, routing output to ``logcat``.

14. The Big Bang: ``_MIR_init``
-------------------------------

The moment of creation where the ``MIR_context`` is allocated and every subsystem (strings, modules, environment, code generation) is brought online.

15. The Surgical Removal: ``MIR_remove_insn``
---------------------------------------------

Safely extracts and frees an instruction from a function's doubly-linked list.

16. The Great Purge: ``remove_item`` and ``remove_module``
----------------------------------------------------------

Recursively destroys items and modules, ensuring no memory leaks when part of the environment is discarded.

17. The Apocalypse: ``MIR_finish``
----------------------------------

Systematically dismantles the entire context, closing the archives, factories, and universes before finally freeing the context handle itself.

**Summary:** We have witnessed the birth and death of the MIR runtime environment. We saw how it defensively allocates resources and how it ruthlessly cleans them up. Now, we are ready to look at the **Creation** of new life: building modules and functions.

18. The Colony Ship (``MIR_new_module``)
----------------------------------------

Launches a new module container. MIR enforces a "one ship at a time" rule; you must finish the current module before starting another.

19. The Rosetta Stone (``type_str`` & ``mode_str``)
---------------------------------------------------

Translators that convert internal numeric enums into human-readable strings for diagnostics and error messages.

20. The Gatekeeper (``add_item``)
---------------------------------

The bureaucrat ensuring that names within a module remain unique, resolving links between forward declarations and their eventual definitions.

21. The Forge (``create_item``)
-------------------------------

The raw factory that allocates and initializes the base ``MIR_item`` structure.

22. The Diplomatic Corps (Imports, Exports, Forwards)
----------------------------------------------------

Manages external symbol visibility and promises of future definitions.

23. Claiming the Void: ``MIR_new_bss``
--------------------------------------

Stakes a claim for zero-initialized global memory.

24. The Reality Distortion Field: ``canon_type``
------------------------------------------------

Ensures cross-platform binary compatibility by mapping types like ``long double`` to their host-appropriate representation (e.g., mapping to ``double`` on Windows).

25. The Surveyor: ``_MIR_type_size``
------------------------------------

Determines the physical byte size of MIR types on the host architecture.

26. The Vault of Knowledge: Data and References
-----------------------------------------------

26.1 The Scribe: ``MIR_new_data`` & ``MIR_new_string_data``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Etches raw bytes or string literals into the module's data segment.

26.2 The Navigator: ``MIR_new_ref_data``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Creates data items that store the address of another item (resolved at link-time).

26.3 The Local Guide: ``MIR_new_lref_data``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Handles relative label addresses, crucial for jump tables and position-independent code.

26.4 The Oracle: ``MIR_new_expr_data``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Initializes data by executing a MIR function during the linking phase.

27. The Protocols of Engagement: ``MIR_proto``
----------------------------------------------

Defines the function signatures and ABI contracts for calls.

28. The Birth of a Function (``MIR_new_func``)
----------------------------------------------

Starts the construction of an executable unit, initializing its instruction lists and local register table.

29. The Final Inspection (``MIR_finish_func``)
----------------------------------------------

A multi-pass validator that ensures the instruction stream is coherent and matches the function's prototype.

30. The Global Stage (``setup_global``)
---------------------------------------

Registers host pointers (like ``printf``) in the special environment module.

31. The Void (``undefined_interface``)
--------------------------------------

The runtime trap that catches calls to unlinked or unresolved functions.

33. The Loading Dock (``load_bss_data_section``)
------------------------------------------------

Physically allocates and populates the data/BSS segments in RAM.

34. The Cartographer's Survey (``link_module_lrefs``)
-----------------------------------------------------

Resolves label-to-function ownership for local references.

35. The Grand Opening (``MIR_load_module``)
-------------------------------------------

Loads a module, sets up thunks, and registers exports in the global scope.

36. The Diplomatic Landing (``MIR_load_external``)
--------------------------------------------------

Bridges MIR to the host C runtime, handling special cases like ``setjmp``.

37. The Grand Junction (``MIR_link``)
-------------------------------------

The nexus point where imports are resolved, inlining is performed, and final function interfaces (interpreter or JIT) are set.

38. The Simplifier's Toolbox (``simplify_func``)
------------------------------------------------

Strips away syntactic sugar and lowers high-level MIR constructs into a canonical form suitable for the generator.

39. The Unnamed Legion: Temporary Registers
-------------------------------------------

Manages the creation of unique virtual registers for internal use by optimizations.

40. The Census Bureau: Register Lookup
--------------------------------------

High-speed lookup functions for register IDs, names, and types.

41. The Instruction Forge: ``MIR_new_insn`` & Friends
-----------------------------------------------------

Validates and constructs the individual instructions that make up a function body.

42. Copying Reality: ``MIR_copy_insn``
--------------------------------------

Efficiently clones instructions for optimizations like loop unrolling.

43. The Label Maker: ``MIR_new_label``
--------------------------------------

Creates uniquely identified marker instructions for branch targets.

44. The Armory: Operand Construction
------------------------------------

Provides constructors for all MIR operand types (registers, immediates, memory addresses, strings).

45. The Identifier: ``MIR_op_eq_p``
-----------------------------------

Strictly compares two operands for structural and semantic equality.

46. The Fingerprint: ``MIR_op_hash_step``
-----------------------------------------

Generates hash values for operands, enabling their use in optimization tables.

47. The Time Machine: ``_MIR_duplicate_func_insns`` and ``_MIR_restore_func_insns``
-------------------------------------------------------------------------------------

Allows saving and restoring function bodies to support trial-and-error optimization passes.

48. The Identity Shifter: ``MIR_change_module_ctx``
---------------------------------------------------

A complex operation to teleport a module between contexts, re-interning all strings and symbols. Note: **This operation is not thread-safe.**

49. The Printing Press: ``MIR_output``
--------------------------------------

Renders the internal AST back into human-readable assembly for debugging.

50. The Binary Scribes: ``MIR_write`` and ``MIR_read``
------------------------------------------------------

Handles high-performance serialization of MIR modules into a tagged binary format with structural compression.

51. The Output Machinery Revisited: ``MIR_output`` (and friends)
----------------------------------------------------------------

Detailed Scribes for operands, instructions, items, and modules that reconstruct the source code from the internal representation.

52. The Simplification Pass (``simplify_func``)
-----------------------------------------------

Beat the wild user input into a canonical form, handling memory-to-memory moves and complex operands to simplify the backend generator.

53. The Scanner and Parser (``scan_token``, ``MIR_scan_string``)
----------------------------------------------------------------

The combined Lexer and Parser that reads textual MIR, using ``setjmp``/``longjmp`` for error recovery.

54. The End of the Journey: ``MIR.c`` Complete
----------------------------------------------

The Core MIR implementation provides a complete universe for representing, manipulating, and linking code.

**What remains?**
The machine code itself. We have built the *idea* of the program. Now we must translate that idea into the specific dialect of the processor. That happens in **``mir-gen.c``**.

55. The Optimization Forge: Value Numbering
-------------------------------------------

Implements Local Value Numbering (LVN) to eliminate redundant calculations (CSE) within basic blocks by tracking computations in a hash table.

56. The Simplifier's Core: ``simplify_op``
------------------------------------------

Standardizes operands, decomposing complex memory addressing and moving string literals into the data segment.

57. The Return Fixer: ``make_one_ret``
-------------------------------------

Canonicalizes functions to have a Single Point of Exit, simplifying subsequent liveness analysis and register allocation.

58. The Label Sweeper: ``remove_unused_and_enumerate_labels``
-------------------------------------------------------------

Prunes unreferenced labels and renumbers the survivors to maintain a dense and clean label space.

59. The Name Binder: ``_MIR_uniq_string``
----------------------------------------

We start with a small but crucial incantation.

In the realm of MIR, names are power, but only if they are **unique**. If you have two strings "foo" at different memory addresses, the compiler treats them as strangers. This function is the **Identity Unifier**. It takes a raw C string and returns the *One True Pointer* to that string within the context's archives. It ensures that when we speak of "memcpy", we are all talking about the exact same entity.

60. The Elder Scrolls: ``_MIR_builtin_proto``
---------------------------------------------

Now we encounter the mechanism for defining **Builtins**. These are the ancient laws of physics for the target architecture—functions that *must* exist for the machinery to work (like ``__builtin_memcpy`` or ``va_start``).

*   **The Infiltration:** Notice the variable ``saved_module``. This function pulls a heist. It saves your current location, teleports into the target ``module``, plants the prototype, and then teleports back (``curr_module = saved_module``). It doesn't matter where you are when you call this; the prototype appears exactly where it needs to be.
*   **The Duplicate Check:** The system is paranoid. If you try to register a builtin that already exists, it interrogates the existing one. "Do you have the same return type? The same arguments?" If everything matches, it smiles and returns the existing one. If you try to change the laws of physics (e.g., claiming ``memcpy`` takes a float), it throws a ``MIR_repeated_decl_error``.
*   **The VIP Treatment:** ``DLIST_PREPEND``. Builtin prototypes don't wait in line. They are shoved to the very front of the module's item list. They are the aristocracy of the IR.

61. The Hardline: ``_MIR_builtin_func``
----------------------------------------

If ``_MIR_builtin_proto`` describes the *shape* of the magic, ``_MIR_builtin_func`` provides the **Mana**.

This function bridges the gap between the abstract MIR universe and the concrete "Native Soil" of the host CPU. It associates a string name (e.g., ``"mir.blk_mov"``) with a raw C function pointer (``void *addr``).

*   **The Global Ether (``environment_module``):**
    *   First, it checks the **Environment**—the shared global namespace.
    *   If the function isn't there, it creates an ``import`` item in the Environment. But this is a special import: it comes pre-loaded with an address (``ref_item->addr = addr``). It is an import that needs no finding; it *is* the definition.
*   **The Local Link:**
    *   Then, it turns to the specific ``module`` requested.
    *   It creates a local ``import`` item there.
    *   **The Wiring:** It connects the local import (``item``) to the global environment import (``ref_def = ref_item``).
    *   It copies the raw address (``item->addr = ref_item->addr``).

**The Result:** When the generator sees a call to this function in this module, it doesn't need to ask the dynamic linker for help. It already has the hard-coded memory address of the C function to jump to. It is a hardline telephone directly to the hardware.

63. The Machine Code Foundry
----------------------------

We have crossed the threshold. This isn't abstract syntax anymore; this is **raw, executable memory**. The code here deals with the physical reality of the operating system's memory manager.

63.1 ``_MIR_set_code``: The W^X Violation
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Modern Operating Systems enforce a strict policy known as **W^X (Write XOR Execute)**. Memory can be writable or executable, but never both simultaneously (to prevent security exploits).
But a JIT compiler *must* write code and then execute it.

*   **The Dance:** ``_MIR_set_code`` performs a dangerous dance with the OS kernel.
    1.  ``MIR_mem_protect(..., PROT_WRITE_EXEC)``: "Hey Kernel, look away for a second. I need to write here."
    2.  ``memcpy``: The bytes are blasted into place.
    3.  ``MIR_mem_protect(..., PROT_READ_EXEC)``: "Okay, I'm done. You can lock it down again."

63.2 The Code Holder: ``get_last_code_holder``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

You can't just ``malloc`` executable memory. You need ``mmap`` (on Unix) or ``VirtualAlloc`` (on Windows), and these only work in page-sized chunks (usually 4KB). Requesting a fresh page for every 10-byte function would be wasteful.

*   **The Strategy:** MIR allocates large chunks (``code_holder_t``).
*   **The Allocation:** It fills them up linearly (``ch_ptr->free += code_len``).
*   **The Expansion:** When a chunk is full, it allocates a new page-aligned block. This is a custom memory allocator specifically for machine code.

63.3 The Cache Flush: ``_MIR_flush_code_cache``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

CPUs are fast because they cache instructions. But when we generate new code, it sits in the *Data Cache*, not the *Instruction Cache*. If we try to jump to it immediately, the CPU might execute stale garbage.

*   **The Instruction:** ``__builtin___clear_cache`` (or OS specific equivalent) forces the CPU to invalidate its instruction cache for the given range. It screams at the processor: "FORGET WHAT YOU KNOW! RELOAD FROM RAM!"

64. The Code Patcher: ``_MIR_update_code``
------------------------------------------

JIT compilation is rarely a "write once" affair. We often need to patch code after it's generated (e.g., resolving a forward jump label or linking a function call).

This function allows surgical strikes on existing code. It takes a base address and a list of ``relocs`` (offset + value pairs) and patches the machine code in place, handling the memory protection dance automatically.

**Adventure Status Check**

We have completed our survey of the **Core (``mir.c``)**. We have seen the universe created, functions defined, data laid out, and finally, the memory management for the executable code itself.

The foundation is rock solid.

But a foundation is not a house. We have the *tools* to generate code, but we haven't seen *how* the decisions are made. How does ``ADD a, b, c`` actually become ``0x48 0x01 0xD8``? How do registers get assigned?

The next leg of our journey takes us into the brain of the operation: **``mir-gen.c``**. This is where the Register Allocator lives. This is where the Control Flow Graph is built. This is where the abstract becomes concrete.

Are you ready to enter the Generator?

65. The Secret Language: Binary Serialization

-------------------------------------------



We have stumbled upon the **Cipher Room**.



Until now, we have been dealing with verbose, chatty text formats ("module", "func", "add"). But machines, like spies, prefer brevity. They whisper in **Binary**.



This section defines a custom binary protocol for freezing a MIR module into a compact block of bytes (Serialization) and thawing it back out (Deserialization).



65.1 The Tag System (``bin_tag_t``)

~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~



This enum is the decoder ring. Every piece of data in the binary stream is prefixed by one of these tags.



*   **The Tiny Integers:** ``TAG_U0`` (embedded directly in the tag byte) through ``TAG_U8``. Why waste 8 bytes storing the number ``5``? The tag tells the reader exactly how many bytes follow. This is variable-length integer encoding, the bread and butter of compression.

*   **The Types:** ``TAG_TI8``, ``TAG_TF``, ``TAG_TP``. These are the elemental symbols of the MIR universe.

*   **The References:** ``TAG_STR1``...``TAG_STR4``. Instead of writing the string "printf" every time, we write it once in a header table, and then refer to it by index. It's like using pronouns instead of full names.



65.2 The Compressors (``put_uint``, ``put_int``)

~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~



These functions are the hydraulic presses of the operation.



*   ``uint_length``: Measures the "true" size of a number. ``0xFF`` fits in 1 byte. ``0xFFFF`` needs 2.

*   ``put_uint``: Squeezes the number into that minimum space.

*   **The Joke:** It's like packing for a vacation. You *could* bring a giant suitcase for one pair of socks (8 bytes for the number 1), but ``put_uint`` insists you use a coin purse instead.



65.3 The Floating Point Trick

~~~~~~~~~~~~~~~~~~~~~~~~~~~~~



.. code-block:: c



    union { uint32_t u; float f; } u;



Ah, the classic **Type Punning Union**. To write a float as bytes, we pretend it's an integer. The CPU doesn't care; bits are bits. We dump the raw IEEE 754 bit pattern directly into the stream. It's crude, it's effective, and it avoids any nasty conversion errors.



65.4 The Context: ``io_ctx``

~~~~~~~~~~~~~~~~~~~~~~~~~~~~



This structure holds the state of the transcription.



*   **``bin_strings``**: The dictionary of all strings used in the module.

*   **``io_reduce_data``**: A pointer to the **Compression Engine** (``mir-reduce.h``). Yes, MIR has a built-in compressor (likely an LZ4 variant or similar dictionary-based scheme) layered *underneath* this binary format. It's like wrapping a riddle in a mystery inside an enigma.



**The Adventure So Far:** We have learned how to freeze-dry code. We can take a living, breathing function, dehydrate it into a stream of dense bytes, and store it for later re-animation.



66. The Decoding Chamber: Binary Resurrection
-------------------------------------------

We have now traveled full circle. We have seen how to write the sacred binary scrolls; now we must learn to read them. ``MIR_read`` is the resurrection ritual. It takes the frozen bytes from disk and breathes life back into them, reconstructing the vibrant object graph of modules, functions, and instructions.

66.1 Originality: The Tag-Driven Recursive Descent
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Most binary formats are "struct dumps" or "fixed schemas." MIR's binary format is different. It is a **Stream of Consciousness**.

*   **The Originality:** The format is essentially a bytecode for a state machine that reconstructs the AST. It doesn't just say "Here is a function"; it says "Start Function. Name it 'foo'. Add argument. Add argument. End Function." It mirrors the ``MIR_scan_string`` text parser's logic almost perfectly, but consumes binary tokens instead of ASCII characters.
*   **The Convention:** It follows a **Tag-Length-Value (TLV)** philosophy, but highly optimized.
    *   **Tiny Integers (``TAG_U0``):** Integers < 128 are embedded directly into the top bit of the tag byte. This is a classic compression trick (used in UTF-8, LEB128, Protocol Buffers), but MIR's implementation is bespoke and dead simple.
    *   **Implicit Context:** The reader knows where it is. If it just read a ``TAG_MEM_DISP``, it knows the next token *must* be an integer displacement. It doesn't need self-describing schemas for every field.

66.2 The Hidden Detail: String Interning on the Fly
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

An inexperienced reader might miss ``read_all_strings``.

*   **The Trick:** The binary file starts with a **String Table**. All strings used anywhere in the module are dumped at the start.
*   **The Benefit:** When the rest of the file refers to a function name or a variable, it just uses an integer index (``TAG_NAME1``...``TAG_NAME4``). The reader looks up the interned string in ``bin_strings``. This means string comparison during loading is fast (integer comparison), and memory usage is minimized.

66.3 The Label Puzzle: Forward References
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Labels are the bane of single-pass parsers. You might jump to a label that hasn't been defined yet.

*   **The Solution:** ``to_lab``.
    *   When the reader sees a label ID (``TAG_LAB``), it checks ``func_labels``.
    *   If the label exists, great.
    *   If not, it **pre-creates** the label (``create_label``) and stores it.
    *   Later, when the label definition is encountered in the stream, the pre-created object is used.
    This resolves the "chicken-and-egg" problem of forward jumps without needing a second pass.

66.4 The Safety Net: ``reduce_decode``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Just like the writer had a compressor, the reader has a decompressor (``reduce_decode_get``).

*   **The Detail:** The binary reader doesn't read from the file directly. It pulls bytes from the *decompression engine*, which pulls from the file. The complexity of LZ-style compression is completely hidden from the MIR parsing logic.

67. The End of ``mir.c``
------------------------

We have reached the absolute end of the core file. We have seen the birth, life, death, and resurrection (IO) of MIR code.

*   **The Adventure:** We started with empty structs and ended with a fully serializable, optimizing, dynamic compiler infrastructure.
*   **The Next Chapter:** The ``mir.c`` file is the **Mind**. It understands the *meaning* of the code. But it cannot *execute* it. To run, we need a **Body**. We need machine code.

Our journey must now cross the bridge to **``mir-gen.c``**. That is where the abstract ``MIR_ADD`` becomes a concrete ``0x01 0xC8`` (x86 add). That is where the infinite virtual registers clash for survival in the finite physical register file. That is the realm of the **Backend**.

Gather your things. The abstract world is behind us. The concrete world of assembly awaits.

**End of `mir.c` Deep Dive.**