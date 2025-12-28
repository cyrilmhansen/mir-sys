The Simulator: Implementation of ``mir-interp.c``
======================================================

Welcome to the **Simulator**.

If ``mir.c`` is the architect's map and ``mir-gen.c`` is the foundry building the heavy machinery, then ``mir-interp.c`` is the virtual laboratory. Here, we don't transform code into silicon commands; we act it out. It is a world of roleplay where every instruction is a script to be followed by the interpreter's clockwork engine.

The interpreter is an obligatory part of the MIR API, providing a safe haven for code execution on architectures where a JIT generator is not yet available or where security policies (like W^X) forbid the creation of new executable memory.

1. The Interpreter's Workspace: ``interp_ctx``
----------------------------------------------

Every virtual universe needs a foundation. The ``interp_ctx`` structure holds the global state of the interpreter.

*   **The Dispatch Table**: If the compiler supports it, the interpreter uses **Direct Threaded Dispatch**. Instead of a giant ``switch`` statement, it maintains a table of memory addresses (labels) for each instruction, allowing it to jump directly from one "act" to the next.
*   **The Global Registers**: These are the interpreter's equivalent of hardware registers, used to store values that persist across the virtual world.
*   **The Stack**: The interpreter manages its own virtual stack for function calls, ensuring that recursive MIR functions don't blow up the host's physical stack.

2. Translating the Script: ICode Generation
-------------------------------------------

Before a function can be acted out, it must be translated from the user-friendly ``MIR_insn_t`` into a more efficient internal format called **ICode**.

*   **The Transformation**: ``generate_icode`` walks through the function's instructions and packs them into a dense array of ``MIR_val_t`` objects.
*   **The Result**: This flattened representation is much faster to traverse than a doubly-linked list of complex structs. It's essentially "Interpreter Assembly."

2.1 The Chameleon Opcodes: ``MIR_full_insn_code_t``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

In the abstract IR, a ``MOV`` is just a ``MOV``. But in the interpreter's world, we need to know exactly what kind of move it is to avoid expensive checks during the main loop.

*   **Specific Roles**: ``generate_icode`` specialized generic opcodes into internal variants like ``IC_LDI8`` (Load Int 8), ``IC_MOVI`` (Move Immediate), or ``IC_MOVFG`` (Move from Global).
*   **Pre-computation**: By the time the main loop starts, all the difficult questions ("Is this a memory load?", "Is this an immediate?") have already been answered and encoded into the ICode.

2.2 Connecting the Dots: Branch Resolution
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

In textual MIR, jumps target labels. In the ICode array, labels don't exist; they are just indices.

*   **The Patching**: After the first pass of ICode generation, the interpreter performs a second pass to resolve all branch targets. It replaces the abstract "Label A" with the integer offset of the target instruction in the ``code_varr`` array.
*   **Speed**: This turns a symbolic search into a simple array index jump, keeping the simulator's heart beating fast.

3. The Heartbeat: The Main Interpretation Loop (``eval``)
--------------------------------------------------------------------------------

Once the script is translated into ICode, the interpreter enters its main loop: ``eval``. This function is the processor's virtual soul.

3.1 The Great Choice: Dispatch Methods
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

How the interpreter jumps from one instruction to the next determines its fundamental speed.

*   **The Standard Switch**: The most portable method. A giant ``switch(opcode)`` statement inside a ``while(1)`` loop. It's easy to write but can be slow due to the overhead of the loop control and the switch jump.
*   **Direct Threaded Dispatch**: If the compiler (like GCC or Clang) supports "Labels as Values," MIR uses a more aggressive approach.
    *   **The Labels**: Every instruction handler starts with a label (e.g., ``L_MIR_ADD:``).
    *   **The Jump**: Instead of returning to a central loop, each handler ends with a jump directly to the address of the *next* instruction's handler. It's like a relay race where the baton is never dropped.

