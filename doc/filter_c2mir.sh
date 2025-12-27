#!/bin/bash
# Replaces "D (name)" with "static node_t name (c2m_ctx_t c2m_ctx, int no_err_p)"
# Replaces "DA (name)" with "static node_t name (c2m_ctx_t c2m_ctx, int no_err_p, node_t arg)"
sed -E 's/^D \((.*)\)/static node_t \1 (c2m_ctx_t c2m_ctx, int no_err_p)/g' $1 | sed -E 's/^DA \((.*)\)/static node_t \1 (c2m_ctx_t c2m_ctx, int no_err_p, node_t arg)/g'
