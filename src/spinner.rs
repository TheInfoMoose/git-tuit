use ratatui::{backend::Backend, style::Style, Terminal};
use std::{cell::Cell, char};

// static SPINNER_CHARS: &[char] = &['◢', '◣', '◤', '◥'];
// static SPINNER_CHARS: &[char] = &['⢹', '⢺', '⢼', '⣸', '⣇', '⡧', '⡗', '⡏'];
static SPINNER_CHARS: &[char] =
	&['⣷', '⣯', '⣟', '⡿', '⢿', '⣻', '⣽', '⣾'];

///
pub struct Spinner {
	idx: usize,
	active: bool,
	last_char: Cell<char>,
	style: Style,
}

impl Default for Spinner {
	fn default() -> Self {
		Self::new(Style::default())
	}
}

impl Spinner {
	///
	pub const fn new(style: Style) -> Self {
		Self {
			idx: 0,
			active: false,
			last_char: Cell::new(' '),
			style,
		}
	}

	/// increment spinner graphic by one
	pub fn update(&mut self) {
		self.idx += 1;
		self.idx %= SPINNER_CHARS.len();
	}

	///
	pub const fn set_state(&mut self, active: bool) {
		self.active = active;
	}

	/// draws or removes spinner char depending on `pending` state
	pub fn draw<B: ratatui::backend::Backend>(
		&self,
		terminal: &mut Terminal<B>,
	) -> Result<(), B::Error> {
		let idx = self.idx;

		let char_to_draw =
			if self.active { SPINNER_CHARS[idx] } else { ' ' };

		if self.last_char.get() != char_to_draw {
			self.last_char.set(char_to_draw);

			let mut c = ratatui::buffer::Cell::default();
			c.set_char(char_to_draw).set_style(self.style);

			terminal
				.backend_mut()
				.draw(vec![(0_u16, 0_u16, &c)].into_iter())?;

			Backend::flush(terminal.backend_mut())?;
		}

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::ui::style::Theme;
	use ratatui::{backend::TestBackend, style::Color};

	#[test]
	fn t15f_spinner_cell_styled() {
		let mut terminal =
			Terminal::new(TestBackend::new(18, 12)).unwrap();

		// Amber (default) theme
		let mut spinner = Spinner::new(Theme::default().spinner());
		spinner.set_state(true);
		spinner.draw(&mut terminal).unwrap();

		let cell = &terminal.backend().buffer()[(0, 0)];
		assert_eq!(cell.symbol(), SPINNER_CHARS[0].to_string());
		assert_eq!(cell.fg, Color::Rgb(255, 216, 102));
		assert_eq!(cell.bg, Color::Rgb(0, 0, 0));

		// Unchanged char writes nothing
		let mut sentinel = ratatui::buffer::Cell::default();
		sentinel.set_char('X');
		terminal
			.backend_mut()
			.draw(std::iter::once((0_u16, 0_u16, &sentinel)))
			.unwrap();
		spinner.draw(&mut terminal).unwrap();
		assert_eq!(terminal.backend().buffer()[(0, 0)].symbol(), "X");

		// update() while active advances the char
		spinner.update();
		spinner.draw(&mut terminal).unwrap();
		let cell = &terminal.backend().buffer()[(0, 0)];
		assert_eq!(cell.symbol(), SPINNER_CHARS[1].to_string());
		assert_eq!(cell.fg, Color::Rgb(255, 216, 102));
		assert_eq!(cell.bg, Color::Rgb(0, 0, 0));

		// Inactive keeps canvas_bg
		spinner.set_state(false);
		spinner.draw(&mut terminal).unwrap();
		let cell = &terminal.backend().buffer()[(0, 0)];
		assert_eq!(cell.symbol(), " ");
		assert_eq!(cell.bg, Color::Rgb(0, 0, 0));

		// Classic theme: fg and bg Reset in both states
		let mut classic = Spinner::new(Theme::classic().spinner());
		classic.set_state(true);
		classic.draw(&mut terminal).unwrap();
		let cell = &terminal.backend().buffer()[(0, 0)];
		assert_eq!(cell.symbol(), SPINNER_CHARS[0].to_string());
		assert_eq!(cell.fg, Color::Reset);
		assert_eq!(cell.bg, Color::Reset);

		classic.set_state(false);
		classic.draw(&mut terminal).unwrap();
		let cell = &terminal.backend().buffer()[(0, 0)];
		assert_eq!(cell.symbol(), " ");
		assert_eq!(cell.fg, Color::Reset);
		assert_eq!(cell.bg, Color::Reset);
	}
}
