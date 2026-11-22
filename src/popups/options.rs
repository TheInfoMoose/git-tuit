use crate::{
	app::Environment,
	components::{
		string_width_align, visibility_blocking, CommandBlocking,
		CommandInfo, Component, DrawableComponent, EventState,
	},
	keys::{key_match, SharedKeyConfig},
	options::SharedOptions,
	queue::{InternalEvent, Queue},
	strings,
	ui::{self, style::SharedTheme},
};
use anyhow::Result;
use asyncgit::sync::ShowUntrackedFilesConfig;
use crossterm::event::Event;
use ratatui::{
	layout::{Alignment, Rect},
	style::{Modifier, Style},
	text::{Line, Span},
	widgets::{Block, Borders, Clear, Paragraph},
	Frame,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AppOption {
	StatusShowUntracked,
	DiffIgnoreWhitespaces,
	DiffContextLines,
	DiffInterhunkLines,
}

pub struct OptionsPopup {
	selection: AppOption,
	queue: Queue,
	visible: bool,
	key_config: SharedKeyConfig,
	options: SharedOptions,
	theme: SharedTheme,
}

impl OptionsPopup {
	///
	pub fn new(env: &Environment) -> Self {
		Self {
			selection: AppOption::StatusShowUntracked,
			queue: env.queue.clone(),
			visible: false,
			key_config: env.key_config.clone(),
			options: env.options.clone(),
			theme: env.theme.clone(),
		}
	}

	fn get_text(&self, width: u16) -> Vec<Line<'_>> {
		let mut txt: Vec<Line> = Vec::with_capacity(10);

		self.add_status(&mut txt, width);

		txt
	}

	fn add_status(&self, txt: &mut Vec<Line>, width: u16) {
		Self::add_header(txt, "Status");

		self.add_entry(
			txt,
			width,
			"Show untracked",
			match self.options.borrow().status_show_untracked() {
				None => "Gitconfig",
				Some(ShowUntrackedFilesConfig::No) => "No",
				Some(ShowUntrackedFilesConfig::Normal) => "Normal",
				Some(ShowUntrackedFilesConfig::All) => "All",
			},
			self.is_select(AppOption::StatusShowUntracked),
		);
		Self::add_header(txt, "");

		let diff = self.options.borrow().diff_options();
		Self::add_header(txt, "Diff");
		self.add_entry(
			txt,
			width,
			"Ignore whitespaces",
			&diff.ignore_whitespace.to_string(),
			self.is_select(AppOption::DiffIgnoreWhitespaces),
		);
		self.add_entry(
			txt,
			width,
			"Context lines",
			&diff.context.to_string(),
			self.is_select(AppOption::DiffContextLines),
		);
		self.add_entry(
			txt,
			width,
			"Inter hunk lines",
			&diff.interhunk_lines.to_string(),
			self.is_select(AppOption::DiffInterhunkLines),
		);
	}

	fn is_select(&self, kind: AppOption) -> bool {
		self.selection == kind
	}

	fn add_header(txt: &mut Vec<Line>, header: &'static str) {
		txt.push(Line::from(vec![Span::styled(
			header,
			//TODO: use style
			Style::default().add_modifier(Modifier::UNDERLINED),
		)]));
	}

	fn add_entry(
		&self,
		txt: &mut Vec<Line>,
		width: u16,
		entry: &'static str,
		value: &str,
		selected: bool,
	) {
		let half = usize::from(width / 2);
		txt.push(Line::from(vec![
			Span::styled(
				string_width_align(entry, half),
				self.theme.text(true, false),
			),
			Span::styled(
				format!("{value:^half$}"),
				self.theme.text(true, selected),
			),
		]));
	}

	const fn move_selection(&mut self, up: bool) {
		if up {
			self.selection = match self.selection {
				AppOption::StatusShowUntracked => {
					AppOption::DiffInterhunkLines
				}
				AppOption::DiffIgnoreWhitespaces => {
					AppOption::StatusShowUntracked
				}
				AppOption::DiffContextLines => {
					AppOption::DiffIgnoreWhitespaces
				}
				AppOption::DiffInterhunkLines => {
					AppOption::DiffContextLines
				}
			};
		} else {
			self.selection = match self.selection {
				AppOption::StatusShowUntracked => {
					AppOption::DiffIgnoreWhitespaces
				}
				AppOption::DiffIgnoreWhitespaces => {
					AppOption::DiffContextLines
				}
				AppOption::DiffContextLines => {
					AppOption::DiffInterhunkLines
				}
				AppOption::DiffInterhunkLines => {
					AppOption::StatusShowUntracked
				}
			};
		}
	}

	fn switch_option(&self, right: bool) {
		if right {
			match self.selection {
				AppOption::StatusShowUntracked => {
					let untracked =
						self.options.borrow().status_show_untracked();

					let untracked = match untracked {
						None => {
							Some(ShowUntrackedFilesConfig::Normal)
						}
						Some(ShowUntrackedFilesConfig::Normal) => {
							Some(ShowUntrackedFilesConfig::All)
						}
						Some(ShowUntrackedFilesConfig::All) => {
							Some(ShowUntrackedFilesConfig::No)
						}
						Some(ShowUntrackedFilesConfig::No) => None,
					};

					self.options
						.borrow_mut()
						.set_status_show_untracked(untracked);
				}
				AppOption::DiffIgnoreWhitespaces => {
					self.options
						.borrow_mut()
						.diff_toggle_whitespace();
				}
				AppOption::DiffContextLines => {
					self.options
						.borrow_mut()
						.diff_context_change(true);
				}
				AppOption::DiffInterhunkLines => {
					self.options
						.borrow_mut()
						.diff_hunk_lines_change(true);
				}
			}
		} else {
			match self.selection {
				AppOption::StatusShowUntracked => {
					let untracked =
						self.options.borrow().status_show_untracked();

					let untracked = match untracked {
						None => Some(ShowUntrackedFilesConfig::No),
						Some(ShowUntrackedFilesConfig::No) => {
							Some(ShowUntrackedFilesConfig::All)
						}
						Some(ShowUntrackedFilesConfig::All) => {
							Some(ShowUntrackedFilesConfig::Normal)
						}
						Some(ShowUntrackedFilesConfig::Normal) => {
							None
						}
					};

					self.options
						.borrow_mut()
						.set_status_show_untracked(untracked);
				}
				AppOption::DiffIgnoreWhitespaces => {
					self.options
						.borrow_mut()
						.diff_toggle_whitespace();
				}
				AppOption::DiffContextLines => {
					self.options
						.borrow_mut()
						.diff_context_change(false);
				}
				AppOption::DiffInterhunkLines => {
					self.options
						.borrow_mut()
						.diff_hunk_lines_change(false);
				}
			}
		}

		self.queue
			.push(InternalEvent::OptionSwitched(self.selection));
	}
}

