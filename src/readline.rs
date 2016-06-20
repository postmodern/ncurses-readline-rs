#![allow(dead_code)]
#![allow(non_camel_case_types)]

use libc::{c_int, c_char, c_void, FILE, self};
use std::ffi::CStr;

type rl_command_func_t = Option<unsafe extern "C" fn(c_int, c_int) -> c_int>;
type rl_voidfunc_t = Option<extern "C" fn()>;
type rl_vintfunc_t = Option<extern "C" fn(c_int)>; 
type rl_vcpfunc_t = Option<extern "C" fn(*mut c_char)>;
type rl_getc_func_t = Option<extern "C" fn(*mut FILE) -> c_int>;
type rl_hook_func_t = Option<extern "C" fn() -> c_int>;

#[link(name = "readline")]
extern {
	static mut rl_display_prompt: *mut c_char;
	static mut rl_line_buffer: *mut c_char;
	static mut rl_point: c_int;
	static mut rl_catch_signals: c_int;
	static mut rl_catch_sigwinch: c_int;
	static mut rl_deprep_term_function: rl_voidfunc_t;
	static mut rl_prep_term_function: rl_vintfunc_t;
	static mut rl_change_environment: c_int;
	static mut rl_getc_function: rl_getc_func_t;
	static mut rl_input_available_hook: rl_hook_func_t;
	static mut rl_redisplay_function: rl_voidfunc_t;

	fn rl_set_prompt(prompt: *const c_char) -> c_int;
	fn rl_callback_read_char();
	fn rl_insert(_: c_int, _: c_int) -> c_int;
	fn rl_bind_key(key: c_int, callback: rl_command_func_t) -> c_int;
	fn rl_unbind_key(key: c_int) -> c_int;
	fn rl_callback_handler_install(prompt: *const c_char, callback: rl_vcpfunc_t);
	fn rl_callback_handler_remove();

	fn add_history(line: *const c_char);
}

static mut input_callback: Option<fn(&str)> = None;
static mut input_available: bool = false;
static mut input_eof: bool = false;
static mut input: i32 = 0;

static mut redisplay_callback: Option<fn(&str, &str)> = None;

const KEY_TAB: i32 = '\t' as i32;

pub fn hook(input_func: fn(&str), redisplay_func: fn(&str, &str)) {
	if unsafe { rl_bind_key(KEY_TAB, Some(rl_insert)) } != 0 {
		panic!("invalid key passed to rl_bind_key()");
	}

	unsafe {
		// disable readline's signal handling
		rl_catch_signals        = 0;
		rl_catch_sigwinch       = 0;
		rl_deprep_term_function = None;
		rl_prep_term_function   = None;
	}

	/*
	 * Prevent readline from setting the LINES and COLUMNS environment
	 * variables, which override dynamic size adjustments in ncurses. When
	 * using the alternate readline interface (as we do here), LINES and
	 * COLUMNS are not updated if the terminal is resized between two calls to
	 * rl_callback_read_char() (which is almost always the case).
	 */
	unsafe { rl_change_environment = 0; }

	// Handle input by manually feeding characters to readline.
	unsafe {
		rl_getc_function        = Some(getc);
		input_callback          = Some(input_func);
		rl_input_available_hook = Some(is_input_available);

		redisplay_callback      = Some(redisplay_func);
		rl_redisplay_function   = Some(redisplay_handler);
	}
}

pub fn start<'main>(prompt: &str) {
	unsafe {
		rl_callback_handler_install(
			prompt.as_ptr() as (*const i8),
			Some(input_handler)
		);
	}
}

pub fn cursor_index() -> i32 { unsafe { rl_point } }

pub fn send(key: i32) {
	unsafe {
		input = key;
		input_available = true;

		rl_callback_read_char();
	}
}

pub fn eof() -> bool { unsafe { input_eof } }

pub fn unhook() {
	unsafe {
		rl_callback_handler_remove();
		rl_unbind_key(KEY_TAB);
	}

	unsafe {
		input_available = false;
		input_eof       = false;
		input           = 0;
		input_callback  = None;

		redisplay_callback = None;
	}
}

extern "C" fn is_input_available() -> i32 {
	unsafe { input_available as i32 }
}

extern "C" fn getc(_: *mut FILE) -> i32 {
	unsafe {
		input_available = false;

		return input;
	}
}

extern "C" fn input_handler(ptr: *mut c_char) {
	if ptr.is_null() {
		unsafe { input_eof = true; }
		return;
	}

	let line = unsafe { CStr::from_ptr(ptr) };

	if line.to_bytes().len() > 0 {
		let callback = unsafe { input_callback }.unwrap();
		let slice = line.to_str().unwrap();

		unsafe { add_history(ptr); }

		callback(slice);
	}

	unsafe { libc::free(ptr as *mut c_void); }
}

extern "C" fn redisplay_handler() {
	let prompt   = unsafe { CStr::from_ptr(rl_display_prompt) };
	let buffer   = unsafe { CStr::from_ptr(rl_line_buffer) };
	let callback = unsafe { redisplay_callback.unwrap() };

	callback(
		prompt.to_str().unwrap(),
		buffer.to_str().unwrap()
	);
}
