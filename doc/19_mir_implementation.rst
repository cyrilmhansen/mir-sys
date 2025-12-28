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
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

This seemingly small function reveals a crucial capability: **Endianness Awareness**.

*   ``int v = 1; if (... *(char *) &v != 0)``: This is a runtime check to see if the machine is **Little Endian** (like x86 or ARM64) or **Big Endian** (like s390x or legacy PowerPC).
*   **The Big Endian Problem:** In a 64-bit register on a Big Endian machine, an 8-bit value lives at the *highest* address byte (offset 7), whereas on Little Endian, it lives at offset 0.
*   **The Adjustment:** If the machine is Big Endian, this function returns the necessary offset (7 for byte, 6 for short, 4 for int) to point to the correct data inside the 64-bit slot.

This snippet proves that MIR is designed to be **portable across architectures**, dynamically adapting to the "magnetic north" of the CPU it finds itself running on.

9. The Great Archives (String Interning)
----------------------------------------

Yes, this absolutely deserves its own section. We have stepped away from the grinding gears of the instruction machinery and entered the cool, quiet halls of the **String Interning System**.

In any grand expedition (or compiler), names are power. But names are also heavy. We cannot afford to carry the string ``"counter_variable_name"`` on our backs every time it is referenced. Instead, we visit the Archives.

9.1 The Law of Uniqueness
~~~~~~~~~~~~~~~~~~~~~~~~~

This snippet implements a mechanism known in computer science cartography as **String Interning**.

*   **The Problem:** A source program might use the identifier ``i`` or the function name ``printf`` thousands of times. Allocating memory for these characters thousands of times is wasteful; comparing them character-by-character is slow.
*   **The Solution (``string_store``):** When you bring a string to the Archives, the Scribe (``string_find``) checks the great index (the **Hash Table** ``string_tab``).
    *   If the text is already recorded, the Scribe hands you back a simple **Number** (``size_t num``). This is the catalog ID.
    *   If the text is new, the Scribe makes a **local copy** (``MIR_malloc`` + ``memcpy``), assigns it a new ID, and files it away in the Vault (the **Vector** ``strings``).

**What we learn:** MIR manages its own memory for strings. Once you pass a string to MIR, MIR copies it. You, the traveler, are free to destroy your original copy. MIR's copy lives until ``MIR_finish`` burns the Archives down.

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

By reducing complex strings to simple integers, the rest of the compiler can compare identifiers just by comparing numbers. "Is ``var_a`` the same as ``var_b``?" becomes ``if (102 == 450)``, which is instantaneous.

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

The retrieval function adds a layer of protection that the generic string retrieval did not explicitly show in the previous snippet:

.. code-block:: c

    if (alias >= VARR_LENGTH (string_t, aliases))
        MIR_get_error_func (ctx) ...

The Department of Aliases checks your credentials. If you ask for the name of Alias #500, but the registry only goes up to #50, it triggers the alarm (``MIR_alloc_error``, though semantically it's an out-of-bounds error).

**Summary of the Expedition so far:** We have seen how the world is built (Contexts), how instructions are defined (Opcodes), and how data is deduplicated (Strings and Aliases). We are now ready to look at how these elements are bound together into functions.

11. The Registry of Names (Function-Local Symbol Tables)
------------------------------------------------------

We have navigated the global archives and aliases, but now we descend into the specific quarters of a function. Each function is its own fiefdom, and ``func_regs_t`` is the local census bureau.

Here, we see how MIR manages the chaotic world of user-defined variable names within a function scope.

11.1 The Trinity of Lookup Tables
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Deep within ``struct func_regs``, we find three Hash Tables guarding the identity of registers:

1.  **``name2rdn_tab`` (Name to Register Descriptor Number):**
    *   **The Query:** "I have a variable name string (e.g., ``counter``). Which register ID does it belong to?"
    *   **The Mechanism:** This maps the string name to an index in the ``reg_descs`` vector. It's the primary phonebook for the function.

2.  **``reg2rdn_tab`` (Register ID to Register Descriptor Number):**
    *   **The Query:** "I have a raw register ID (e.g., ``1042``). What are its properties?"
    *   **The Mechanism:** This maps the numeric ID back to the descriptor index. This is crucial because MIR allows register IDs to be sparse or non-sequential in some contexts.

3.  **``hrn2rdn_tab`` (Hard Register Name to Register Descriptor Number):**
    *   **The Query:** "Is any variable currently tied to the hardware register ``rax``?"
    *   **The Mechanism:** This specialized table tracks "Tied Registers"—variables that the user has explicitly demanded live in a specific CPU register (e.g., for ABI compliance or specific optimizations).

11.2 The Binding Ritual (``create_func_reg``)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

The function ``create_func_reg`` is the ceremony where a new variable is born.

*   **The Name Check:** First, the ``_MIR_reserved_name_p`` sentinel checks the name. You cannot name your variable ``hr5`` or ``t10``. Those names belong to the machine gods.
*   **The Hard Register Pact:** If you ask for a specific hardware register (``hard_reg_name != NULL``), the system performs a rigorous background check:
    *   Does the hardware register actually exist on this architecture? (``_MIR_get_hard_reg``)
    *   Does the requested type match the register's capabilities? (No putting a ``float`` in a general-purpose integer register).
    *   Is the register reserved? (You cannot claim the Stack Pointer ``sp`` or Frame Pointer ``fp`` for your own vanity).
*   **The Global Bitmap:** If you successfully bind a variable to a hard register, that register is marked in the module's global bitmap (``func_module->data``). This signals to the rest of the compiler: "This hardware register is occupied. Do not touch."

11.3 The Zero Register
~~~~~~~~~~~~~~~~~~~~~~

In ``func_regs_init``, we see a curious offering:

.. code-block:: c

    reg_desc_t rd = {MIR_T_I64, 0, NULL, NULL};
    VARR_PUSH (reg_desc_t, func_regs->reg_descs, rd); /* for 0 reg */

The zeroth entry in the descriptor array is a dummy. Register ID 0 is invalid/reserved in MIR. By pushing a dummy entry first, the system ensures that valid register IDs (starting at 1) align perfectly with indices 1+ in the vector, or at least reserves the slot so 0 is never handed out.

**Summary:** We have explored how names are turned into numbers, and how numbers are bound to silicon. The machinery is robust, enforcing type safety and hardware constraints before a single instruction is even generated. Now, we must look at how these functions are brought to life.

12. The Utility Belt and the Cartographer’s Tools
-------------------------------------------------

We have reached the final staging area before the heavy machinery of the compiler logic begins. This section is a collection of essential tools, identification tags, and initialization protocols. Think of this as the **Quartermaster’s Station**. Before we march into the jungle of compilation, we must ensure our tools are sharp and our maps are ready.

12.1 The Identifier (``MIR_item_name``)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

This function is the **Universal Identification Spell**.
Because ``MIR_item`` is a shapeshifter (a union that can be a function, a prototype, an import, etc.), we need a safe way to ask, "Who are you?"
This switch statement peels back the wrapper to reveal the name string hidden inside, regardless of whether it is a ``bss`` section, a ``func``, or a ``forward`` declaration.

12.2 The Safety Toggles (``func_redef_permission``)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

*   **``MIR_set_func_redef_permission``**: By default, the MIR universe is immutable—once a function is defined, it is written in stone. This toggle allows the user to play god, granting permission to overwrite existing functions. This is dangerous magic, used primarily for dynamic environments like REPLs where a user might redefine ``int foo()`` five times in a row.