impl DrawableComponent for OptionsPopup {
	fn draw(&self, f: &mut Frame, area: Rect) -> Result<()> {
		if self.is_visible() {
			const SIZE: (u16, u16) = (50, 10);
			let area =
				ui::centered_rect_absolute(SIZE.0, SIZE.1, area);

			let width = area.width;

			f.render_widget(Clear, area);
			f.render_widget(
				Paragraph::new(self.get_text(width))
					.block(
						Block::default()
							.borders(Borders::ALL)
							.title(Span::styled(
								"Options",
								self.theme.title(true),
							))
							.border_style(self.theme.block(true)),
					)
					.alignment(Alignment::Left),
				area,
			);
		}

		Ok(())
	}
}

impl Component for OptionsPopup {
	fn commands(
		&self,
		out: &mut Vec<CommandInfo>,
		force_all: bool,
	) -> CommandBlocking {
		if self.is_visible() || force_all {
			out.push(
				CommandInfo::new(
					strings::commands::close_popup(&self.key_config),
					true,
					true,
				)
				.order(1),
			);
			out.push(
				CommandInfo::new(
					strings::commands::options_select(
						&self.key_config,
					),
					true,
					true,
				)
				.order(1),
			);
			out.push(
				CommandInfo::new(
					strings::commands::options_change(
						&self.key_config,
					),
					true,
					true,
				)
				.order(1),
			);
		}

		visibility_blocking(self)
	}

