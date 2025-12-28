C2MIR: The C Frontend
======================

Welcome to the **Alchemist's Laboratory**.

MIR includes a complete, embeddable C11 compiler known as **C2MIR** (or ``c2m``). This component allows users to transmute raw C source code directly into MIR modules at runtime, bypassing external compilers like GCC or Clang. It is a four-pass compiler living entirely within ``mir/c2mir/c2mir.c``.

1. The Alchemist's Pipeline
---------------------------

The compilation process is a tightly integrated sequence of transformations:

.. image:: _static/diagrams/c2mir_pipeline.*
   :alt: C2MIR Pipeline
   :align: center

1.  **Preprocessor Pass**: Generates a stream of tokens from source files.
2.  **Parsing Pass**: Builds an **AST (Abstract Syntax Tree)** using recursive descent.
3.  **Context Pass**: Performs semantic analysis, checking constraints and adding type information to the AST.
4.  **Generation Pass**: Walks the typed AST to emit MIR instructions.

2. The Preprocessor: Token Stream Sorcery
-----------------------------------------

Unlike traditional compilers that pipeline text (``cpp | cc1``), C2MIR's preprocessor is integrated directly into the lexer. It works on a **stream of tokens**, not a stream of characters.

- **Implementation**: Functions starting with ``pre_`` in ``mir/c2mir/c2mir.c``.
- **Macro Expansion**:
    - Macros are stored in a high-speed hash table (``macro_tab``).
    - When a macro is triggered, the lexer transparently switches its input source to a temporary buffer containing the macro's expansion.
- **Key Data Structures**:
    - ``struct macro``: Stores parameters and the sequence of replacement tokens.
    - ``struct macro_call``: Manages the context of nested expansions (allowing macros to call macros).

.. note::
   **Historical Lore: The Recursion Paradox**
   
   C macros are notoriously dangerous because they can be self-referential. To prevent the compiler from falling into an infinite loop, the C standard requires that a macro name is "painted blue" (made unavailable for expansion) while it is currently being expanded. C2MIR implements this "blue painting" logic within the ``macro_call`` stack, ensuring that even the most devious macro tricks won't crash the compiler.

3. The Parser: Navigating the C11 Labyrinth
-------------------------------------------

The parser implements the C11 grammar (minus a few optional features like atomics). It is a **Manually Written Recursive Descent Parser** with **Speculative Backtracking**.

- **Backtracking (``TRY``)**: C's grammar is ambiguous. Is ``(A)*B`` a multiplication or a cast? C2MIR solves this with the ``TRY(func)`` macro. It snapshots the current token position and speculatively attempts to parse a rule. If it hits a dead end, it rewinds the clock and tries a different path.
- **The Lexer Hack**: To distinguish between identifiers and typedef names, C2MIR maintains a ``symbol_tab`` that tracks the current scope. The parser feeds information back to the lexer so it knows when an ID has become a **Type**.

.. _c2m_declarators:

4. Arcane Arts: Complex C Types and Bitfields
----------------------------------------------

C type declarations are famously counter-intuitive. ``int *(*f())[5]`` is a function returning a pointer to an array of pointers to integers.

- **The Solution**: C2MIR uses a recursive ``type`` structure that mirrors the nested nature of C declarations.
- **Bitfield Layout**: Implementing bitfields requires deep knowledge of the host's **Endianness**.
    - **Little Endian**: Bits are typically packed from the least significant bit (LSB) up.
    - **Big Endian**: Bits are packed from the most significant bit (MSB) down.
    - C2MIR's ``set_type_layout`` function handles this "Bit Squeeze" by consulting the target-specific headers (e.g., ``cx86_64.h``).

5. Computational Complexity
---------------------------

- **Time Complexity**:
    - **Average Case**: :math:`O(N)` where :math:`N` is the number of tokens.
    - **Worst Case**: The backtracking mechanism (``TRY``) theoretically allows for exponential time on pathological grammar edges, but standard C11 code is designed to be parsed in linear time.
- **Memory Complexity**:
    - **AST**: :math:`O(N)` nodes. C2MIR uses an arena-style allocator (``reg_malloc``) to keep all nodes for a single translation unit in a contiguous block, which is then freed in one go.
    - **Symbol Tables**: :math:`O(S)` where :math:`S` is the number of unique strings and identifiers.

Summary: C2MIR is a high-performance, embeddable gateway to the MIR multiverse. It provides the convenience of C with the speed of JIT, all without leaving the comfort of your own process memory.