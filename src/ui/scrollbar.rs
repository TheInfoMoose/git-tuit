use super::style::SharedTheme;
use easy_cast::CastFloat;
use ratatui::{
	buffer::Buffer,
	layout::{Margin, Rect},
	style::Style,
	symbols::{
		block::FULL,
		line::{DOUBLE_HORIZONTAL, DOUBLE_VERTICAL},
	},
	widgets::Widget,
	Frame,
};

pub enum Orientation {
	Vertical,
	Horizontal,
}

///
struct Scrollbar {
	max: u16,
	pos: u16,
	style_bar: Style,
	style_pos: Style,
	orientation: Orientation,
}

impl Scrollbar {
	fn new(max: usize, pos: usize, orientation: Orientation) -> Self {
		Self {
			max: u16::try_from(max).unwrap_or_default(),
			pos: u16::try_from(pos).unwrap_or_default(),
			style_pos: Style::default(),
			style_bar: Style::default(),
			orientation,
		}
	}

	fn render_vertical(self, area: Rect, buf: &mut Buffer) {
		if area.height <= 2 {
			return;
		}

		if self.max == 0 {
			return;
		}

		let right = area.right().saturating_sub(1);
		if right <= area.left() {
			return;
		}

		let (bar_top, bar_height) = {
			let scrollbar_area = area.inner(Margin {
				horizontal: 0,
				vertical: 1,
			});

			(scrollbar_area.top(), scrollbar_area.height)
		};

		for y in bar_top..(bar_top + bar_height) {
			buf.set_string(right, y, DOUBLE_VERTICAL, self.style_bar);
		}

		let progress = f32::from(self.pos) / f32::from(self.max);
		let progress = if progress > 1.0 { 1.0 } else { progress };
		let pos = f32::from(bar_height) * progress;

		let pos: u16 = pos.cast_nearest();
		let pos = pos.saturating_sub(1);

		buf.set_string(right, bar_top + pos, FULL, self.style_pos);
	}

	fn render_horizontal(self, area: Rect, buf: &mut Buffer) {
		if area.width <= 2 {
			return;
		}

		if self.max == 0 {
			return;
		}

		let bottom = area.bottom().saturating_sub(1);
		if bottom <= area.top() {
			return;
		}

		let (bar_left, bar_width) = {
			let scrollbar_area = area.inner(Margin {
				horizontal: 1,
				vertical: 0,
			});

			(scrollbar_area.left(), scrollbar_area.width)
		};

		for x in bar_left..(bar_left + bar_width) {
			buf.set_string(
				x,
				bottom,
				DOUBLE_HORIZONTAL,
				self.style_bar,
			);
		}

		let progress = f32::from(self.pos) / f32::from(self.max);
		let progress = if progress > 1.0 { 1.0 } else { progress };
		let pos = f32::from(bar_width) * progress;

		let pos: u16 = pos.cast_nearest();
		let pos = pos.saturating_sub(1);

		buf.set_string(bar_left + pos, bottom, FULL, self.style_pos);
	}
}

impl Widget for Scrollbar {
	fn render(self, area: Rect, buf: &mut Buffer) {
		match &self.orientation {
			Orientation::Vertical => self.render_vertical(area, buf),
			Orientation::Horizontal => {
				self.render_horizontal(area, buf);
			}
		}
	}
}

pub fn draw_scrollbar(
	f: &mut Frame,
	r: Rect,
	theme: &SharedTheme,
	max: usize,
	pos: usize,
	orientation: Orientation,
) {
	let mut widget = Scrollbar::new(max, pos, orientation);
	widget.style_bar = theme.scroll_bar_track();
	widget.style_pos = theme.scroll_bar_pos();
	f.render_widget(widget, r);
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::ui::style::Theme;
	use ratatui::{backend::TestBackend, style::Color, Terminal};
	use std::{io::Write, rc::Rc};
	use tempfile::NamedTempFile;

	fn render_vertical_scrollbar(theme: &SharedTheme) -> Buffer {
		let mut terminal =
			Terminal::new(TestBackend::new(3, 10)).unwrap();

		terminal
			.draw(|f| {
				draw_scrollbar(
					f,
					f.area(),
					theme,
					10,
					0,
					Orientation::Vertical,
				);
			})
			.unwrap();

		terminal.backend().buffer().clone()
	}

	fn assert_scrollbar_colors(
		buf: &Buffer,
		expected_track: Color,
		expected_thumb: Color,
	) {
		let right = buf.area.right() - 1;
		let mut track_count = 0;
		let mut pos_count = 0;

		for y in buf.area.top()..buf.area.bottom() {
			let cell = &buf[(right, y)];
			if cell.symbol() == DOUBLE_VERTICAL {
				track_count += 1;
				assert_eq!(cell.fg, expected_track);
			} else if cell.symbol() == FULL {
				pos_count += 1;
				assert_eq!(cell.fg, expected_thumb);
			}
		}

		assert!(track_count > 0);
		assert_eq!(pos_count, 1);
	}

	#[test]
	fn t12b_scrollbar_track_styled() {
		let theme = Rc::new(Theme::default());
		let buf = render_vertical_scrollbar(&theme);
		assert_scrollbar_colors(
			&buf,
			Color::Rgb(92, 58, 0),
			Color::Rgb(255, 216, 102),
		);
	}

	#[test]
	fn t13a_thumb_color_from_patch() {
		let mut override_file = NamedTempFile::new().unwrap();
		writeln!(override_file, r#"(scroll_thumb_fg: Some("Cyan"))"#)
			.unwrap();
		let overridden =
			Rc::new(Theme::init(&override_file.path().to_path_buf()));
		let buf = render_vertical_scrollbar(&overridden);
		assert_scrollbar_colors(&buf, Color::Reset, Color::Cyan);

		let mut selection_file = NamedTempFile::new().unwrap();
		writeln!(selection_file, r#"(selection_bg: Some("Black"))"#)
			.unwrap();
		let selection_only = Rc::new(Theme::init(
			&selection_file.path().to_path_buf(),
		));
		let buf = render_vertical_scrollbar(&selection_only);
		assert_scrollbar_colors(&buf, Color::Reset, Color::Black);
	}
}