12.3 The Global Index (``module_item_tab``)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Here we see the **Cartographer** at work.

*   **``item_hash`` & ``item_eq``**: These are the coordinates. To find an item quickly, we hash its name mixed with its module pointer.
*   **``item_tab_find`` / ``insert`` / ``remove``**: These manage the global registry. When the compiler needs to know "Where is function ``printf``?", it doesn't search list by list; it queries this hash table for an instant answer.

12.4 The Standard Issue Gear (``#include *.c``)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: c

    #include "mir-alloc-default.c"
    #include "mir-code-alloc-default.c"

This is a rare sight in the wild—**including ``.c`` files directly**.
MIR is designed to be embeddable. If you (the user) do not provide your own custom memory allocators via ``MIR_init2``, MIR unpacks this "Standard Issue Gear." These files contain the default ``malloc``/``free`` wrappers and the OS-specific logic (mmap/VirtualAlloc) to allocate executable memory. It ensures that MIR works out-of-the-box without requiring you to write an OS abstraction layer.

13. The Alien Terrain: Android
------------------------------

You asked: **What is special about Android?**

We find ourselves staring at a strange artifact at the end of the file:

.. code-block:: c

    #if defined(__ANDROID__) && defined(MIR_ANDROID_TRACE)
    #include <android/log.h>
    #define MIR_ANDROID_LOG_TAG "mir-scan"
    #define MIR_ANDROID_LOGI(...) __android_log_print(ANDROID_LOG_INFO, MIR_ANDROID_LOG_TAG, __VA_ARGS__)
    #else
    #define MIR_ANDROID_LOGI(...) ((void) 0)
    #endif

The Problem: The Void of ``stdout``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

On a standard Linux or Windows machine, if the compiler wants to speak, it prints to the console (``stdout`` or ``stderr``).
However, inside the Android operating system, there is no console. If you ``printf`` inside an Android App (APK), your words vanish into the void. They are lost in space.

The Solution: ``logcat``
~~~~~~~~~~~~~~~~~~~~~~~~

Android uses a specialized, circular logging buffer system called ``logcat``. To write to it, you cannot use standard C functions; you must summon the **Android NDK Logging API** (``__android_log_print``).

The Logic Flow:
~~~~~~~~~~~~~~~

1.  **Detection:** The code checks ``defined(__ANDROID__)``. Are we on the green robot's planet?
2.  **Opt-In:** It checks ``defined(MIR_ANDROID_TRACE)``. Do we actually *want* the noise? (Logging on mobile is slow and battery-consuming).
3.  **The Bridge:** If yes, it maps the internal macro ``MIR_ANDROID_LOGI`` to the system call ``__android_log_print``.
4.  **The Silence:** If we are *not* on Android, or tracing is off, the macro is defined as ``((void) 0)``—a "No-Op." It evaporates from the code entirely, costing zero CPU cycles.

**This concludes the setup phase.** The structures are defined, the tools are racked, the memory is ready, and we even have a radio tuned to Android's frequency. We are now ready to begin the compilation process.

We have now landed on the shores of **Functionality**. The engines are humming, the maps are drawn, and the context is alive. It is time to see how this machine breathes.

This section covers the **Genesis** and **Apocalypse** of a MIR Context—its creation and its destruction—and the lifecycle of the artifacts within it.

14. The Big Bang: ``_MIR_init``
-------------------------------

This is the moment of creation. Before this function runs, there is nothing. After it runs, there is a Universe.

*   **The Allocators:** If the user provided no custom memory managers (``NULL``), MIR falls back to its default implementations (``default_alloc``, ``default_code_alloc``).
*   **The Assertions:** A quick sanity check (``mir_assert (MIR_OP_BOUND < OUT_FLAG)``) ensures the fundamental constants of the universe haven't been broken.
*   **The Allocation:** The ``MIR_context`` struct is born from the heap.
*   **The Initialization:** Every subsystem is brought online:
    *   **Strings & Aliases:** ``string_init`` prepares the Archives.
    *   **Instructions:** ``check_and_prepare_insn_descs`` validates the Book of Spells.
    *   **Modules:** ``DLIST_INIT`` prepares the empty list of modules.
    *   **Environment:** The special ``.environment`` module is created to hold built-ins.
    *   **Code Generation:** ``code_init`` prepares the memory for executable machine code.
    *   **Wrappers:** ``wrapper_end_addr`` marks the boundaries for function call wrappers.

This function returns the ``ctx`` pointer—the key to the kingdom.

15. The Surgical Removal: ``MIR_remove_insn``
---------------------------------------------

A compiler is a living thing; it grows and changes. Optimization often means pruning dead branches.

*   **The Assertion:** It checks if you are operating on a valid function item.
*   **The Excision:** ``DLIST_REMOVE`` surgically extracts the instruction from the doubly-linked list.
*   **The Disposal:** ``MIR_free`` returns the memory to the void.

16. The Great Purge: ``remove_item`` and ``remove_module``
----------------------------------------------------------

These functions are the destroyers of worlds.

*   **``remove_item``:** This is a polymorphic destructor. It switches on ``item_type`` to know how to destroy the object.
    *   If it's a **Function**: It destroys instructions, variables, registers, and the struct itself.
    *   If it's **Data**: It frees the memory holding the bytes.
    *   If it's a **Prototype**: It destroys the argument list.
*   **``remove_module``:** It iterates through every item in the module, calling ``remove_item`` on each one, then destroys the module container itself.

17. The Apocalypse: ``MIR_finish``
----------------------------------

This is the end of days. When the user is done with the compiler, they call ``MIR_finish``. It systematically dismantles the entire universe created by ``_MIR_init``.

1.  **Stop the Interpreter:** ``interp_finish``.
2.  **Destroy All Worlds:** ``remove_all_modules`` wipes out every module ever loaded.
3.  **Burn the Archives:** ``string_finish`` destroys the string and alias tables.
4.  **Close the Factories:** ``code_finish`` unmaps executable memory pages.
5.  **Final Sanity Check:** It checks if you left a function or module "open" (unfinished). If so, it scolds you with an error.
6.  **The Final Free:** ``MIR_free(..., ctx)`` deletes the context itself. The universe is gone.

**Summary:** We have witnessed the birth and death of the MIR runtime environment. We saw how it defensively allocates resources and how it ruthlessly cleans them up. Now, we are ready to look at the **Creation** of new life: building modules and functions.

18. The Colony Ship (``MIR_new_module``)
----------------------------------------

Our universe is expanding!

.. code-block:: c

    MIR_module_t MIR_new_module (MIR_context_t ctx, const char *name)

This function launches a new Colony Ship (a Module). It carves out a chunk of memory and gives it a name. But beware! The strict laws of the MIR universe forbid multitasking on this level. ``if (curr_module != NULL)`` triggers a ``MIR_nested_module_error``. You cannot launch a second ship until the first one has landed (``MIR_finish_module``). It's one expedition at a time, soldier.

19. The Rosetta Stone (``type_str`` & ``mode_str``)
---------------------------------------------------

In the dark depths of debugging, raw integers like ``MIR_T_I64`` or ``MIR_OP_MEM`` are meaningless hieroglyphs.

*   **``type_str``**: Converts the cryptic type enums into human language: "i64", "f", "d". It handles the exotic "blk" types dynamically.
*   **``mode_str``**: Does the same for operands: "reg", "mem", "int".

These are the translators that let the machine speak to its creator. Without them, error messages would be riddles wrapped in enigmas.

20. The Gatekeeper (``add_item``)
---------------------------------

This is one of the most complex checkpoints in the system. When you try to add an item (a function, a global, an import) to a module, you must pass the Gatekeeper.

*   **The Conflict Resolution Logic:**
    *   **Duplicate Import?** "You already asked for ``printf``. I'll just give you the old ticket."
    *   **Export vs. Forward?** "Ah, you promised ``foo`` earlier (forward decl), and now you are delivering it (export). I will replace your promise with the real thing."
    *   **Collision?** "You are trying to define ``bar`` again? **Denied.**" (``MIR_repeated_decl_error``).
    *   **Importing Yourself?** "You are importing a symbol you defined right here? That's just silly. **Error.**"

This function is the bureaucrat that keeps the module's symbol table sane. It handles the messy business of linking declarations to definitions so the rest of the compiler can assume a clean world state.

21. The Forge (``create_item``)
-------------------------------

Before the Gatekeeper can judge an item, it must be forged.

.. code-block:: c

    static MIR_item_t create_item (...)

This is the raw factory. It allocates the memory block for a generic ``MIR_item``.

*   It checks if we are actually *inside* a module (``curr_module == NULL`` check). You cannot forge items in the void of space; you need a ship (module).
*   It initializes the item to a blank slate: no address, no data, just a type and a name tag.

**Summary:** We have seen how colonies (modules) are launched, how the language is translated for humans, and the rigorous border control for adding new citizens (items) to the colony. The infrastructure is solid. Now, we must look at how to populate these colonies with specific types of citizens: Exports, Imports, and Data.

22. The Diplomatic Corps (Imports, Exports, Forwards)
----------------------------------------------------

We now turn our attention to the **Foreign Relations** of our module. A module cannot live in isolation; it must trade data and call functions from the outside world.

.. code-block:: c

    static MIR_item_t new_export_import_forward (...)

This helper function is the **Chief Diplomat**. Whether you are declaring an **Export** (offering a service to the world), an **Import** (requesting a service, like ``printf``), or a **Forward** (a promise that a definition is coming soon), the bureaucracy is the same.

1.  **Forge the Item:** Call ``create_item``.
2.  **Stamp the Papers:** Intern the name string so it is unique in the Archives.
3.  **Check the Registry:** Call ``add_item``. If this diplomatic treaty already exists, destroy the new draft and return the existing agreement. This ensures that if you import ``printf`` ten times, the system only tracks one entity.

The public API functions (``MIR_new_export``, etc.) are merely wrappers that dispatch this diplomat with specific orders.

23. Claiming the Void: ``MIR_new_bss``
--------------------------------------

Now we encounter the **Land Grab**.

**BSS** stands for *Block Started by Symbol* (an ancient assemblers term). In the world of C and JITs, this represents **Global Variables initialized to zero**.

.. code-block:: c

    MIR_item_t MIR_new_bss (MIR_context_t ctx, const char *name, size_t len)

When you call this, you aren't providing data; you are staking a claim. You are telling the compiler: *"I need ``len`` bytes of empty land, and I want to call it ``name``."*

*   **Anonymous Lands:** If ``name`` is NULL, the land is claimed but unnamed. It is simply appended to the item list. This is useful for internal storage that doesn't need to be linked against.
*   **Named Lands:** If it has a name, it goes through the ``add_item`` Gatekeeper to ensure no one else has already claimed that territory.

24. The Reality Distortion Field: ``canon_type``
------------------------------------------------

Here lies a subtle but dangerous trap in the landscape of C programming, and MIR navigates it with a clever trick.

.. code-block:: c

    static MIR_type_t canon_type (MIR_type_t type) {
    #if defined(_WIN32) || __SIZEOF_LONG_DOUBLE__ == 8
      if (type == MIR_T_LD) type = MIR_T_D;
    #endif
      return type;
    }

**The ``long double`` Betrayal:**
In the C standard, ``long double`` is a shapeshifter.

*   On Linux x86_64? It's usually 128 bits (or 80 bits padded).
*   On Windows (MSVC)? It is **identical** to ``double`` (64 bits). Microsoft decided long ago that 80-bit math was not worth the alignment headaches.

If MIR generates 128-bit instructions for ``long double`` on Windows, the JIT will crash when interfacing with C code.

**The Fix:** This function is the **Reality Anchor**. It checks the environment at compile-time. If it detects it's running on Windows (or any platform where ``long double`` is just ``double`` in a trench coat), it silently rewrites ``MIR_T_LD`` (Long Double) to ``MIR_T_D`` (Double). The user thinks they are using Long Doubles, but the JIT ensures binary compatibility with the host OS.

25. The Surveyor: ``_MIR_type_size``
------------------------------------

Finally, we have the **Surveyor**.

To build a stack frame or allocate an array, you must know exactly how much space a type occupies. You cannot guess.

.. code-block:: c

    size_t _MIR_type_size (MIR_context_t ctx MIR_UNUSED, MIR_type_t type)

This switch statement is the ultimate source of truth for physical dimensions.

*   It doesn't use magic numbers (like ``4`` or ``8``).
*   It uses ``sizeof(int32_t)``, ``sizeof(double)``, ``sizeof(void *)``.

**Why is this exciting?**
Because it makes MIR **host-aware**. If you compile MIR on a strange architecture where pointers are 128-bit, ``MIR_T_P`` will automatically scale to match ``sizeof(void *)``. This function anchors the abstract MIR types to the concrete reality of the CPU executing the JIT.

**Up Next:** We have defined the items, the memory, and the physics of types. Now, we must define the Content. How do we fill those data segments with actual bytes?

26. The Vault of Knowledge: Data and References
-----------------------------------------------

We've staked our claim on the BSS void, but sometimes, a civilization needs more than empty land. It needs **archives**—pre-written scrolls of data, maps to distant stars, and ancient prophecies.

26.1 The Scribe: ``MIR_new_data`` & ``MIR_new_string_data``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

This is where raw bytes are etched into the memory of our module.

*   **The Check:** First, a quick glance at ``wrong_type_p``. You can't store complex block types as raw data here; only primitives.
*   **The Allocation:** Memory is allocated for the header *plus* the payload (``el_len * nel``). This is a classic C optimization called the **"struct hack"** or flexible array member pattern, keeping the metadata and the data contiguous in cache.
*   **The Copy:** ``memcpy`` seals the deal. The user's data is duplicated into the module's vault.
*   **The Convenience:** ``MIR_new_string_data`` is just a friendly wrapper that treats a string as an array of ``MIR_T_U8`` (unsigned bytes).

26.2 The Navigator: ``MIR_new_ref_data``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Sometimes, data isn't a number; it's a **location**.
Imagine you want a global variable ``int *p = &foo;``. You can't know the address of ``foo`` when you write the code—it changes every time you run the program!

*   **The Solution:** This item type stores a *reference* (``ref_item``) and a *displacement* (``disp``).
*   **The Magic:** The JIT, acting as the Linker, will later resolve this. It will find where ``foo`` lives in memory, add the displacement, and write that final address into this data slot.

26.3 The Local Guide: ``MIR_new_lref_data``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Similar to the Navigator, but for **Labels**. This is for advanced maneuvers like "computed gotos" (jump tables).

*   **The Trick:** It supports ``label - label2 + disp``. This allows for relative addressing calculations (like PC-relative jumps), critical for position-independent code.

26.4 The Oracle: ``MIR_new_expr_data``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

This is the most powerful (and dangerous) construct.

*   **The Concept:** Initialize data not with a constant, but with the **result of a function execution**.
*   **The Usage:** Imagine ``int x = complex_math();``. The JIT will actually *execute* ``complex_math`` during the linking phase and store the result in ``x``.
*   **The Safeguard:** The code asserts ``expr_item->u.func->vararg_p...``. The oracle function must be simple: no arguments, one return value. We don't want side effects blowing up the universe before the program even starts.

27. The Protocols of Engagement: ``MIR_proto``
----------------------------------------------

Before we can execute functions, we must agree on the rules of engagement. This is the **ABI** (Application Binary Interface) contract.

27.1 The Blueprints (``create_proto``)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

This function drafts the treaty.

*   It allocates space for return types and arguments.
*   It copies names into the context's string repository (``get_ctx_str``), ensuring they persist even if the caller frees their strings.

27.2 The Interface Builders (``MIR_new_proto_arr``, etc.)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

These are the public-facing architects.

*   **Variadic Power:** Notice the split between ``_arr`` functions (taking arrays) and the ``...`` functions (taking C variadic arguments). This dual interface makes MIR friendly to both C compilers (which have arrays of types) and humans writing JITs by hand.
*   **The MSVC Quirk:**

    .. code-block:: c

        #if defined(_MSC_VER)
        #define alloca _alloca
        #endif

    Even in the depths of a JIT, we must bow to the idiosyncrasies of the Microsoft Compiler. ``alloca`` (allocating on the stack) is standard on Unix, but an underscore-prefixed hermit on Windows.

**Summary:** We have established how to store static data, how to create dynamic references that resolve at runtime, and how to define the "shape" of functions via prototypes. The stage is set. The actors (functions) have their scripts (instructions) and their props (data). Now, we must build the actors themselves.

28. The Birth of a Function (``MIR_new_func``)
----------------------------------------------

We have arrived at the shipyard. This is where the mighty vessels—Functions—are constructed.
The process is elaborate, for a function is not merely a list of instructions; it is a self-contained world with its own laws (prototypes) and citizens (variables).

*   **The Blueprint:** ``new_func_arr`` is the master builder. It demands a name, return types, and arguments.
*   **The Nesting Trap:** A check (``if (curr_func != NULL)``) ensures you aren't trying to build a ship inside another ship. MIR forbids nested function definitions.
*   **The Type Police:** It ruthlessly checks if you are trying to return something illegal, like a block (``MIR_wrong_type_error``).
*   **The Census:** It initializes the local variable registry (``func_regs_init``) and registers every argument as a local variable. Note the ``i + 1`` logic—register 0 is reserved, so arguments start at register 1.

29. The Final Inspection (``MIR_finish_func``)
----------------------------------------------

Once the hull is laid and the instructions are loaded, the **inspector** arrives. This function is a massive, multi-pass validator. It walks the deck of the ship (the instruction list) and checks every rivet.

*   **The Role Call:** It verifies that every operand matches the instruction's expected mode. You cannot pass a float to an integer add. You cannot jump to a variable.
*   **The Return Protocol:** It ensures ``RET`` instructions match the function's return signature. If you promised to return an ``int`` and a ``float``, you better have two operands of the correct types.
*   **The Fallback Plan:** If you forgot to put a ``ret`` at the end of your function (and it's not a ``jret`` function), the inspector graciously bolts one on for you, returning zero/null by default.

This function transforms a raw list of instructions into a verified, executable entity. It sets the ``expr_p`` flag if the function is simple enough to be used as a compile-time expression (no side effects).

30. The Global Stage (``setup_global``)
---------------------------------------

This function is the **Ambassador**. It takes a raw pointer (``void *addr``)—perhaps the address of ``printf`` in libc—and wraps it in a MIR Import item.

*   **The Environment Module:** Notice ``curr_module = &environment_module``. Globals like ``printf`` or ``malloc`` don't belong to your user module; they belong to the **Environment**—the shared context that all modules can see.
*   **The Redefinition Check:** It checks if you are trying to redefine a global. If so, it returns ``TRUE``, signaling a potential conflict or update.

31. The Void (``undefined_interface``)
--------------------------------------

What happens if you try to call a function that hasn't been linked? You stare into the abyss.
``undefined_interface`` is the default address for unresolved symbols. If execution reaches here, it screams ``MIR_call_op_error`` and the program dies. It is the cliff's edge.

**Summary:** We have seen the birth of functions, the rigorous validation of their internal logic, the registration of global symbols, and the materialization of data segments into real memory. The ship is built, inspected, and loaded with cargo. Next, we must launch it.

33. The Loading Dock (``load_bss_data_section``)
------------------------------------------------

Finally, we arrive at the heavy lifting—the physical materialization of data. ``load_bss_data_section`` is the cargo master of our expedition.

This function is responsible for taking the abstract blueprints of data (``MIR_data_item``, ``MIR_bss_item``, etc.) and laying them out in actual, physical RAM.

*   **The Size Calculation:** Before we allocate a single byte, we must measure the cargo. The loop iterates through consecutive data items (data, bss, ref, lref, expr). It sums up their sizes, ensuring we allocate a contiguous block of memory.
    *   **The Contiguity Promise:** MIR guarantees that adjacent anonymous data items (items without names) are loaded into a single contiguous block. This is crucial for arrays or structures split across multiple MIR items.
    *   **The Alignment:** ``section_size`` is rounded up to 8 bytes. In the world of high-performance JITs, unaligned access is a sin we do not commit.

*   **The Allocation:** ``MIR_malloc`` carves out the territory. If it fails, the expedition ends here (``MIR_alloc_error``).

*   **The Population:** Once the land is claimed, we fill it.
    *   **BSS:** Zeroes are swept across the memory (``memset``).
    *   **Data:** Raw bytes are copied (``memmove``).
    *   **Refs/LRefs/Exprs:** We reserve space (pointers) but leave them for the Linker to fill later. Their ``load_addr`` is set to point into this new memory block.

This function transforms abstract intent into concrete bytes. It is the bridge between the compiler's mind and the CPU's reality.

34. The Cartographer's Survey (``link_module_lrefs``)
-----------------------------------------------------

Label References (``lref``) are tricky beasts. They allow data to point to a code label *inside* a function (think "jump tables" or "computed gotos"). But a label is just an instruction in a list; it has no inherent "address" until code generation.

*   **The Problem:** ``lref`` data items exist at the module level, but they point to labels inside functions. How do we verify they point to the *same* function?
*   **The Pass:** This function performs a survey.
    1.  It iterates through all functions, tagging every label instruction with a pointer to its owning function (``insn->data = item->u.func``).
    2.  It then checks every ``lref`` item. It follows the label pointer, checks the tag, and verifies that ``label->func == label2->func``.
    3.  If they match, it links the ``lref`` into the function's own list (``func->first_lref``).
    4.  Finally, it cleans up the tags (``insn->data = NULL``).

This ensures that when we generate machine code for a function, we know exactly which data items depend on its internal addresses.

35. The Grand Opening (``MIR_load_module``)
-------------------------------------------

This is the ribbon-cutting ceremony.

.. code-block:: c

    void MIR_load_module (MIR_context_t ctx, MIR_module_t m)

*   **The Simplification:** Before a module can be run, it must be simplified. ``MIR_load_module`` triggers this transformation (though the actual call happens later in linking, the setup starts here).
*   **The Thunks:** Every function gets an entry point (``item->addr``). Initially, this points to a **Thunk**—a small piece of trampoline code. Why? Because we haven't generated the machine code yet! The thunk is a placeholder that says, "I'm not ready yet."
*   **The Redirect:** The thunk is set to redirect to ``undefined_interface``. If you try to call this function before linking, you crash. Safety first.
*   **The Exports:** If an item is exported, it is registered in the Global Environment. This makes it visible to other modules. But beware the ``MIR_repeated_decl_error``! The global namespace is a crowded market; you cannot claim a stall that is already taken.

36. The Diplomatic Landing (``MIR_load_external``)
--------------------------------------------------

Sometimes, you need natives. You need ``printf``, ``malloc``, ``exit``. These functions live outside the MIR universe, in the host C runtime.

``MIR_load_external`` bridges this gap. It registers a name (e.g., "printf") and a raw C function pointer. It wraps this in a ``MIR_import_item`` inside the special ``environment_module``. Now, your JITed code can call C code as if it were just another MIR function.

**Special Case:** ``setjmp``. This function is magic. It manipulates the stack in ways that terrify ordinary code. MIR recognizes it by name (``SETJMP_NAME``) and stores its address specially (``setjmp_addr``), likely because the interpreter needs to handle it with extreme caution.

**Summary:** We have loaded the cargo, surveyed the land, established diplomatic ties with the host OS, and opened the module for business. The only thing left is to connect the wires—**Linking**—and then, finally, **Execution**.

37. The Grand Junction (``MIR_link``)
-------------------------------------

We have now reached the nexus point. The modules have been loaded, the functions built, and the data allocated. But they are islands in an archipelago, disconnected and lonely. ``MIR_link`` is the bridge builder. It wires everything together.

This function is a multi-pass beast, a whirlwind of resolution and connection.

37.1 Pass 1: The Gathering (Imports and Simplification)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

The first loop iterates through every module waiting in the ``modules_to_link`` queue.

*   **Simplification:** It calls ``simplify_func``. We'll explore this later, but know that it polishes raw MIR code into something the generator can handle (removing complex operands, canonicalizing instructions).
*   **The Hunt for Imports:** It scans for ``import`` items.
    *   **Internal Search:** First, it checks the ``environment_module`` (the global registry).
    *   **External Search:** If not found, it summons the ``import_resolver`` provided by the user (usually ``dlsym`` or similar). This is how MIR finds ``printf`` in the host process.
    *   **The Link:** Once found, the import item is updated with the real address (``tab_item->addr``). The missing link is forged.
*   **The Handshake (Exports/Forwards):** It resolves exports and forwards against their definitions within the module, ensuring every promise is kept.

37.2 Pass 2: The Inlining and Execution
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

The second loop revisits the modules, now that symbols are resolved.

*   **Inlining:** ``process_inlines``. If a function call is marked ``MIR_INLINE``, the linker attempts to smash the callee's code directly into the caller. This is a massive optimization step, eliminating call overhead.
*   **Ref Data:** It fixes up ``ref_data`` items. Remember those? ``int *p = &x;``. Now that ``x`` has an address, we can write ``&x`` into the memory of ``p``.
*   **The Oracle Speaks (``expr_data``):**
    *   This is the coolest part. ``expr_data`` needs a value computed at runtime.
    *   ``MIR_interp`` is called right here, right now. The interpreter runs the expression function *during the linking phase*.
    *   The result (be it ``int``, ``float``, or ``ptr``) is captured in ``res`` and then ``memcpy``'d directly into the target memory. This allows for incredibly complex static initialization that C++ static constructors can only dream of.

37.3 The Final Interface: ``set_interface``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

The last loop is the handover.

*   The ``set_interface`` callback is invoked for every function.
*   This is where the user says: "I want to interpret this" (``MIR_set_interp_interface``) or "I want to compile this to machine code" (``MIR_set_gen_interface``).
*   The system configures the entry points accordingly.

**Result:** A fully interconnected web of code and data, ready to run.

38. The Simplifier's Toolbox (``simplify_func``)
------------------------------------------------

Before we can generate machine code, we must tame the wild MIR. The ``simplify_func`` is the logic that strips away the syntactic sugar of MIR to leave only the hard, cold reality of machine-operable instructions.

*   **Complex Memory:** Hardware instructions generally can't do ``mem = mem``. They can do ``reg = mem`` or ``mem = reg``. The simplifier detects these "memory-to-memory" moves and splits them into two instructions using a temporary register.
*   **Alloca Optimization:** It consolidates adjacent stack allocations. Why grow the stack ten times by 8 bytes when you can grow it once by 80 bytes?
*   **Dead Code:** It spots trivial dead code (jumps to the next instruction) and prunes it.
*   **Retargeting:** It handles ``MIR_switch``, ``MIR_ret``, and other high-level constructs, preparing them for the lower-level analysis passes (Live Variable Analysis, Register Allocation) that live in ``gen``.

**Why is this separate from generation?**
Because ``simplify_func`` modifies the *MIR itself*. The Generator (``gen``) assumes the MIR is in a "canonical" form. By separating this pass, the Generator can be simpler and faster, focusing on register allocation and machine code emission rather than AST rewriting.

**Summary:** We have seen the linker weave connections between modules and the host, execute build-time logic to initialize data, and prepare the functions for the final stage. The MIR context is now fully primed. The next step is the descent into the **Generator**—where abstract IR becomes concrete CPU instructions.

39. The Unnamed Legion: Temporary Registers
-------------------------------------------

The deeper we go into the compiler's machinery, the less things have names and the more they have numbers.
``new_temp_reg`` (and its public face ``_MIR_new_temp_reg``) is the recruiter for this legion.

*   **The Check:** Temp registers are simple creatures. They only understand primitive types: ``I64``, ``F``, ``D``, ``LD``. You cannot have a temporary block.
*   **The Generation:** It loops, incrementing ``last_temp_num`` and checking if a name like ``t42`` is taken. It keeps spinning the wheel until it finds a unique ID.
*   **The Recruitment:** Once a name is found, it calls ``MIR_new_func_reg`` to officially register this soldier in the function's roster.

Why do we need this? Optimization passes like **CSE** (Common Subexpression Elimination) need places to store intermediate results that the user never asked for.

40. The Census Bureau: Register Lookup
--------------------------------------

We have ``MIR_reg``, ``MIR_reg_type``, ``MIR_reg_name``. These are the bureaucrats.

*   **``MIR_reg``**: "I have a name ('count'), give me the ID (5)."
*   **``MIR_reg_name``**: "I have an ID (5), give me the name ('count')."
*   **``MIR_reg_type``**: "What is ID 5? Ah, it's a 64-bit integer."

Under the hood, they delegate to ``find_rd_by_name`` and ``find_rd_by_reg``. These helpers dive into the ``func_regs`` hash tables we saw earlier. If a lookup fails, ``MIR_undeclared_func_reg_error`` is raised—the bureaucrat couldn't find your file.

41. The Instruction Forge: ``MIR_new_insn`` & Friends
-----------------------------------------------------

We return to the factory floor.
If ``MIR_new_func`` built the ship, ``MIR_new_insn`` builds the engine parts.

*   **The Validator:** The first thing ``MIR_new_insn`` does is consult the Book of Spells (``insn_descs``). It checks ``insn_code_nops``. If you try to build an ``ADD`` instruction with 5 operands, it stops you.
*   **The Exception:** Some instructions are chaotic—``CALL``, ``RET``, ``PHI``, ``UNSPEC``. These have variable operands. The validator refuses to build them via the standard ``MIR_new_insn`` function, forcing you to use specialized constructors like ``MIR_new_call_insn``. This prevents accidental misuse.
*   **The Construction:** ``va_start`` gathers the arguments, and ``MIR_new_insn_arr`` does the heavy lifting.

41.1 ``MIR_new_insn_arr``: The Master Builder
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

This is the core logic for instruction creation. It is rigorous.

1.  **Opcode Check:** It verifies the opcode is valid.
2.  **Operand Count:** It verifies the number of operands matches the specification.
3.  **Prototype Check (Call/Unspec):** If it's a call, it verifies the operands match the function prototype. "You said this function takes 3 ints, but you gave me a float? **Error.**"
4.  **Memory Type Check:** If an operand is memory (``MIR_OP_MEM``), it checks if the type makes sense. You can't load a block if the function expects an int.
5.  **Property Checks:** It validates ``MIR_PRSET`` and ``MIR_PRBEQ``, ensuring property manipulation uses integer operands.

Finally, it allocates the memory (``create_insn``) and populates the ``ops`` array.

42. Copying Reality: ``MIR_copy_insn``
--------------------------------------

Sometimes, optimization requires cloning. Loop unrolling, inlining—these processes need identical copies of instructions.
``MIR_copy_insn`` is the cloning vat. It calculates the exact size of the instruction (header + operands) and performs a raw memory copy. Simple, efficient, dangerous if misuse (pointers to data might need deep copying, but MIR operands are mostly values or references to immutable items).

43. The Label Maker: ``MIR_new_label``
--------------------------------------

A label is just an instruction that does nothing but mark a spot.

*   It creates a ``MIR_LABEL`` instruction.
*   It gives it a unique ID (``curr_label_num++``).
*   This ID is stored as an integer operand.

Later, jumps will refer to this ID to know where to land.

**Summary:** The armory is fully stocked. We can create simple register references, complex memory addressing modes, and everything in between. We can compare them for equality and fingerprint them for storage. We are now fully equipped to build the instructions themselves.

**Next Stop:** The actual construction of the instruction list (``MIR_append_insn``), and handling module-level context switching (``MIR_change_module_ctx``).

47. The Time Machine: ``_MIR_duplicate_func_insns`` and ``_MIR_restore_func_insns``
-------------------------------------------------------------------------------------

We have stumbled upon the secret to time travel. Or at least, the JIT equivalent.

When the optimizer descends upon a function, it tears it apart. It moves code, deletes instructions, and renames variables. But what if we want to compile the same function twice? Or what if something goes wrong?
We need a backup.

*   **``_MIR_duplicate_func_insns`` (The Save State):** This function creates a perfect copy of the function's body.
    *   It saves the original instruction list to ``original_insns``.
    *   It creates a new list in ``insns`` and populates it with clones (``MIR_copy_insn``).
    *   **The Label Puzzle:** Cloning jump instructions is hard. A jump to "Label A" in the original code must become a jump to "Clone of Label A" in the new code. The function meticulously builds a map (``labels`` varr) linking original labels to their clones, and then patches up all branches (``redirect_duplicated_labels``). It's like rewriting a treasure map so it still leads to the treasure, even after you've moved the island.

*   **``_MIR_restore_func_insns`` (The Reset Button):** This undoes the chaos.
    *   It purges all the temporary variables created during optimization.
    *   It destroys the modified instruction list.
    *   It restores the ``original_insns`` list back to its rightful place.
    *   The function is pristine again, ready for another round of optimization or a different target architecture.

48. The Identity Shifter: ``MIR_change_module_ctx``
---------------------------------------------------

(This function appeared in the previous snippet but deserves mention).
Imagine moving an entire city from one planet to another. That is what ``MIR_change_module_ctx`` does.
It takes a module from one ``MIR_context`` and grafts it onto another.

#### **48.1 The "Thread Safety" Warning**

You asked about the ominous comment: `/* It is not thread-safe */`.

Let us be blunt: **This operation is an earthquake.**

To move a module, this function must:
1.  Rip the module out of the `old_ctx`'s linked list.
2.  Surgically extract every item from the `old_ctx`'s hash table.
3.  Inject the module into `new_ctx`'s linked list.
4.  Insert every item into `new_ctx`'s hash table.

If another thread is doing *anything* in `old_ctx` (generating code, looking up a symbol) or `new_ctx` while this teleportation is happening, the ground will open up and swallow your application. Pointers will dangle. Lists will become circular. Chaos will reign.

**The Law:** If you touch the teleporter, you must freeze time (use Mutexes) in both universes.

#### **48.2 The Great Translation**

Why is this function so long? Why does it iterate over every instruction?

Because of **Strings**.

In MIR, strings are interned. In Context A, the string `"count"` might be stored at address `0x1000`. In Context B, `"count"` might not exist yet, or it might be at `0x8000`.
If you simply move the module struct, all its internal pointers (function names, variable names, string literals) will point to memory owned by the *Old Context*. If you then destroy the Old Context, your module in the New Context is holding a map to a land that no longer exists.

This function performs a **Total Translation**:
*   **Module Name:** Re-interned in the new context.
*   **Item Names:** Every function and global is renamed. The Hash Table buckets are recalculated (because the pointer address changed).
*   **Variable Names:** Every local variable and argument is visited and re-registered.
*   **Hard Registers:** Even hardware register names are context-specific strings.
*   **Instruction Operands:**
    *   **String Literals (``MIR_OP_STR``)**: The string content is copied to the new context's vault.
    *   **Memory Aliases (``MIR_OP_MEM``)**: Alias names are re-interned.

It is a tedious, exhaustive migration. Every signpost must be repainted.

#### **48.3 The "No Flying" Rule**

Notice this check:
.. code-block:: c

    if (item->addr != NULL)
      MIR_get_error_func (old_ctx) (MIR_ctx_change_error, "Change context of a loaded module");

You cannot teleport a module that has already been **Loaded** (linked/JITted). Once a module is turned into machine code, it is cemented into the fabric of the host process. Its addresses are baked in. You cannot move a building while people are working inside it.

49. The Printing Press: ``MIR_output``
--------------------------------------

We now leave the complex logic of context switching and return to the observers. The code snippets provided earlier included ``MIR_output``, ``MIR_output_item``, etc.

These are the **Scribes**.

When you are deep in the mines of JIT debugging, you cannot read raw C structs. You need a map. ``MIR_output`` takes the current state of the AST and renders it back into the human-readable MIR assembly language.

*   **``type_str``**: Converts enum ``MIR_T_I64`` -> `"i64"`.
*   **``mode_str``**: Converts enum ``MIR_OP_REG`` -> `"reg"`.
*   **``MIR_output_insn``**: Reconstructs the textual form of an instruction, handling the complex syntax of memory operands (``type: disp(base, index, scale)``).

This serialization capability is vital. It allows you to:
1.  Generate MIR in memory.
2.  Dump it to a file.
3.  Read it back in later (using the Scanner we saw earlier).
It proves that MIR is fully reflective—it can describe itself perfectly.

50. The Binary Scribes: ``MIR_write`` and ``MIR_read``
------------------------------------------------------

Text is great for humans, but machines prefer something denser. We are now entering the domain of **Serialization**.
This isn't just ``fwrite(&struct, sizeof(struct))``. Pointers are meaningless on disk. We need a format.

*   **The Tag System:** The binary format is a stream of tokens, each starting with a **Tag** (``bin_tag_t`` enum).
    *   ``TAG_I8`` means "next byte is an 8-bit integer".
    *   ``TAG_NAME`` means "next token is a string index".
    *   ``TAG_OP`` means "start of an operand".
*   **Compression:** MIR uses a custom compression scheme (implemented in ``mir-reduce.h``, which we saw included earlier). It's not ZIP; it's a structural compression. It deduplicates strings and uses variable-length integers (LEB128-style logic) to save space. A ``0`` takes one byte, not eight.
*   **The Reader:** ``MIR_read`` is a recursive descent parser, but for binary tags instead of text. It reconstructs the entire module hierarchy, resolving string references and rebuilding the instruction lists.

**This concludes the Core MIR implementation.** We have covered:

1.  **Context & Memory:** The universe and its laws.
2.  **Items & Data:** The objects that populate the universe.
3.  **Instructions & Operands:** The atoms of behavior.
4.  **Serialization:** How to save and load the universe.

The next major file in our journey will be the **Generator** (``mir-gen.c``) or the **Interpreter** (``mir-interp.c``). But for now, we rest. The foundation is laid.


44. The Armory: Operand Construction
------------------------------------

We have left the shipyards where massive functions are constructed and entered the armory. Here, the individual bullets, shells, and power cells—the **Operands**—are forged.

Every instruction consumes operands. ``ADD`` needs two inputs and one output. ``JMP`` needs a label. This section of the code provides the tools to build them.

44.1 The Primordial Mold: ``init_op``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

All operands start as raw clay. ``init_op`` sets the ``mode`` tag (Reg? Int? Mem?) and clears the ``data`` field. This ``data`` field is a secret compartment used later by the optimizer to store data-flow information (like liveness or SSA edges). For now, it remains empty.

44.2 The Elementals
~~~~~~~~~~~~~~~~~~~

*   **``MIR_new_reg_op``**: Creates a reference to a register. It takes a ``MIR_reg_t`` ID. Simple, elegant, deadly.
*   **``MIR_new_int_op`` / ``MIR_new_uint_op``**: These forge immediate integer constants.
*   **``MIR_new_float_op`` / ``MIR_new_double_op``**: The floating-point equivalents. Notice the ``mir_assert(sizeof(float) == 4)``. MIR relies on IEEE 754 standards. If you try to run this on a VAX from 1980, the compiler will explode right here.

44.3 The Specialized Projectiles
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

*   **``MIR_new_str_op``**: This creates a string literal operand. Crucially, it calls ``get_ctx_string``, ensuring the string text is interned in the Great Archives.
*   **``MIR_new_ref_op``**: A reference to a global symbol (Function, Import, etc.). It stores the pointer to the ``MIR_item_t``.

44.4 The Cluster Bomb: ``MIR_new_mem_op``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

This is the heavy artillery. Memory operands are complex beasts: ``type: disp(base, index, scale)``.

*   **Type:** How many bytes to read/write? (e.g., ``MIR_T_I32``).
*   **Disp:** Constant offset.
*   **Base & Index:** Register IDs.
*   **Scale:** Multiplier for the index.
*   **Alias/Nonalias:** These optional flags (passed via ``MIR_new_alias_mem_op``) give hints to the optimizer about memory overlap.

The function ``new_mem_op`` packs all these parameters into the ``MIR_mem_t`` struct hidden inside the operand union.

45. The Identifier: ``MIR_op_eq_p``
-----------------------------------

In the chaotic battlefield of optimization, the compiler often needs to ask: "Are these two bullets the same?"

*   **Simple types:** Integers and floats are compared by value.
*   **Registers:** Compared by ID.
*   **Memory:** This is the hard part. It must compare *every single field*: type, displacement, base register, index register, and scale. If even one bit differs, they point to different universes.
*   **References:**
    *   Normally, it compares the item pointers (``op1.u.ref == op2.u.ref``).
    *   **The Plot Twist:** If the items are Imports or Exports, it compares their *Names*. Why? Because an Import in Module A and an Export in Module B might be distinct ``MIR_item_t`` structs, but they refer to the same logical entity "printf".

46. The Fingerprint: ``MIR_op_hash_step``
-----------------------------------------

To store operands in hash tables (for Common Subexpression Elimination or Global Value Numbering), we need to fingerprint them.
This function takes an existing hash ``h`` and mixes in the data from the operand ``op``.

*   It walks through the same logic as ``eq_p``, feeding register IDs, integer values, and memory offsets into the ``mir_hash_step`` mixer.
*   **Floating Point Hashing:** Notice the unions for floats and doubles. We cannot hash floating point values directly because of NaN weirdness and bit representation. The code type-puns them to ``uint64_t`` or ``uint32_t`` to hash their raw bit patterns. This ensures that ``-0.0`` and ``+0.0`` might hash differently if their bits differ, preserving strict identity.

**Summary:** The armory is fully stocked. We can create simple register references, complex memory addressing modes, and everything in between. We can compare them for equality and fingerprint them for storage. We are now fully equipped to build the instructions themselves.

**Next Stop:** The actual construction of the instruction list (``MIR_append_insn``), and handling module-level context switching (``MIR_change_module_ctx``).

47. The Time Machine: ``_MIR_duplicate_func_insns`` and ``_MIR_restore_func_insns``
-------------------------------------------------------------------------------------

We have stumbled upon the secret to time travel. Or at least, the JIT equivalent.

When the optimizer descends upon a function, it tears it apart. It moves code, deletes instructions, and renames variables. But what if we want to compile the same function twice? Or what if something goes wrong?
We need a backup.

*   **``_MIR_duplicate_func_insns`` (The Save State):** This function creates a perfect copy of the function's body.
    *   It saves the original instruction list to ``original_insns``.
    *   It creates a new list in ``insns`` and populates it with clones (``MIR_copy_insn``).
    *   **The Label Puzzle:** Cloning jump instructions is hard. A jump to "Label A" in the original code must become a jump to "Clone of Label A" in the new code. The function meticulously builds a map (``labels`` varr) linking original labels to their clones, and then patches up all branches (``redirect_duplicated_labels``). It's like rewriting a treasure map so it still leads to the treasure, even after you've moved the island.

*   **``_MIR_restore_func_insns`` (The Reset Button):** This undoes the chaos.
    *   It purges all the temporary variables created during optimization.
    *   It destroys the modified instruction list.
    *   It restores the ``original_insns`` list back to its rightful place.
    *   The function is pristine again, ready for another round of optimization or a different target architecture.

48. The Identity Shifter: ``MIR_change_module_ctx``
---------------------------------------------------

(This function appeared in the previous snippet but deserves mention).
Imagine moving an entire city from one planet to another. That is what ``MIR_change_module_ctx`` does.
It takes a module from one ``MIR_context`` and grafts it onto another.

*   **The Challenge:** Strings. Every string in MIR (names of functions, variables, etc.) is interned in the context's specific string table. A string pointer valid in Context A is meaningless garbage in Context B.
*   **The Solution:** The function iterates through *every single item* and *every single instruction* in the module, re-interning every string in the new context. It's a massive migration effort, ensuring the module assimilates perfectly into its new home.

49. The Printing Press: ``MIR_output``
--------------------------------------

We have spent so much time building structures in memory. Now, we must learn to speak. ``MIR_output`` (and its helper ``MIR_output_item``) converts the internal binary representation back into human-readable MIR text.

*   **The Formatter:** It knows the syntax. It prints ``func``, ``proto``, ``import``.
*   **The Details:** It prints arguments, return types, and local variables.
*   **The Code:** It iterates through the instruction list, calling ``MIR_output_insn``.
    *   For registers, it prints names (``%r1``).
    *   For memory, it prints the complex ``type: disp(base, index, scale)`` format.
    *   For branches, it prints label names.

This is essential for debugging. If your JIT is generating crashing code, you dump the MIR to a file and read it. If the text looks wrong ("Why am I jumping to a variable named ``format_string``?"), you've found your bug.

50. The Binary Scribes: ``MIR_write`` and ``MIR_read``
------------------------------------------------------

Text is great for humans, but machines prefer something denser. We are now entering the domain of **Serialization**.
This isn't just ``fwrite(&struct, sizeof(struct))``. Pointers are meaningless on disk. We need a format.

*   **The Tag System:** The binary format is a stream of tokens, each starting with a **Tag** (``bin_tag_t`` enum).
    *   ``TAG_I8`` means "next byte is an 8-bit integer".
    *   ``TAG_NAME`` means "next token is a string index".
    *   ``TAG_OP`` means "start of an operand".
*   **Compression:** MIR uses a custom compression scheme (implemented in ``mir-reduce.h``, which we saw included earlier). It's not ZIP; it's a structural compression. It deduplicates strings and uses variable-length integers (LEB128-style logic) to save space. A ``0`` takes one byte, not eight.
*   **The Reader:** ``MIR_read`` is a recursive descent parser, but for binary tags instead of text. It reconstructs the entire module hierarchy, resolving string references and rebuilding the instruction lists.

51. The Output Machinery Revisited: ``MIR_output`` (and friends)
----------------------------------------------------------------

We have returned to the **Scribes** we glimpsed in the header file, but now we see their inner workings. These functions are the voice of the compiler, translating the internal binary representation into human-readable text. This is crucial for debugging, testing, and understanding what the JIT is actually doing.

51.1 The Operands Scribe: ``MIR_output_op``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

This function is a meticulous translator. It takes a raw ``MIR_op_t`` and renders it faithfully.

*   **Registers (``MIR_OP_REG``/``MIR_OP_VAR``):** It looks up the name (``output_reg``, ``output_var``). If it's a hard register, it prints ``hrN``. If it's a virtual register, it prints the user-defined name.
*   **Constants:** It uses ``PRId64``, ``PRIu64``, and scientific notation (``%.*e``) for floats to ensure precision is not lost in translation.
*   **Memory (``MIR_OP_MEM``):** This is the most complex part. It reconstructs the assembly syntax: ``type: disp(base, index, scale)``. It checks for aliases and prints them if present.
*   **References & Labels:** It resolves pointers back to their names (``MIR_item_name``).

51.2 The Instruction Scribe: ``MIR_output_insn``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Here we see the assembly line.

*   **Labels:** It handles labels specially (``L<ID>:``).
*   **Opcodes:** It fetches the mnemonic string (``MIR_insn_name``).
*   **Operands:** It iterates through the operand array, calling ``MIR_output_op`` for each, separating them with commas.
*   **Annotations:** For ``UNSPEC`` instructions (unspecified machine-specific ops), it helpfully prints the prototype name as a comment, so you know what that mystery instruction is supposed to be doing.

51.3 The Item Scribe: ``MIR_output_item``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

This function handles the diversity of module items.

*   **Simple Items:** Exports, Imports, Forwards are one-liners.
*   **Data/BSS:** It prints the type and value. For byte arrays (``MIR_T_U8``), it cleverly checks if it looks like a string (null-terminated) and prints it as a C-string comment for readability.
*   **Functions:** This is the big one.
    *   It prints the signature (``func``).
    *   It dumps the local variable table (``local ...``).
    *   It iterates through the entire instruction list, dumping the body.
    *   It ends with ``endfunc``.

51.4 The Module Scribe: ``MIR_output_module``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

The top-level orchestrator. It wraps the items in ``module <name> ... endmodule`` blocks.

**Summary:** The output system is robust. It doesn't just dump raw data; it reconstructs the *source code* of the MIR program. This means MIR is reversible: you can generate it in memory, dump it to a file, and read it back (using the scanner) without losing information.

52. The Simplification Pass (``simplify_func``)
-----------------------------------------------

We are now looking at the **Simplification Engine**. This is not just optimization; it is **canonicalization**. The generator expects instructions in a very specific format. ``simplify_func`` beats the wild user input into shape.

*   **Memory-Memory Moves:** Most CPUs cannot move data directly from RAM to RAM. ``simplify_func`` detects ``MOV mem, mem`` and injects a temporary register: ``MOV temp, mem; MOV mem, temp``.
*   **Immediate-Immediate Moves:** Similarly, ``MOV mem, imm`` might be illegal on some architectures. It might be split.
*   **Complex Operands:** It lowers complex operands that might be valid in high-level MIR but invalid in machine code.
*   **Dead Code:** It removes trivial dead code (jumps to the next instruction) to save the generator from doing it.

This pass ensures that the ``gen`` module doesn't have to handle every possible edge case of valid MIR; it only has to handle "Simplified MIR".

53. The Scanner and Parser (``scan_token``, ``MIR_scan_string``)
----------------------------------------------------------------

We have seen the output; now we see the input. ``MIR_scan_string`` is the **Lexer** and **Parser** combined.

*   **The State Machine:** It walks the string character by character.
*   **The Tokenizer:** It identifies keywords (``module``, ``func``, ``add``, ``mov``), numbers (integers, floats, hex), strings, and identifiers.
*   **The recursive descent:** It identifies the current context (are we in a module? in a function?) and dispatches to the appropriate creation function (``MIR_new_module``, ``MIR_new_insn``).
*   **Error Handling:** It uses ``setjmp``/``longjmp`` for error recovery, a classic C technique for exception handling in parsers. If it hits a syntax error, it jumps back to the recovery point, prints a message with the line number, and aborts.

54. The End of the Journey: ``MIR.c`` Complete
----------------------------------------------

We have traversed the entire ``mir.c`` file.

1.  **Context & Memory**: The foundation.
2.  **Containers**: Modules, Items, Lists.
3.  **Data**: Strings, Binary Data, BSS.
4.  **Logic**: Functions, Instructions, Operands.
5.  **IO**: Reading and Writing textual MIR.
6.  **Binary IO**: Reading and Writing compressed binary MIR.
7.  **Simplification**: Preparing code for generation.
8.  **Linking**: Connecting modules and the host.

The Core is complete. It is a self-contained universe that can represent, manipulate, save, load, and link code.

**What remains?**
The machine code itself. We have built the *idea* of the program. Now we must translate that idea into the specific dialect of the processor (x86_64, ARM64, etc.). That happens in **``mir-gen.c``**. That is where the rubber meets the road.