.. note::
   **Compiler Lore: The Branch Predictor's Nightmare**
   
   In a standard ``switch`` loop, there is one single "indirect jump" at the end of the loop that targets every possible handler. This confuses the CPU's **Branch Predictor**, as it can't guess which instruction is coming next.
   
   In **Direct Threaded Dispatch**, every handler has its own jump. The CPU learns the patterns *specific to your code*. If your virtual program often follows an ``ADD`` with a ``STORE``, the CPU hardware will eventually predict that jump perfectly, resulting in a significant speed boost without a single line of assembly.

3.2 Acting Out the Instructions
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Each "Act" in the simulator corresponds to a MIR opcode.

*   **Arithmetic (``IOP3``, ``FOP3``, etc.)**: These macros perform the actual math. They fetch values from the virtual registers (the ``bp`` pointer), apply the C operator (``+``, ``-``, ``*``), and store the result back.
*   **Memory (``LD``, ``ST``)**: These handle the bridge to the host's RAM. They calculate addresses and perform raw memory accesses, acting as the simulator's hands in the real world.
*   **Branching (``BICMP``, etc.)**: These manipulate the virtual **Program Counter (PC)**. If a comparison is true, they set the PC to the target index in the ICode array, causing the engine to "jump" to a new part of the script.

3.3 Debugging the Simulation: Tracing
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

If ``MIR_INTERP_TRACE`` is enabled, the interpreter becomes a verbose storyteller.

*   **The Narrative**: It prints every instruction it executes, along with the input values and the final result.
*   **The Identity**: It uses the interning system to print register names, making the trace look like a live-action assembly listing. It's an invaluable tool for finding out exactly where a virtual program went off the rails.

4. The Actors' Craft: Instruction Macros
-----------------------------------------

The interpreter's handlers are not written in raw C code for every possible instruction. That would be a maintenance nightmare. Instead, MIR uses a sophisticated set of **C Macros** to generate the handlers.

4.1 The Stage Hands: ``IOP3``, ``FOP3``, etc.
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

These macros are the **Stage Hands**. They handle the repetitive work of fetching operands and storing results.

*   **``IOP3(op)``**: Handles 3-operand integer instructions (e.g., ``ADD``, ``SUB``). It fetches two inputs, applies the C operator (``+``, ``-``), and writes the result.
*   **``ICMP(op)``**: The Comparison variant. It performs the check and writes a boolean result (0 or 1) into the target register.
*   **``EXT(tp)``**: The Type Conversion variant. It handles sign-extension and zero-extension, casting the raw bits to the requested type (``int8_t``, ``uint16_t``, etc.) before promoting them back to the 64-bit virtual register format.

4.2 The Physics of Overflow (``MIR_ADDO``, etc.)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Handling integer overflow in C is notoriously difficult. MIR's interpreter handles this with explicit checks.

*   **The Internal Flags**: When an overflow-aware instruction (like ``ADDO``) is executed, it calculates the result and then performs a bitwise sanity check.
*   **The Result**: It sets internal flags (``signed_overflow_p``, ``unsigned_overflow_p``) which are then consumed by the branch instructions in the next "act." This ensures the virtual machine's arithmetic matches the expected behavior of real silicon.

4.3 The Memory Bridge (``LD``, ``ST``)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

The ``LD`` (Load) and ``ST`` (Store) macros are the only parts of the simulator that touch the host's real memory.

*   **Address Calculation**: They fetch the address from a virtual register.
*   **The Cast**: They cast that address to the correct C type (e.g., ``int32_t *``) and perform the dereference.
*   **Portability**: Because it uses standard C types, the interpreter automatically handles the host's endianness and alignment rules.
        
Summary: The use of these macros ensures that the interpreter is both **fast** (by minimizing code paths) and **extensible** (by making it easy to add new instructions).

5. Crossing the Divide: FFI and Function Calls
----------------------------------------------

