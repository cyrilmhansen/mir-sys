// Use a module to encapsulate bindings and suppress ALL warnings (clippy + rustc)
#[allow(warnings)]
mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub use bindings::*;

#[cfg(unix)]
pub mod code_alloc {
    use super::*;
    use libc::{
        c_int, c_void, mmap, mprotect, munmap, size_t, MAP_ANONYMOUS, MAP_FAILED, MAP_PRIVATE,
        PROT_EXEC, PROT_READ, PROT_WRITE,
    };
    use std::ptr;

    unsafe extern "C" fn mem_map(len: size_t, _user_data: *mut c_void) -> *mut c_void {
        let ptr = mmap(
            ptr::null_mut(),
            len,
            PROT_READ | PROT_WRITE,
            MAP_PRIVATE | MAP_ANONYMOUS,
            -1,
            0,
        );
        if ptr == MAP_FAILED { ptr::null_mut() } else { ptr }
    }

    unsafe extern "C" fn mem_unmap(ptr: *mut c_void, len: size_t, _user_data: *mut c_void) -> c_int {
        munmap(ptr, len)
    }

    unsafe extern "C" fn mem_protect(
        ptr: *mut c_void,
        len: size_t,
        prot: MIR_mem_protect_t,
        _user_data: *mut c_void,
    ) -> c_int {
        let native_prot = if prot == MIR_mem_protect_PROT_WRITE_EXEC {
            PROT_READ | PROT_WRITE
        } else if prot == MIR_mem_protect_PROT_READ_EXEC {
            PROT_READ | PROT_EXEC
        } else {
            return -1;
        };
        if mprotect(ptr, len, native_prot) != 0 { -1 } else { 0 }
    }

    pub fn unix_mmap() -> MIR_code_alloc {
        MIR_code_alloc {
            mem_map: Some(mem_map),
            mem_unmap: Some(mem_unmap),
            mem_protect: Some(mem_protect),
            user_data: ptr::null_mut(),
        }
    }
}

#[cfg(test)]
#[cfg(unix)]
mod tests {
    use super::*;
    use libc::{
        mmap, mprotect, munmap, size_t, MAP_ANONYMOUS, MAP_FAILED, MAP_PRIVATE, PROT_EXEC,
        PROT_READ, PROT_WRITE,
    };
    use std::ffi::{c_int, c_void, CString};
    use std::ptr;

    // --- Custom Allocator (Required for Android/Modern Linux) ---
    unsafe extern "C" fn test_mem_map(len: size_t, _user_data: *mut c_void) -> *mut c_void {
        let ptr = mmap(
            ptr::null_mut(),
            len,
            PROT_READ | PROT_WRITE,
            MAP_PRIVATE | MAP_ANONYMOUS,
            -1,
            0,
        );
        if ptr == MAP_FAILED {
            ptr::null_mut()
        } else {
            ptr
        }
    }

    unsafe extern "C" fn test_mem_unmap(
        ptr: *mut c_void,
        len: size_t,
        _user_data: *mut c_void,
    ) -> c_int {
        munmap(ptr, len)
    }

    unsafe extern "C" fn test_mem_protect(
        ptr: *mut c_void,
        len: size_t,
        prot: MIR_mem_protect_t,
        _user_data: *mut c_void,
    ) -> c_int {
        let mut native_prot = 0;
        if prot == MIR_mem_protect_PROT_WRITE_EXEC {
            native_prot = PROT_READ | PROT_WRITE;
        } else if prot == MIR_mem_protect_PROT_READ_EXEC {
            native_prot = PROT_READ | PROT_EXEC;
        }
        if mprotect(ptr, len, native_prot) != 0 {
            -1
        } else {
            0
        }
    }

    fn get_test_allocator() -> MIR_code_alloc {
        code_alloc::unix_mmap()
    }

    #[test]
    fn test_mir_jit_compile_add() {
        unsafe {
            let mut code_alloc = get_test_allocator();

            // Initialize
            let ctx = _MIR_init(ptr::null_mut(), &mut code_alloc);
            MIR_gen_init(ctx);
            MIR_gen_set_optimize_level(ctx, 0);

            let mod_name = CString::new("test_mod").unwrap();
            let _m = MIR_new_module(ctx, mod_name.as_ptr());

            let func_name = CString::new("add").unwrap();
            let arg_a = CString::new("a").unwrap();
            let arg_b = CString::new("b").unwrap();
            let res_reg_name = CString::new("res").unwrap();

            let mut type_i64 = MIR_type_t_MIR_T_I64;

            let mut args = vec![
                MIR_var_t {
                    type_: type_i64,
                    name: arg_a.as_ptr(),
                    size: 0,
                },
                MIR_var_t {
                    type_: type_i64,
                    name: arg_b.as_ptr(),
                    size: 0,
                },
            ];

            let func = MIR_new_func_arr(
                ctx,
                func_name.as_ptr(),
                1,
                &mut type_i64 as *mut _,
                2,
                args.as_mut_ptr(),
            );

            let reg_a = MIR_reg(ctx, arg_a.as_ptr(), (*func).u.func);
            let reg_b = MIR_reg(ctx, arg_b.as_ptr(), (*func).u.func);
            let reg_res = MIR_new_func_reg(ctx, (*func).u.func, type_i64, res_reg_name.as_ptr());

            // 1. Instruction: ADD
            let mut ops_add = vec![
                MIR_new_reg_op(ctx, reg_res),
                MIR_new_reg_op(ctx, reg_a),
                MIR_new_reg_op(ctx, reg_b),
            ];
            let insn_add = MIR_new_insn_arr(ctx, MIR_insn_code_t_MIR_ADD, 3, ops_add.as_mut_ptr());
            MIR_append_insn(ctx, func, insn_add);

            // 2. Instruction: RET
            // FIX: Use MIR_new_insn_arr instead of MIR_new_ret_insn to safely pass the operands array
            let mut ops_ret = vec![MIR_new_reg_op(ctx, reg_res)];
            let insn_ret = MIR_new_insn_arr(
                ctx,
                MIR_insn_code_t_MIR_RET, // Use RET opcode
                1,
                ops_ret.as_mut_ptr(),
            );
            MIR_append_insn(ctx, func, insn_ret);

            // Compile & Link
            MIR_finish_func(ctx);
            MIR_finish_module(ctx);
            MIR_load_module(ctx, _m);
            MIR_link(ctx, Some(MIR_set_gen_interface), None);

            // Run
            let fun_ptr = MIR_gen(ctx, func);
            assert!(!fun_ptr.is_null());

            let rust_func: extern "C" fn(i64, i64) -> i64 = std::mem::transmute(fun_ptr);
            let result = rust_func(10, 20);

            assert_eq!(result, 30);

            // Cleanup
            MIR_gen_finish(ctx);
            MIR_finish(ctx);
        }
    }

    #[test]
    fn test_mir_load_from_string_and_exec() {
        unsafe {
            // 1. Setup Context
            let mut code_alloc = get_test_allocator();
            let ctx = _MIR_init(ptr::null_mut(), &mut code_alloc);
            MIR_gen_init(ctx);
            MIR_gen_set_optimize_level(ctx, 0);

            // 2. Define MIR Code (simulating reading a .mir file)
            // This defines a module 'm_calc' with a function 'add_nums'
            let mir_source = CString::new(
                r#"
m_calc:   module
          export add_nums
add_nums: func i64, i64:a, i64:b
          local i64:r
          add r, a, b
          ret r
          endfunc
          endmodule
"#,
            )
            .unwrap();

            // 3. Parse the MIR string
            // This parses the string and appends the module to the context's module list
            MIR_scan_string(ctx, mir_source.as_ptr());

            // 4. Retrieve the Module
            // MIR_get_module_list returns a pointer to a DLIST (doubly linked list)
            let module_list_ptr = MIR_get_module_list(ctx);
            // We want the last module added (tail)
            let module = (*module_list_ptr).tail;
            assert!(!module.is_null(), "Failed to parse module");

            // 5. Load the Module
            MIR_load_module(ctx, module);

            // 6. Find the Function Item
            // We need to traverse the module's items to find "add_nums"
            let target_func_name = CString::new("add_nums").unwrap();
            let mut func_item = (*module).items.head; // Start at head of item list
            let mut found_func = ptr::null_mut();

            while !func_item.is_null() {
                // Check if item is a function
                if (*func_item).item_type == MIR_item_type_t_MIR_func_item {
                    // Get name
                    let name_ptr = MIR_item_name(ctx, func_item);
                    let name = std::ffi::CStr::from_ptr(name_ptr);

                    if name == target_func_name.as_c_str() {
                        found_func = func_item;
                        break;
                    }
                }
                // Move to next item
                func_item = (*func_item).item_link.next;
            }

            assert!(
                !found_func.is_null(),
                "Function 'add_nums' not found in module"
            );

            // 7. Link
            MIR_link(ctx, Some(MIR_set_gen_interface), None);

            // 8. Generate Machine Code
            let fun_ptr = MIR_gen(ctx, found_func);
            assert!(!fun_ptr.is_null());

            // 9. Execute
            let rust_func: extern "C" fn(i64, i64) -> i64 = std::mem::transmute(fun_ptr);
            let result = rust_func(100, 50);

            println!("MIR String execution result: {}", result);
            assert_eq!(result, 150);

            // 10. Cleanup
            MIR_gen_finish(ctx);
            MIR_finish(ctx);
        }
    }
}
