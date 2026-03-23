use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn chorograph_plugin(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;
    let fn_name_str = fn_name.to_string();

    let wrapper = match fn_name_str.as_str() {
        "init" => generate_init_wrapper(&input),
        "handle_action" => generate_handle_action_wrapper(&input),
        "on_workspace_change" => generate_on_workspace_change_wrapper(&input),
        "identify_project" => generate_identify_project_wrapper(&input),
        _ => quote! { #input },
    };

    // Generate allocate/deallocate. 
    // We'll generate them for 'init' specifically, or provide a way.
    // If we generate them for 'init', and the user doesn't have an 'init', 
    // they can use #[chorograph_plugin] on another function.
    // Let's make it so if it's NOT one of the three, it STILL generates them?
    // No, let's just generate them if it's 'init' OR if the user explicitly wants them.
    // The instructions say "generate them as standard".
    
    let exports = if fn_name_str == "init" {
        generate_standard_exports()
    } else {
        quote! {}
    };

    let expanded = quote! {
        #wrapper
        #exports
    };

    TokenStream::from(expanded)
}

fn generate_init_wrapper(input: &ItemFn) -> proc_macro2::TokenStream {
    let fn_name = &input.sig.ident;
    quote! {
        #input

        #[no_mangle]
        #[export_name = "run"]
        pub unsafe extern "C" fn __ffi_run() {
            #fn_name();
        }
    }
}

fn generate_handle_action_wrapper(input: &ItemFn) -> proc_macro2::TokenStream {
    let fn_name = &input.sig.ident;
    quote! {
        #input

        #[no_mangle]
        #[export_name = "handle_action"]
        pub unsafe extern "C" fn __ffi_handle_action(action_ptr: *const u8, action_len: usize, payload_ptr: *const u8, payload_len: usize) {
            let action_id = {
                let slice = std::slice::from_raw_parts(action_ptr, action_len);
                String::from_utf8_lossy(slice).into_owned()
            };
            let payload: serde_json::Value = {
                let slice = std::slice::from_raw_parts(payload_ptr, payload_len);
                serde_json::from_slice(slice).unwrap_or(serde_json::Value::Null)
            };
            #fn_name(action_id, payload);
        }
    }
}

fn generate_on_workspace_change_wrapper(input: &ItemFn) -> proc_macro2::TokenStream {
    let fn_name = &input.sig.ident;
    quote! {
        #input

        #[no_mangle]
        #[export_name = "on_workspace_change"]
        pub unsafe extern "C" fn __ffi_on_workspace_change(event_ptr: *const u8, event_len: usize) {
            let event: serde_json::Value = {
                let slice = std::slice::from_raw_parts(event_ptr, event_len);
                serde_json::from_slice(slice).unwrap_or(serde_json::Value::Null)
            };
            #fn_name(event);
        }
    }
}

fn generate_identify_project_wrapper(input: &ItemFn) -> proc_macro2::TokenStream {
    let fn_name = &input.sig.ident;
    quote! {
        #input

        #[no_mangle]
        #[export_name = "identify_project"]
        pub unsafe extern "C" fn __ffi_identify_project(root_ptr: *const u8, root_len: usize, files_ptr: *const u8, files_len: usize) -> u64 {
            let root = {
                let slice = std::slice::from_raw_parts(root_ptr, root_len);
                String::from_utf8_lossy(slice).into_owned()
            };
            let files: Vec<String> = {
                let slice = std::slice::from_raw_parts(files_ptr, files_len);
                serde_json::from_slice(slice).unwrap_or_default()
            };
            
            if let Some(profile) = #fn_name(root, files) {
                if let Ok(json) = serde_json::to_string(&profile) {
                    let b = json.as_bytes();
                    let ptr = allocate(b.len());
                    std::ptr::copy_nonoverlapping(b.as_ptr(), ptr, b.len());
                    return ((ptr as u64) << 32) | (b.len() as u64);
                }
            }
            0
        }
    }
}

fn generate_standard_exports() -> proc_macro2::TokenStream {
    quote! {
        #[no_mangle]
        pub extern "C" fn allocate(size: usize) -> *mut u8 {
            let mut buf = vec![0u8; size];
            let ptr = buf.as_mut_ptr();
            std::mem::forget(buf);
            ptr
        }

        #[no_mangle]
        pub extern "C" fn deallocate(ptr: *mut u8, size: usize) {
            if ptr.is_null() {
                return;
            }
            unsafe {
                let _ = Vec::from_raw_parts(ptr, size, size);
            }
        }
    }
}
