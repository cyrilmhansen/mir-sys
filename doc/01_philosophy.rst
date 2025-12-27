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
