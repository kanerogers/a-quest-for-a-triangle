use std::{ffi::CString, mem};

pub fn cstrings_to_raw(cstrings: &Vec<CString>) -> Vec<*const u8> {
    return cstrings.iter().map(|e| e.as_ptr()).collect::<Vec<_>>();
}

const fn num_bits<T>() -> usize {
    mem::size_of::<T>() * 8
}

pub fn log_2(x: i32) -> u32 {
    assert!(x > 0);
    num_bits::<i32>() as u32 - x.leading_zeros() - 1
}
