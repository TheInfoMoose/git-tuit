use easy_cast::Cast;
use ratatui::{
	buffer::Buffer,
	layout::{Alignment, Position, Rect},
	style::Style,
	text::{StyledGrapheme, Text},
	widgets::{Block, StatefulWidget, Widget, Wrap},
};
use std::iter;
use unicode_width::UnicodeWidthStr;

use super::reflow::{LineComposer, LineTruncator, WordWrapper};

const fn get_line_offset(
	line_width: u16,
	text_area_width: u16,
	alignment: Alignment,
) -> u16 {
	match alignment {
		Alignment::Center => {
			(text_area_width / 2).saturating_sub(line_width / 2)
		}
		Alignment::Right => {
			text_area_width.saturating_sub(line_width)
		}
		Alignment::Left => 0,
	}
}

#[derive(Debug, Clone)]
pub struct StatefulParagraph<'a> {
	/// A block to wrap the widget in
	block: Option<Block<'a>>,
	/// Widget style
	style: Style,
	/// How to wrap the text
	wrap: Option<Wrap>,
	/// The text to display
	text: Text<'a>,
	/// Alignment of the text
	alignment: Alignment,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ScrollPos {
	pub x: u16,
	pub y: u16,
}

#[derive(Debug, Copy, Clone, Default)]
pub struct ParagraphState {
	/// Scroll
	scroll: ScrollPos,
	/// after all wrapping this is the amount of lines
	lines: u16,
	/// last visible height
	height: u16,
}

impl ParagraphState {
	pub const fn lines(self) -> u16 {
		self.lines
	}

	pub const fn height(self) -> u16 {
		self.height
	}

	pub const fn scroll(self) -> ScrollPos {
		self.scroll
	}

	pub const fn set_scroll(&mut self, scroll: ScrollPos) {
		self.scroll = scroll;
	}
}

impl<'a> StatefulParagraph<'a> {
	pub fn new<T>(text: T) -> Self
	where
		T: Into<Text<'a>>,
	{
		Self {
			block: None,
			style: Style::default(),
			wrap: None,
			text: text.into(),
			alignment: Alignment::Left,
		}
	}

	pub fn block(mut self, block: Block<'a>) -> Self {
		self.block = Some(block);
		self
	}

	pub const fn wrap(mut self, wrap: Wrap) -> Self {
		self.wrap = Some(wrap);
		self
	}

	fn render_pass(
		&self,
		text_area: Rect,
		buf: &mut Buffer,
		scroll_x: u16,
		scroll_y: u16,
		stop_after_view: bool,
	) -> usize {
		let style = self.style;
		let mut styled = self.text.lines.iter().flat_map(|line| {
			line.spans
				.iter()
				.flat_map(|span| span.styled_graphemes(style))
				// Required given the way composers work but might be refactored out if we change
				// composers to operate on lines instead of a stream of graphemes.
				.chain(iter::once(StyledGrapheme {
					symbol: "\n",
					style: self.style,
				}))
		});

		let mut line_composer: Box<dyn LineComposer> =
			if let Some(Wrap { trim }) = self.wrap {
				Box::new(WordWrapper::new(
					&mut styled,
					text_area.width,
					trim,
				))
			} else {
				let mut line_composer = Box::new(LineTruncator::new(
					&mut styled,
					text_area.width,
				));
				if self.alignment == Alignment::Left {
					line_composer.set_horizontal_offset(scroll_x);
				}
				line_composer
			};

		let scroll_y_usize = usize::from(scroll_y);
		let end_y = scroll_y_usize
			.saturating_add(usize::from(text_area.height));
		let mut y: usize = 0;

		while let Some((current_line, current_line_width)) =
			line_composer.next_line()
		{
			if y >= scroll_y_usize && y < end_y {
				let row_offset =
					u16::try_from(y - scroll_y_usize).unwrap_or(0);
				let mut x = get_line_offset(
					current_line_width,
					text_area.width,
					self.alignment,
				);
				for StyledGrapheme { symbol, style } in current_line {
					buf.cell_mut(Position::new(
						text_area.left().saturating_add(x),
						text_area.top().saturating_add(row_offset),
					))
					.map(|cell| {
						cell.set_symbol(if symbol.is_empty() {
							// If the symbol is empty, the last char which rendered last time will
							// leave on the line. It's a quick fix.
							" "
						} else {
							symbol
						})
						.set_style(*style)
					});
					x = x.saturating_add(Cast::<u16>::cast(
						symbol.width(),
					));
				}
			}
			y = y.saturating_add(1);
			if stop_after_view && y >= end_y {
				break;
			}
		}

		y
	}
}

impl StatefulWidget for StatefulParagraph<'_> {
	type State = ParagraphState;

	fn render(
		mut self,
		area: Rect,
		buf: &mut Buffer,
		state: &mut Self::State,
	) {
		buf.set_style(area, self.style);
		let text_area = self.block.take().map_or(area, |b| {
			let inner_area = b.inner(area);
			b.render(area, buf);
			inner_area
		});

		if text_area.height < 1 || text_area.width < 1 {
			state.lines = 0;
			state.height = area.height;
			state.scroll.y = 0;
			return;
		}

		let total_lines = self.render_pass(
			text_area,
			buf,
			state.scroll.x,
			state.scroll.y,
			false,
		);
		let total_lines_u16 =
			u16::try_from(total_lines).unwrap_or(u16::MAX);
		let max_scroll =
			total_lines_u16.saturating_sub(text_area.height);

		if state.scroll.y > max_scroll {
			state.scroll.y = max_scroll;
			for row in text_area.top()..text_area.bottom() {
				for col in text_area.left()..text_area.right() {
					if let Some(cell) =
						buf.cell_mut(Position::new(col, row))
					{
						cell.set_symbol(" ").set_style(self.style);
					}
				}
			}
			self.render_pass(
				text_area,
				buf,
				state.scroll.x,
				state.scroll.y,
				true,
			);
		}

		state.lines = total_lines_u16;
		state.height = area.height;
	}
}
