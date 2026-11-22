use anyhow::Result;
use asyncgit::sync::{self, RepoPathRef};
use crossterm::event::Event;
use easy_cast::Cast;
use ratatui::{widgets::Paragraph, Frame};

use crate::{
	app::Environment,
	components::{
		visibility_blocking, CommandBlocking, CommandInfo, Component,
		DrawableComponent, EventState, InputType, TextInputComponent,
	},
	keys::{key_match, SharedKeyConfig},
	queue::{InternalEvent, NeedsUpdate, Queue},
	strings,
	ui::style::SharedTheme,
};

pub struct UpdateRemoteUrlPopup {
	repo: RepoPathRef,
	input: TextInputComponent,
	key_config: SharedKeyConfig,
	queue: Queue,
	theme: SharedTheme,
	remote_name: Option<String>,
	initial_url: Option<String>,
}

impl DrawableComponent for UpdateRemoteUrlPopup {
	fn draw(
		&self,
		f: &mut ratatui::Frame,
		rect: ratatui::prelude::Rect,
	) -> anyhow::Result<()> {
		if self.is_visible() {
			self.input.draw(f, rect)?;
			self.draw_warnings(f);
		}
		Ok(())
	}
}

impl Component for UpdateRemoteUrlPopup {
	fn commands(
		&self,
		out: &mut Vec<crate::components::CommandInfo>,
		force_all: bool,
	) -> CommandBlocking {
		if self.is_visible() || force_all {
			self.input.commands(out, force_all);

			out.push(CommandInfo::new(
				strings::commands::remote_confirm_url_msg(
					&self.key_config,
				),
				self.is_valid_url(),
				true,
			));
		}
		visibility_blocking(self)
	}

	fn event(&mut self, ev: &Event) -> Result<EventState> {
		if self.is_visible() {
			if self.input.event(ev)?.is_consumed() {
				return Ok(EventState::Consumed);
			}

			if let Event::Key(e) = ev {
				if key_match(e, self.key_config.keys.enter) {
					self.update_remote_url();
				}

				return Ok(EventState::Consumed);
			}
		}
		Ok(EventState::NotConsumed)
	}

	fn is_visible(&self) -> bool {
		self.input.is_visible()
	}

	fn hide(&mut self) {
		self.input.hide();
	}

	fn show(&mut self) -> Result<()> {
		self.input.show()?;

		Ok(())
	}
}

impl UpdateRemoteUrlPopup {
	pub fn new(env: &Environment) -> Self {
		Self {
			repo: env.repo.clone(),
			input: TextInputComponent::new(
				env,
				&strings::update_remote_url_popup_title(
					&env.key_config,
				),
				&strings::update_remote_url_popup_msg(
					&env.key_config,
				),
				true,
			)
			.with_input_type(InputType::Singleline),
			key_config: env.key_config.clone(),
			queue: env.queue.clone(),
			theme: env.theme.clone(),
			initial_url: None,
			remote_name: None,
		}
	}

	///
	pub fn open(
		&mut self,
		remote_name: String,
		cur_url: String,
	) -> Result<()> {
		self.input.set_text(cur_url.clone());
		self.remote_name = Some(remote_name);
		self.initial_url = Some(cur_url);
		self.show()?;

		Ok(())
	}

	fn is_valid_url(&self) -> bool {
		!self.input.get_text().trim().is_empty()
	}

	fn draw_warnings(&self, f: &mut Frame) {
		let current_text = self.input.get_text();
		if !current_text.is_empty() && !self.is_valid_url() {
			let msg = strings::remote_url_invalid();
			let msg_length: u16 = msg.len().cast();
			let w =
				Paragraph::new(msg).style(self.theme.text_danger());

			let rect = {
				let mut rect = self.input.get_area();
				rect.y += rect.height.saturating_sub(1);
				rect.height = 1;
				let offset =
					rect.width.saturating_sub(msg_length + 1);
				rect.width = rect.width.saturating_sub(offset + 1);
				rect.x += offset;

				rect
			};

			f.render_widget(w, rect);
		}
	}

	///
	pub fn update_remote_url(&mut self) {
		if !self.is_valid_url() {
			return;
		}

		if self.initial_url.as_deref() == Some(self.input.get_text())
		{
			self.input.clear();
			self.remote_name = None;
			self.initial_url = None;
			self.hide();
			return;
		}

		if let Some(remote_name) = &self.remote_name {
			let res = sync::update_remote_url(
				&self.repo.borrow(),
				remote_name,
				self.input.get_text(),
			);
			match res {
				Ok(()) => {
					self.input.clear();
					self.remote_name = None;
					self.initial_url = None;
					self.hide();
					self.queue.push(InternalEvent::Update(
						NeedsUpdate::ALL | NeedsUpdate::REMOTES,
					));
				}
				Err(e) => {
					log::error!("update remote url: {e}");
					self.queue.push(InternalEvent::ShowErrorMsg(
						format!("update remote url error:\n{e}"),
					));
				}
			}
		}
	}

	#[cfg(test)]
	pub(super) fn set_input_text(&mut self, text: String) {
		self.input.set_text(text);
	}

	#[cfg(test)]
	pub(super) fn input_text(&self) -> &str {
		self.input.get_text()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
	use std::cell::RefCell;

	fn env_for_repo(path: &std::path::Path) -> Environment {
		let mut env = Environment::test_env();
		env.repo =
			RefCell::new(sync::RepoPath::Path(path.to_path_buf()));
		env
	}

	fn enter_event() -> Event {
		Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE))
	}

	#[test]
	fn t8h_url_failure_keeps_input() {
		let (_td, repo) = git2_testing::repo_init();
		let root = repo.path().parent().unwrap();
		let env = env_for_repo(root);

		let mut popup = UpdateRemoteUrlPopup::new(&env);
		popup
			.open(
				"missing".to_string(),
				"https://example.com/old.git".to_string(),
			)
			.unwrap();
		popup
			.input
			.set_text("https://example.com/new.git".to_string());
		popup.event(&enter_event()).unwrap();

		assert!(
			popup.is_visible(),
			"update url popup should stay open on git error"
		);
		assert_eq!(
			popup.input.get_text(),
			"https://example.com/new.git"
		);
		assert_eq!(popup.remote_name.as_deref(), Some("missing"));
		assert_eq!(
			popup.initial_url.as_deref(),
			Some("https://example.com/old.git")
		);
		assert!(matches!(
			env.queue.pop(),
			Some(InternalEvent::ShowErrorMsg(_))
		));
	}
}
