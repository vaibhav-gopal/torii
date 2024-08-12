#[cfg(target_os = "windows")]
use ash::khr::win32_surface;

#[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
use ash::khr::xlib_surface;

#[cfg(target_os = "macos")]
use ash::mvk::macos_surface;

use ash::ext::debug_utils;
use ash::khr::surface;

// required extension ------------------------------------------------------------
#[cfg(target_os = "windows")]
pub fn required_extension_names() -> Vec<*const i8> {
    vec![
        surface::NAME.as_ptr(),
        win32_surface::NAME.as_ptr(),
        debug_utils::NAME.as_ptr(),
    ]
}

#[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
pub fn required_extension_names() -> Vec<*const i8> {
    vec![
        surface::NAME.as_ptr(),
        xlib_surface::NAME.as_ptr(),
        debug_utils::NAME.as_ptr(),
    ]
}

#[cfg(target_os = "macos")]
pub fn required_extension_names() -> Vec<*const i8> {
    vec![
        surface::NAME.as_ptr(),
        macos_surface::NAME.as_ptr(),
        debug_utils::NAME.as_ptr(),
    ]
}
