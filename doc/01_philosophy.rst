The Philosophy & Context
========================

The Ideal C Virtual Machine
---------------------------
MIR acts as a bridge between the abstract C language and concrete hardware.
Unlike LLVM IR, which is vast and complex, MIR aims to be the "Medium" IR—just enough abstraction to be portable, but close enough to metal to be fast.

The Context
-----------
The state of the world is held in the context structure.

.. doxygenstruct:: MIR_context
   :project: MIR
   :members:
