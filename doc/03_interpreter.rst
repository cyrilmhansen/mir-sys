The Interpreter
===============

Before the JIT kicks in, MIR can interpret code directly. This allows for immediate execution while compilation happens in the background.

Interpreter Interface
---------------------

The interpreter entry point is::

   void MIR_interp (MIR_context_t ctx,
                    MIR_item_t func_item,
                    MIR_val_t *results,
                    unsigned long nargs, ...);

It executes a compiled MIR function immediately using the current context, filling ``results`` with return values. The implementation lives in ``mir/mir-interp.c`` but the prototype is declared in ``mir/mir.h``.
