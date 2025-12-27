C2MIR: The C Frontend
=====================

MIR includes a complete, embeddable C11 compiler known as **C2MIR** (or `c2m`). This component allows users to compile C source code directly into MIR modules at runtime, bypassing external compilers like GCC or Clang. It resides in `mir/c2mir/`.

Architecture
------------
C2MIR is a single-pass (mostly) compiler consisting of three tightly integrated stages:

1.  **Preprocessor**: Handles macro expansion, file inclusion, and conditional compilation.
2.  **Parser**: A recursive-descent parser that builds a simplified AST (Abstract Syntax Tree).
3.  **Generator**: Walks the AST to emit MIR instructions.

The Preprocessor
----------------
Unlike traditional compilers that pipeline text (`cpp | cc1`), C2MIR's preprocessor works on a stream of **tokens**.

- **Implementation**: `mir/c2mir/c2mir.c` (functions starting with `pre_`)
- **Macro Expansion**:
    - Macros are stored in a hash table (`macro_tab`).
    - When a macro is encountered, it is expanded into a buffer of tokens (`macro_call_t`).
    - The lexer transparently reads from this expansion buffer before returning to the main file stream.
- **Key Data Structures**:
    - `struct macro`: Stores the definition (parameters and replacement tokens).
    - `struct macro_call`: Represents an active expansion context (stackable for nested macros).

The Parser
----------
The parser implements the C11 grammar (with some GNU extensions). It uses a **Recursive Descent** strategy with backtracking capabilities.

- **Implementation**: `mir/c2mir/c2mir.c`
- **Backtracking**:
    - C syntax is ambiguous (e.g., `(A)*B` could be a cast or a multiplication depending on whether `A` is a type).
    - C2MIR handles this via the `TRY(func)` macro. It records the current token position, attempts to parse a non-terminal, and restores the position if parsing fails.
- **Typedef Handling**:
    - To distinguish types from variables (the "lexer hack" problem), C2MIR maintains a `tpname_tab` (hash table) tracking currently visible typedef names.
- **Key Macros**:
    - `D(name)`: Defines a parsing function.
    - `P(func)`: Calls a parsing function, returning error on failure.
    - `TRY(func)`: Speculatively attempts to parse.

Code Generation
---------------
Once an AST node (e.g., a function body or expression) is fully parsed, the generator emits the corresponding MIR.

- **Control Flow**: C loops (`for`, `while`) and `if` statements are lowered into MIR branches (`MIR_BT`, `MIR_JMP`) and labels.
- **Expressions**: The complex C operator precedence is handled by the parser structure; the generator simply emits instructions for the resulting AST nodes (e.g., `MIR_ADD`, `MIR_MUL`).
- ** ABI Details**: Platform-specific headers (`mirc_*.h`) define types like `va_list` and `long` size to match the host ABI.

Complexity Analysis
-------------------

- **Time**:
    - **General Case**: $O(N)$ where $N$ is the number of tokens.
    - **Worst Case**: The backtracking mechanism (`TRY`) theoretically allows for super-linear time in pathological cases, but standard C code parses linearly.
    - **Preprocessing**: $O(N)$ (linear scan and expansion).
- **Memory**:
    - **AST**: $O(N)$ (proportional to the complexity of the function body).
    - **Symbol Tables**: $O(S)$ where $S$ is the number of unique identifiers and macros.


