The Metal: 64-bit Targets
=========================

The promise of MIR is "Write Once, Run Everywhere" (at near-native speed). To achieve this, the JIT backend (`mir-gen`) implements distinct lowerings for each major 64-bit architecture.

x86_64 (The CISC Giant)
-----------------------
The ubiquitous Intel/AMD architecture.
- **Characteristics**: Variable length instructions (1-15 bytes), Complex addressing modes, Two-operand destructive arithmetic (`ADD RAX, RBX` -> `RAX += RBX`).
- **Registers**: 16 General Purpose Registers (RAX, RBX, RCX, RDX, RSI, RDI, RBP, RSP, R8-R15).
- **MIR Strategy**: MIR's 3-operand instructions (`ADD a, b, c`) are often lowered to a `MOV` followed by the destructive x86 instruction.

AArch64 (The RISC Dominator)
----------------------------
The architecture powering modern mobile devices and Apple Silicon.
- **Characteristics**: Fixed 32-bit instruction width, Load/Store architecture.
- **Registers**: 31 General Purpose Registers (X0-X30). X30 is the Link Register (LR).
- **MIR Strategy**: Maps naturally to MIR's 3-operand structure.

RISC-V (The Open Standard)
--------------------------
The rising star of open hardware.
- **Characteristics**: Modular, extremely simple base ISA.
- **Registers**: 32 General Purpose Registers (x0-x31). `x0` is always hardwired to zero.
- **MIR Strategy**: Similar to AArch64, but requires sequences of instructions for large constants or complex addresses due to limited immediate sizes.

The Abstraction Layer
---------------------
MIR bridges these differences by normalizing:
1.  **Calling Conventions**: It handles the complexities of System V AMD64 vs Windows x64 vs AAPCS64.
2.  **Stack Layout**: It automatically calculates spill slots and register save areas.

    - *x86_64 System V*: ~176 bytes save area.
    - *Windows x64*: Shadow space allocation (32 bytes).

Endianness
----------
MIR is currently **Little-Endian** biased, as are all the major supported 64-bit targets (x86_64, AArch64, RISC-V). Big-Endian support (s390x, PPC64) exists but is less commonly exercised in the wild.