A virtual world is no use if it can't talk to the real one. The interpreter includes a sophisticated **FFI (Foreign Function Interface)** bridge to call native C functions and handle incoming calls from the host.

5.1 The Virtual Stack: ``bp``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

When a MIR function is called in the interpreter, it creates a new **Stack Frame**.

*   **The Base Pointer (``bp``)**: This is the anchor for the function's local registers. It is allocated using ``alloca`` on the host's physical stack, but it behaves like a virtual register file.
*   **The Layout**: The ``bp`` array is sized to hold all the registers (``nregs``) defined in the function's descriptor.
*   **Safety Zone**: A few extra slots are reserved at the beginning of the frame for internal use, such as storing variadic argument pointers or handling ``setjmp``.

.. note::
   **Compiler Lore: The Magic of ``alloca``**
   
   Most VMs manage their own heap for stack frames. MIR is bolder: it uses the host's **physical stack** via ``alloca``. 
   
   This has a massive advantage: **Locality**. By keeping the virtual registers on the real stack, they are highly likely to stay in the CPU's **L1 Cache**. However, it means the interpreter's recursion depth is tied to the host's stack size. It's a high-stakes bet on performance over infinite recursion.

5.2 The FFI Bridge: ``get_ff_interface``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Calling a native C function (like ``printf``) from the interpreter requires a "Foreign Function" interface.

*   **The Problem**: The interpreter stores values in a unified ``MIR_val_t`` array, but C functions expect values in specific hardware registers or stack locations according to the host's ABI.
*   **The Solution**: ``get_ff_interface`` uses a target-specific assembly shim (``_MIR_get_ff_call``) to perform the transformation. It takes the interpreter's values, packs them into the correct CPU registers, and jumps to the native code.
*   **The Cache**: Because creating these shims is expensive, the interpreter maintains an ``ff_interface_tab`` (a hash table) to reuse them for identical function signatures.

.. note::
   **Interpreter Lore: The Tower of ABI**
   
   There is no such thing as "The C Language" at the machine level. There are only **ABIs** (Application Binary Interfaces). 
   
   An ``int`` might be passed in ``RDI`` on Linux but on the stack on another system. Floating point values might use the same registers as integers or a completely separate set. The FFI bridge is MIR's **Universal Translator**. It must know the "Bureaucracy" of every supported system to ensure that a virtual MIR value arrives safely in the hands of a native C function.

5.3 The ``setjmp`` Special Case
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

In the land of C, ``setjmp`` and ``longjmp`` are the stuff of nightmares. They perform non-local jumps by snapshotting and restoring the CPU state.

*   **The Hack**: The interpreter explicitly recognizes calls to ``setjmp_addr``. When encountered, it snapshots the virtual program counter (PC) into the stack frame before calling the real ``setjmp``.
*   **The Payoff**: This allows MIR programs to use standard C error-handling patterns without the interpreter losing track of its virtual execution state.

5.4 Variadic Power (``va_list``)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

The interpreter provides full support for C-style variadic functions.

*   **``MIR_VA_START``**: This opcode initializes a virtual variadic argument list. It essentially "captures" the arguments passed to the current virtual frame.
*   **Builtin Helpers**: Complex operations like ``va_arg`` and ``va_copy`` are handled by calling out to native "builtin" helpers, ensuring that the interpreter's variadic logic perfectly matches the host C compiler's behavior.

Summary: The interpreter isn't just a toy; it is a fully capable execution engine that can interoperate with the most complex parts of the C ecosystem. It is the reliable companion to the JIT generator, always ready to step in when the path to native code is blocked.

6. The Cloaking Device: Interpreter Shims
-----------------------------------------

The final piece of the interpreter puzzle is how it integrates into the host process so that it can be called like a normal C function.

