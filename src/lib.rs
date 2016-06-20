extern crate libc;
extern crate ncurses;

pub mod readline;

use std::ptr;

pub fn init(input_func: fn(&str), redisplay_func: fn(&str, &str)) {
	readline::hook(input_func, redisplay_func);

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
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
