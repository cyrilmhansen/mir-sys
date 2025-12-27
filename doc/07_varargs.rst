The Varargs Nightmare
=====================

Variable arguments (`varargs`) represent one of the most complex and fragile aspects of the C ABI. Unlike standard function calls where arguments are passed in predictable registers or stack slots, `varargs` functions require the callee to dynamically "walk" a mixed list of register-passed and stack-passed arguments.

The Challenge
-------------
The complexity arises because modern calling conventions (System V AMD64, AAPCS64) optimize for speed by passing arguments in registers. However, `va_list` needs to access them as if they were a contiguous array in memory.

To bridge this gap, the **Register Save Area (RSA)** concept was invented. Upon entering a varargs function, the callee must "spill" all potential argument registers onto the stack so the `va_arg` logic can address them uniformly.

Target Breakdown
----------------

x86_64 (System V)
~~~~~~~~~~~~~~~~~
This is the most complex implementation.

- **Register Save Area**: 176 bytes.
    - 6 General Purpose Registers (RDI, RSI, RDX, RCX, R8, R9) $\times$ 8 bytes = 48 bytes.
    - 8 XMM Registers (XMM0-XMM7) $\times$ 16 bytes = 128 bytes.
- **va_list Structure**:
    - `gp_offset`: Offset into the GP save area.
    - `fp_offset`: Offset into the FP save area.
    - `overflow_arg_area`: Pointer to stack arguments (those that didn't fit in registers).
    - `reg_save_area`: Pointer to the RSA base.
- **Logic**: `va_arg` checks `gp_offset < 48` (for int). If true, it loads from `reg_save_area + gp_offset` and increments `gp_offset`. Otherwise, it loads from `overflow_arg_area` and increments that pointer.

AArch64 (AAPCS64)
~~~~~~~~~~~~~~~~~
Similar complexity, but distinct separation between integer and vector registers.

- **Register Save Area**: Variable size, up to ~300 bytes if all registers are saved.
- **va_list Structure**:
    - `__stack`: Pointer to stack arguments.
    - `__gr_top`: Top of the General Register save area.
    - `__vr_top`: Top of the Vector Register save area.
    - `__gr_offs`: Negative offset from `__gr_top` to the next available register.
    - `__vr_offs`: Negative offset from `__vr_top`.

RISC-V & PPC64
~~~~~~~~~~~~~~
These architectures often simplify the problem by forcing varargs onto the stack or using a simpler pointer-based `va_list`.

- **RISC-V**: `va_list` is simply a `void*` pointing to the next argument on the stack. The caller often handles the setup.

The MIR Solution
----------------
MIR abstracts these ABI nightmares into three target-agnostic instructions.

.. code-block:: none

   va_start va_list_ptr
   va_arg   val_ptr, va_list_ptr, type_mem
   va_end   va_list_ptr

1. VA_START
~~~~~~~~~~~
**Usage**: `va_start va`

- **JIT Implementation**:
    - Allocates the **Register Save Area** on the stack frame (if the target requires it).
    - Emits instructions to **store** all incoming argument registers (RDI..R9, XMM0..7 on x64) into this area.
    - Initializes the `va_list` struct (setting offsets to 0, pointers to stack top).
- **Complexity**: $O(1)$, but expensive. It involves up to ~20 register stores and struct initialization.

2. VA_ARG
~~~~~~~~~~
**Usage**: `va_arg val, va, type`

- **JIT Implementation**:
    - Generates complex branching logic **inline**.
    - **Branch 1**: Check if the argument fits in the register offset (e.g., `gp_offset < 48`).
    - **Path A (Register)**: Compute address `reg_save_area + offset`, load value, increment offset.
    - **Path B (Stack)**: Load from `overflow_arg_area`, increment pointer.
    - This logic is polymorphic: the generated code differs vastly if `type` is integer vs float.
- **Complexity**: $O(1)$, but high constant factor due to branches and potential cache misses.

3. VA_END
~~~~~~~~~~
**Usage**: `va_end va`

- **JIT Implementation**: Usually a no-op in MIR, as stack cleanup is handled by the function epilogue.
- **Complexity**: $O(1)$.

Implementation Files
--------------------
- **Generic Definitions**: `mir/mir.h` (`MIR_VA_START` enum).
- **Target Specifics**:
    - `mir/mir-gen-x86_64.c`: Defines `reg_save_area_size = 176` and generates the conditional move/branch logic.
    - `mir/mir-gen-aarch64.c`: Handles the `gr_top`/`vr_top` logic.
    - `mir/mir-gen-riscv64.c`: Simpler stack-based implementation.
- **Interpreter Shim**: `mir/mir-interp.c` uses `va_start_interp_builtin` to bridge host `va_list` to MIR's internal VM stack.
