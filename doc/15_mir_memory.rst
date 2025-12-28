MIR Memory Operand Reference (``MIR_mem_t``)
=========================================

The ``MIR_mem_t`` structure is the fundamental building block for describing **memory access** in the MIR Intermediate Representation. It encapsulates the information required to calculate an Effective Address (EA) and defines how data should be interpreted (width and signedness) when loaded from or stored to that address.

It allows MIR to represent complex addressing modes natively, similar to x86/x64 "SIB" (Scale-Index-Base) addressing, while remaining compatible with RISC architectures (which may only support subsets like Base+Displacement) through internal legalization passes.

1. The Addressing Formula
-------------------------

The core responsibility of ``MIR_mem_t`` is to define a target memory address using the following linear equation:

.. math:: 

    \text{Address} = \text{Base} + (\text{Index} \times \text{Scale}) + \text{Displacement}

Where:

*   **Base**: A register holding the starting pointer.
*   **Index**: A register holding a dynamic offset (e.g., a loop counter).
*   **Scale**: A literal multiplier applied to the Index (typically 1, 2, 4, or 8).
*   **Displacement**: A constant literal offset (positive or negative).

2. Structure Field Analysis
---------------------------

2.1. ``type`` (``MIR_type_t : 8``)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Defines the **width** and **interpretation** of the data at the memory location.

*   **Storage Size**: Determines how many bytes are read/written (e.g., ``MIR_T_I8`` = 1 byte, ``MIR_T_D`` = 8 bytes).
*   **Extension Semantics**: When loading a value smaller than the native register size (64-bit), this type dictates extension:
    *   **Signed (``MIR_T_I8``, ``MIR_T_I16``, ``MIR_T_I32``)**: The value is **sign-extended** to 64 bits.
    *   **Unsigned (``MIR_T_U8``, ``MIR_T_U16``, ``MIR_T_U32``)**: The value is **zero-extended** to 64 bits.
*   **Floating Point**: Types ``MIR_T_F``, ``MIR_T_D``, ``MIR_T_LD`` indicate the data is transferred to/from floating-point registers.

2.2. ``scale`` (``MIR_scale_t``)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

The multiplier applied to the ``index`` register.

*   **Type**: ``uint8_t``.
*   **Typical Values**: 1, 2, 4, 8.
*   **Usage**: Used primarily for array access where the stride corresponds to the size of the array element (e.g., Scale 4 for an array of 32-bit integers).

2.3. ``base`` (``MIR_reg_t``)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

The primary register holding the base address.

*   **Value**: A MIR register number.
*   **Absence**: If ``0`` (or ``MIR_NON_VAR`` in internal contexts), it implies the base is effectively zero (absolute addressing).

2.4. ``index`` (``MIR_reg_t``)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

The secondary register used for dynamic offsetting.

*   **Value**: A MIR register number.
*   **Absence**: If ``0`` (or ``MIR_NON_VAR``), the index component is ignored (treated as 0).

2.5. ``disp`` (``MIR_disp_t``)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

The constant displacement (offset) added to the address.

*   **Type**: ``int64_t`` (Signed).
*   **Usage**: Used for accessing structure members (offset from struct start) or stack variables (offset from stack pointer).

2.6. ``alias`` and ``nonalias`` (``MIR_alias_t``)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

These fields provide **Pointer Aliasing Hints** to the optimizer (specifically for Global Value Numbering and redundancy elimination).

*   **``alias``**:
    *   Value ``0``: May alias *any* memory.
    *   Value ``>0``: Memory operands sharing the *same* alias ID are considered aliased.
*   **``nonalias``**:
    *   Value ``0``: Ignored.
    *   Value ``>0``: Memory operands sharing the *same* nonalias ID are strictly **not** aliased (similar to C99 ``restrict``).

2.7. ``nloc`` (``uint32_t``)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~

*   **Usage**: **Internal Use Only**.
*   **Purpose**: Used by the generator and optimizer to track memory locations. Memory operands with the same nonzero ``nloc`` are guaranteed to refer to the exact same memory location.

3. Usage Examples
-----------------

Scenario A: Simple Pointer Dereference
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**C Code:**

.. code-block:: c

    int64_t x = *ptr; // Assuming ptr is in register r1

**MIR Representation:**

*   ``type``: ``MIR_T_I64``
*   ``base``: ``r1``
*   ``index``: ``0``
*   ``disp``: ``0``
*   **MIR Text**: ``i64:0(r1)``

Scenario B: Structure Member Access
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**C Code:**

.. code-block:: c

    struct Point { int x; int y; }; // y is at offset 4
    int y = p->y; // Assuming p is in r1

**MIR Representation:**

*   ``type``: ``MIR_T_I32`` (Loaded value extends to 64-bit signed)
*   ``base``: ``r1``
*   ``disp``: ``4``
*   **MIR Text**: ``i32:4(r1)``

Scenario C: Array Access (Looping)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**C Code:**

.. code-block:: c

    // int array[] at r1, index i at r2
    int x = array[i]; // sizeof(int) == 4

**MIR Representation:**

*   ``type``: ``MIR_T_I32``
*   ``base``: ``r1``
*   ``index``: ``r2``
*   ``scale``: ``4``
*   **MIR Text**: ``i32:(r1, r2, 4)``

Scenario D: Complex Addressing
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**C Code:**

.. code-block:: c

    // struct { int a; int b[10]; } *s;
    // access s->b[i]
    // s is in r1, i is in r2. 'b' is at offset 4. int size is 4.
    int val = s->b[i];

**MIR Representation:**

*   ``type``: ``MIR_T_I32``
*   ``base``: ``r1``
*   ``index``: ``r2``
*   ``scale``: ``4``
*   ``disp``: ``4``
*   **Equation**: ``Address = r1 + (r2 * 4) + 4``
*   **MIR Text**: ``i32:4(r1, r2, 4)``

4. Internal Mechanics: ``MIR_OP_MEM`` vs ``MIR_OP_VAR_MEM``
-----------------------------------------------------------

While ``MIR_mem_t`` is the struct definition, it is used in two different ``MIR_op_mode_t`` contexts within the compiler:

1.  **``MIR_OP_MEM``**: Standard memory operand. Registers ``base`` and ``index`` refer to **named registers** (created via ``MIR_new_func_reg``). This is what users typically create via ``MIR_new_mem_op``.
2.  **``MIR_OP_VAR_MEM``**: Internal variable memory operand. Used during the code generation phase. Here, ``base`` and ``index`` refer to **internal pseudo-registers** or **hard registers** (physical CPU registers). The semantics of ``0`` vs ``MIR_NON_VAR`` (defined as ``UINT32_MAX``) become important here to distinguish between "register 0" and "no register".

5. Aliasing Strategy
--------------------

The optimizer uses ``alias`` and ``nonalias`` to reorder instructions safely.

*   If you have a store to ``alias:1`` and a load from ``alias:2``, the compiler assumes they do not overlap and may move the load before the store to hide memory latency.
*   If you have a store to ``nonalias:5`` and a load from ``nonalias:5``, the compiler assumes they represent distinct objects (like arrays passed with ``restrict``) and can optimize aggressively.