	fn event(
		&mut self,
		event: &crossterm::event::Event,
	) -> Result<EventState> {
		if self.is_visible() {
			if let Event::Key(key) = &event {
				if key_match(key, self.key_config.keys.exit_popup) {
					self.hide();
				} else if key_match(key, self.key_config.keys.move_up)
				{
					self.move_selection(true);
				} else if key_match(
					key,
					self.key_config.keys.move_down,
				) {
					self.move_selection(false);
				} else if key_match(
					key,
					self.key_config.keys.move_right,
				) {
					self.switch_option(true);
				} else if key_match(
					key,
					self.key_config.keys.move_left,
				) {
					self.switch_option(false);
				} else if key_match(
					key,
					self.key_config.keys.cmd_bar_toggle,
				) {
					return Ok(EventState::NotConsumed);
				}
			}

			return Ok(EventState::Consumed);
		}

		Ok(EventState::NotConsumed)
	}

	fn is_visible(&self) -> bool {
		self.visible
	}

	fn hide(&mut self) {
		self.visible = false;
	}

	fn show(&mut self) -> Result<()> {
		self.visible = true;

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::keys::KeyConfig;
	use crossterm::event::{KeyCode, KeyModifiers};
	use std::rc::Rc;

	#[test]
	fn t8k_options_hint() {
		let mut env = Environment::test_env();
		let mut kc = KeyConfig::default();
		kc.keys.move_up.code = KeyCode::Char('u');
		kc.keys.move_up.modifiers = KeyModifiers::empty();
		kc.keys.move_down.code = KeyCode::Char('d');
		kc.keys.move_down.modifiers = KeyModifiers::empty();
		kc.keys.move_left.code = KeyCode::Char('l');
		kc.keys.move_left.modifiers = KeyModifiers::empty();
		kc.keys.move_right.code = KeyCode::Char('r');
		kc.keys.move_right.modifiers = KeyModifiers::empty();
		env.key_config = Rc::new(kc);

		let mut popup = OptionsPopup::new(&env);
		popup.show().unwrap();

		let mut cmds = Vec::new();
		popup.commands(&mut cmds, false);

		for cmd in &cmds {
			let lower_desc = cmd.text.desc.to_lowercase();
			let lower_name = cmd.text.name.to_lowercase();
			assert!(
				!lower_desc.contains("tree")
					&& !lower_desc.contains("collapse")
					&& !lower_desc.contains("expand")
					&& !lower_name.contains("tree"),
				"OptionsPopup must not have tree wording: {} ({})",
				cmd.text.name,
				cmd.text.desc
			);
		}

		assert!(
			cmds.iter().any(|c| {
				c.text.name.contains('u')
					&& c.text.name.contains('d')
					&& c.text.desc.to_lowercase().contains("select")
			}),
			"OptionsPopup must advertise move_up/move_down for selecting: {:?}",
			cmds.iter()
				.map(|c| (&c.text.name, c.text.desc))
				.collect::<Vec<_>>()
		);
		assert!(
			cmds.iter().any(|c| {
				c.text.name.contains('l')
					&& c.text.name.contains('r')
					&& c.text.desc.to_lowercase().contains("change")
			}),
			"OptionsPopup must advertise move_left/move_right for changing value: {:?}",
			cmds.iter()
				.map(|c| (&c.text.name, c.text.desc))
				.collect::<Vec<_>>()
		);
	}
}
