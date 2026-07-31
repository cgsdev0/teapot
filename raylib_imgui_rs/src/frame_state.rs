pub struct FrameState {
	pub is_window_focused: bool,
	pub is_control_pressed: bool,
	pub is_shift_pressed: bool,
	pub is_alt_pressed: bool,
	pub is_super_pressed: bool,
}

impl FrameState {
	pub fn new(raylib_handle: &mut raylib::RaylibHandle) -> Self {
		FrameState {
			is_window_focused: raylib_handle.is_window_focused(),
			is_control_pressed: false,
			is_shift_pressed: false,
			is_alt_pressed: false,
			is_super_pressed: false,
		}
	}
}