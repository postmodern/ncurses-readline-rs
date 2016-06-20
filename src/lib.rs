extern crate libc;
extern crate ncurses;

pub trait Callbacks : Sync {
	fn on_input(&self, buffer: &str);
	fn on_redisplay(&self, prompt: &str, buffer: &str);
}

static mut registered_callbacks: Option<&Callbacks> = None;

pub mod readline;

use std::ptr;

pub fn init<C: Callbacks>(callbacks: C) {
	registered_callbacks = Some(callbacks);

	readline::hook();

	if ncurses::cbreak() == ncurses::ERR {
		panic!("ncurses cbreak() failed");
	}

	if ncurses::noecho() == ncurses::ERR {
		panic!("ncurses noecho() failed");
	}

	if ncurses::nonl() == ncurses::ERR {
		panic!("ncurses nonl() failed");
	}

	if ncurses::intrflush(ptr::null_mut(), false) == ncurses::ERR {
		panic!("ncurses intrflush() failed");
	}

	/*
	 * Note: do not enable keypad() since we want to pass unadultered input
	 * to readline.
	 */
}

pub fn deinit() {
	readline::unhook();

	ncurses::intrflush(ptr::null_mut(), true);
	ncurses::nl();
	ncurses::echo();
	ncurses::nocbreak();

	registered_callbacks = None;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
