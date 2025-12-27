The JIT Pipeline
================

Compilation involves several passes: simplification, SSA construction, register allocation, and machine code generation.

Generator Context
-----------------
The state of the generation process.

Internally the generator uses a ``gen_ctx`` struct (``mir/mir-gen.c``) to carry allocator hooks, machine description tables, CFG, SSA state, and scratch buffers during lowering. It is intentionally opaque to users; the public API exposes initialization via ``MIR_gen_init`` and clean-up via ``MIR_gen_finish``.
