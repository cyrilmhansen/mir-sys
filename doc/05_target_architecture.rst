The 64-bit Targets
==================

MIR targets x86_64, ARM64, RISC-V 64, and others.

Stack Layouts
-------------
Each architecture defines its own stack frame layout. 
For example, on x86_64, the register save area size differs between Windows and Linux.

.. doxygenvariable:: reg_save_area_size
   :project: MIR

.. note::
   The variable above is extracted directly from the source code. If you see it multiple times, it is because it is defined statically in multiple mir-gen-*.c files. Doxygen aggregates them here.
