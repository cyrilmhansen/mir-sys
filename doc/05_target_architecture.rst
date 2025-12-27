The 64-bit Targets
==================

MIR targets x86_64, ARM64, RISC-V 64, and others.

Stack Layouts
-------------
Each architecture defines its own stack frame layout. 
For example, on x86_64, the register save area size differs between Windows and Linux.

The backend keeps per-target constants such as ``reg_save_area_size`` in the corresponding ``mir-gen-*.c`` files to size stack frames and spill slots. On System V x86_64 the save area is 176 bytes; on Windows it is 192 bytes because of shadow space rules.
