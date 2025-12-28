The Uncharted Territories
=========================

While MIR is a powerful "Ideal C Machine", it is not yet a complete operating system abstraction. Several modern primitives are currently missing or require external implementation.

Concurrency & Threading
-----------------------
MIR is thread-safe in the sense that multiple `MIR_context_t` instances can coexist in different threads. However, the language itself **lacks threading primitives**:

- **No `spawn`**: MIR cannot launch threads natively.
- **No Atomics**: There are no `atomic_load`, `atomic_store`, or `CAS` (Compare-And-Swap) instructions.
- **No Mutexes**: Synchronization must be handled via FFI (Foreign Function Interface) calls to C functions (e.g., `pthread_mutex_lock`).

*Implication:* Implementing a multi-threaded garbage collector or scheduler in pure MIR is currently impossible without external C calls.

Endianness Independence
-----------------------
MIR does not enforce a specific endianness. It adopts the endianness of the host CPU.
- **Load/Store**: `MOV` from memory reads bytes in native order.
- **Cross-Compilation**: Generating code for a Big Endian target on a Little Endian host (or vice-versa) requires careful manual byte-swapping, which MIR does not automate.

Exception Handling
------------------
MIR supports basic control flow, but lacks high-level exception handling mechanisms:
- **No Unwind**: There is no generation of `.eh_frame` or DWARF CFI side-tables.
- **setjmp/longjmp**: Must be implemented via FFI.

Future Roadmap
--------------
1.  **Atomic Intrinsics**: Add `MIR_ATOMIC_*` opcodes.
2.  **Memory Model**: Define a C11-compatible memory model.
3.  **Debug Info**: Generate DWARF for JITed code to allow debugging with GDB/LLDB.