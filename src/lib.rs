pub mod ffi;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocate_deallocate() {
        let size = 1024;
        let ptr = allocate(size);
        assert!(!ptr.is_null());
        
        unsafe {
            // Write to memory to ensure it's valid
            for i in 0..size {
                *ptr.add(i) = (i % 256) as u8;
            }
            
            // Read back to verify
            for i in 0..size {
                assert_eq!(*ptr.add(i), (i % 256) as u8);
            }
        }
        
        deallocate(ptr, size);
    }
}
