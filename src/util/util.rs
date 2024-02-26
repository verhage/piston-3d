use ash::vk::{api_version_major, api_version_minor, api_version_patch};
use std::ffi::{c_char, CStr};

pub fn vk_to_string(raw_string_array: &[c_char]) -> String {
    unsafe {
        CStr::from_ptr(raw_string_array.as_ptr())
            .to_str()
            .expect("Failed to convert raw vk string!")
            .to_owned()
    }
}

pub fn vk_version_to_string(version: u32) -> String {
    let major = api_version_major(version);
    let minor = api_version_minor(version);
    let patch = api_version_patch(version);
    format!("{}.{}.{}", major, minor, patch)
}

pub fn yes_no<'value>(value: bool) -> &'value str {
    if value {
        "yes"
    } else {
        "no"
    }
}
