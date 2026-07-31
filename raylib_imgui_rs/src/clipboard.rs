use raylib::prelude::*;
use std::ffi::{CStr, CString};

pub struct ClipboardBackend;

impl imgui::ClipboardBackend for ClipboardBackend {
	fn get(&mut self) -> Option<String> {
		unsafe {
			let text = ffi::GetClipboardText();

			if text.is_null() {
				return None;
			}

			Some(CStr::from_ptr(text).to_string_lossy().to_string())
		}
	}

	fn set(&mut self, value: &str) {
		// Try to create the CString. If value contains a null
		let str = create_c_string(value);
		unsafe {
			ffi::SetClipboardText(str.as_ptr());
		}
	}
}

fn create_c_string(value: &str) -> CString {
	CString::new(value).unwrap_or_else(|mut err| {
		let mut value = value.to_string();

		loop {
			value.remove(err.nul_position());

			match CString::new(value.as_str()) {
				Ok(str) => { return str; }
				Err(new_err) => { err = new_err; }
			}
		}
	})
}