use std::ffi::CString;

pub fn cstrings_to_raw(cstrings: &Vec<CString>) -> Vec<*const u8> {
    return cstrings.iter().map(|e| e.as_ptr()).collect::<Vec<_>>();
}
