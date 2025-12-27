Missing Links & Future Work
===========================

Concurrency
-----------
MIR currently lacks atomic primitives, making it difficult to implement lock-free data structures directly in MIR.

Endianness
----------
MIR assumes the endianness of the host machine.

Exception Handling
------------------
There is currently no support for DWARF CFI generation for stack unwinding.
