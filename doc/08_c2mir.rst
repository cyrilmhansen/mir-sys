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
   :width: 100%

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

3.1 The Grammar Macros
~~~~~~~~~~~~~~~~~~~~~~

To keep the code readable and close to the C standard, C2MIR uses a domain-specific language built on C macros:

- **``D(name)``**: Defines a parsing function (a "non-terminal").
- **``P(func)``**: Calls a parsing function and returns an error node if it fails.
- **``TRY(func)``**: The **Speculative Scout**. It attempts to parse a rule but allows for failure without halting the entire process.
- **``PT(token)``**: Matches a specific token type (e.g., ``PT('(')``).

3.2 Speculative Backtracking
~~~~~~~~~~~~~~~~~~~~~~~~~~~~

C's grammar is famously ambiguous. For example, ``(A)*B`` could be:
1.  A cast of ``*B`` to type ``A``.
2.  A multiplication of variable ``A`` and ``B``.

C2MIR resolves this by using ``record_start()`` and ``record_stop()``. When ``TRY(f)`` is called, the parser "bookmarks" its current position in the token stream. If the speculative path fails, it "rewinds" the stream to the bookmark and tries the next alternative.

.. _c2m_lexer_hack:

3.3 The Lexer Hack and Symbol Sovereignty
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

One of the oldest riddles in C compiler design is the **Lexer Hack**. Because C allows users to define new type names (via ``typedef``), the lexer must know if an identifier is a variable or a type to provide the correct token to the parser.

- **The Shared Knowledge**: C2MIR's parser and lexer share a ``symbol_tab`` (a high-speed hash table).
- **Scope Management**: As the parser enters a new block (``{ ... }``), it creates a new **Scope**. When a typedef is encountered, it is added to the current scope.
- **The Query**: When the lexer sees ``MyType``, it checks the ``symbol_tab``. If ``MyType`` is registered as a ``S_REGULAR`` symbol with a typedef attribute, it returns a type-specific token; otherwise, it returns a generic ``T_ID``.

.. _c2m_context_pass:

4. The Context Pass: Semantic Enlightenment
--------------------------------------------------------------------------------

Once the AST is built, it is a "skeleton" of the program. The **Context Pass** (the ``check`` function) provides the "soul" by adding type information and verifying C's strict rules.

- **Type Resolution**: Every expression node is annotated with a ``struct type``.
- **Constant Folding**: If the alchemist sees ``2 + 2``, the context pass replaces the addition node with a single constant node ``4``.
- **L-Value Analysis**: It determines if an expression can be assigned to (e.g., ``x = 10`` is valid, but ``5 = 10`` is not).

5. The Final Alchemy: MIR Generation
------------------------------------

The last step is the most rewarding: turning the typed AST into executable MIR instructions. The ``gen_mir`` function walks the tree and emits code.

- **Expression Lowering**: Nested AST expressions are flattened into a sequence of MIR instructions using temporary registers.
- **Control Flow**: C's high-level constructs (``if``, ``while``, ``for``) are lowered into low-level MIR labels and jumps.
- **Built-in Support**: C2MIR leverages the MIR core's built-in functions for complex operations like ``memcpy`` or ``long double`` arithmetic.

6. Computational Complexity
---------------------------

- **Time Complexity**:
    - **Average Case**: :math:`O(N)` where :math:`N` is the number of tokens.
    - **Worst Case**: The backtracking mechanism (``TRY``) theoretically allows for exponential time on pathological grammar edges, but standard C11 code is designed to be parsed in linear time.
- **Memory Complexity**:
    - **AST**: :math:`O(N)` nodes. C2MIR uses an arena-style allocator (``reg_malloc``) to keep all nodes for a single translation unit in a contiguous block, which is then freed in one go.
    - **Symbol Tables**: :math:`O(S)` where :math:`S` is the number of unique strings and identifiers.

Summary: C2MIR is a high-performance, embeddable gateway to the MIR multiverse. It provides the convenience of C with the speed of JIT, all without leaving the comfort of your own process memory.
