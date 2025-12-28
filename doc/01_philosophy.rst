The Philosophy & Context
========================

The Ideal C Virtual Machine
---------------------------
MIR acts as a bridge between the abstract C language and concrete hardware.
Unlike LLVM IR, which is vast and complex, MIR aims to be the "Medium" IR—just enough abstraction to be portable, but close enough to metal to be fast.

The Context
-----------
The state of the world is held in the context structure.

``MIR_context_t`` (declared in ``mir/mir.h``) owns allocator hooks, module lists, and target-specific state. Each context is independent so multiple interpreters or JITs can run safely in the same process. A context is created with ``MIR_init`` and destroyed with ``MIR_finish``.

OS & Standard Library Requirements
----------------------------------
MIR is designed to be extremely lightweight and embeddable, with minimal dependencies on the host environment:

- **Standard Library**: MIR requires a basic C11-compliant standard library (``stdio.h``, ``stdint.h``, ``stdlib.h``, ``string.h``, and ``assert.h``).
- **Operating Systems**:
    - **Linux**: Primary development and testing platform.
    - **macOS**: Full support for both Intel (x86_64) and Apple Silicon (AArch64).
    - **Windows**: Support for 64-bit (x64) environments. MIR explicitly does **not** support 32-bit Windows.
- **Embedded Use**: For restricted environments, MIR can be compiled with ``MIR_NO_IO`` and ``MIR_NO_SCAN`` to remove dependencies on file I/O and text scanning, allowing it to run on "bare metal" or within custom kernels if an allocator is provided.
- **Memory**: MIR uses its own internal memory management abstractions (``mir-alloc.h``), which can be hooked by the user to use custom pool allocators or garbage collectors.