*   **The Problem**: A native C caller expects to jump to a machine code address. It doesn't know about the interpreter's virtual stack or its internal ICode format.
*   **The Shim (``_MIR_get_interp_shim``)**: This is the **Cloaking Device**. When ``MIR_set_interp_interface`` is called, it generates a tiny piece of machine code (the shim).
*   **The Handover**: When the C caller jumps to the shim, the shim:
    1.  Captures the physical CPU registers into a ``va_list``.
    2.  Calls the interpreter's entry point (``interp``).
    3.  The interpreter extracts the arguments from the ``va_list``, runs the code, and puts the result back into the physical registers.
*   **The Result**: To the rest of the application, the interpreted function is indistinguishable from a compiled one. It has a real address and obeys the standard ABI.

7. The Physical Limits: ``MIR_ALLOCA`` and Stack Management
-----------------------------------------------------------

One of the most delicate areas of the interpreter's simulation is the implementation of ``MIR_ALLOCA`` (dynamic stack allocation).

7.1 The Host's Shadow: Using C ``alloca``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

The interpreter implements ``MIR_ALLOCA`` by calling the host's native C ``alloca`` function.

*   **The Direct Link**: When the virtual program asks for 1KB of stack space, the interpreter's ``eval`` loop calls ``alloca(1024)`` on the **physical host stack**.
*   **The Benefit**: This is extremely fast and ensures that the allocated memory is close to the interpreter's own state (the ``bp`` array), maintaining high cache locality.

7.2 The Loop Trap: Memory Persistence
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Because the interpreter executes an entire MIR function within a single call to the ``eval`` loop, ``MIR_ALLOCA`` memory has **Function Scope**, not block scope.

*   **The Danger**: If an interpreted program calls ``MIR_ALLOCA`` inside a loop (e.g., a ``while`` loop that runs 10,000 times), the physical stack will grow 10,000 times. The memory is **not reclaimed** until the interpreted function returns.
*   **The Limit**: This can lead to a host-level **Stack Overflow** even if the virtual program appears logically sound. Users should avoid calling ``alloca`` inside loops in interpreted code.

7.3 Block-Level Recovery: ``MIR_BSTART`` and ``MIR_BEND``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

To mitigate these limits, MIR supports explicit stack markers:

*   **``MIR_BSTART``**: Snapshots the current stack pointer.
*   **``MIR_BEND``**: Restores the stack pointer to a previous snapshot.
*   **The Implementation**: These opcodes use machine-dependent built-ins (like ``_MIR_get_bend_builtin``) to manually manipulate the CPU's Stack Pointer (e.g., ``RSP`` on x86). This is a rare case where the "Simulator" breaks the fourth wall and directly rearranges the host's own silicon state to reclaim memory.

.. note::
   **Historical Lore: The Dangerous Beauty of ``alloca``**
   
   The ``alloca`` function has been a point of contention since the 1970s. It was not part of the original C standard (ANSI C89) because it is inherently un-portable—it requires direct knowledge of how the stack works. However, it is so useful for performance that almost every compiler (including GCC and Clang) implements it as a built-in. MIR continues this tradition of "Dangerous Beauty" by leveraging ``alloca`` for its speed while warning the alchemist of its hidden thorns.

Conclusion: The Simulator's Triumph
-----------------------------------

We have completed our tour of the **Simulator**.

From the dense arrays of ICode to the recursive-descent logic of the FFI bridge, we have seen how MIR can execute code anywhere, anytime. The interpreter is the bedrock of MIR's portability—the guarantee that no matter how strange the CPU or how strict the OS, the code will run.

*   **Safety**: Runs in a virtual sandbox with its own stack.
*   **Speed**: Optimized with direct threaded dispatch and macro-generated handlers.
*   **Seamless**: Bridges the gap between virtual IR and native C code via shims.

**Next Steps**:
Our journey through the core implementation is complete. You have seen the mind (``mir.c``), the muscles (``mir-gen.c``), and the simulated soul (``mir-interp.c``) of the machine.

Congratulations on becoming a master of the MIR architecture!


        


