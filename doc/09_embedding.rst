Embedding MIR
=============

MIR is designed from the ground up to be embedded into other applications, from high-level language runtimes (Ruby, Lua) to operating system kernels. This chapter covers the essential mechanics of integrating MIR: memory management, context isolation, and serialization.

Memory Management
-----------------
MIR never calls `malloc`, `realloc`, or `free` directly. Instead, it relies on a user-provided allocator interface, making it suitable for restricted environments (kernels, embedded systems) or specialized memory strategies (arenas, pools).

Initialization
~~~~~~~~~~~~~~
To start MIR, you must create a context using `MIR_init` (or `MIR_init2`), providing two allocators:

1.  **Data Allocator** (`MIR_alloc_t`): Used for internal structures (IR nodes, lists, hash tables).
2.  **Code Allocator** (`MIR_code_alloc_t`): Used by the JIT to allocate executable memory (needs `PROT_EXEC` on POSIX).

.. code-block:: c

   #include "mir.h"
   #include "mir-gen.h"

   // Example: Using standard malloc/free
   void *my_malloc (size_t size, void *user_data) { return malloc(size); }
   // ... define realloc and free ...

   int main() {
       MIR_alloc_t alloc = { .malloc = my_malloc, ... };
       MIR_context_t ctx = MIR_init(); // Uses default wrappers around malloc
       // ...
       MIR_finish(ctx);
       return 0;
   }

Context Isolation
-----------------
All MIR state is encapsulated within the `MIR_context_t` opaque handle. This design guarantees:

-   **Thread Safety**: Multiple threads can run independent MIR contexts simultaneously without locking.
-   **Resource cleanup**: `MIR_finish(ctx)` frees all memory associated with that context.

Serialization (Binary IO)
-------------------------
For rapid startup, MIR modules can be serialized to a compact binary format. This allows you to compile standard libraries once and load them quickly, or cache JIT input.

Standard I/O
~~~~~~~~~~~~
The simplest API reads/writes to a C `FILE*`.

.. code-block:: c

   // Save to file
   FILE *f = fopen("lib.mirb", "wb");
   MIR_write(ctx, f);
   fclose(f);

   // Load from file
   f = fopen("lib.mirb", "rb");
   MIR_read(ctx, f);
   fclose(f);

Custom Streams
~~~~~~~~~~~~~~
For advanced usage (e.g., loading from a memory buffer, network socket, or compressed archive), MIR provides function-pointer based APIs:

-   `MIR_write_with_func`
-   `MIR_read_with_func`

.. code-block:: c

   // Example: Reading from a memory buffer
   int byte_reader(MIR_context_t ctx) {
       // return next byte from buffer or EOF
   }

   MIR_read_with_func(ctx, byte_reader);

Complexity Considerations
-------------------------
-   **Initialization**: $O(1)$.
-   **Binary I/O**:
    -   **Time**: $O(N)$ where $N$ is the size of the module. Binary loading is typically 10-100x faster than parsing textual MIR.
    -   **Memory**: Stream-based. Only the resulting internal structures are allocated; no massive intermediate buffers are required.

API Reference
-------------
.. doxygenfunction:: MIR_init
   :project: MIR

.. doxygenfunction:: MIR_read
   :project: MIR

.. doxygenfunction:: MIR_write
   :project: MIR
