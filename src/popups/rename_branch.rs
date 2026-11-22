use crate::components::{
	visibility_blocking, CommandBlocking, CommandInfo, Component,
	DrawableComponent, EventState, InputType, TextInputComponent,
};
use crate::ui::style::SharedTheme;
use crate::{
	app::Environment,
	keys::{key_match, SharedKeyConfig},
	queue::{InternalEvent, NeedsUpdate, Queue},
	strings,
};
use anyhow::Result;
use asyncgit::sync::{self, RepoPathRef};
use crossterm::event::{Event, KeyCode, KeyEvent};
use easy_cast::Cast;
use ratatui::{layout::Rect, widgets::Paragraph, Frame};
use std::borrow::Cow;

pub struct RenameBranchPopup {
	repo: RepoPathRef,
	input: TextInputComponent,
	branch_ref: Option<String>,
	cur_name: Option<String>,
	queue: Queue,
	key_config: SharedKeyConfig,
	theme: SharedTheme,
}

impl DrawableComponent for RenameBranchPopup {
	fn draw(&self, f: &mut Frame, rect: Rect) -> Result<()> {
		if self.is_visible() {
			self.input.draw(f, rect)?;
			self.draw_warnings(f);
		}
		Ok(())
	}
}

impl Component for RenameBranchPopup {
	fn commands(
		&self,
		out: &mut Vec<CommandInfo>,
		force_all: bool,
	) -> CommandBlocking {
		if self.is_visible() || force_all {
			self.input.commands(out, force_all);

			out.push(CommandInfo::new(
				strings::commands::rename_branch_confirm_msg(
					&self.key_config,
				),
				self.is_valid_branch_name(),
				true,
			));
		}

		visibility_blocking(self)
	}

	fn event(&mut self, ev: &Event) -> Result<EventState> {
		if self.is_visible() {
			let ev = match ev {
				Event::Key(KeyEvent {
					code: KeyCode::Char(c),
					modifiers,
					kind,
					state,
				}) => Cow::Owned(Event::Key(KeyEvent {
					code: KeyCode::Char(
						strings::normalize_branch_name_char(*c),
					),
					modifiers: *modifiers,
					kind: *kind,
					state: *state,
				})),
				_ => Cow::Borrowed(ev),
			};

			if self.input.event(&ev)?.is_consumed() {
				return Ok(EventState::Consumed);
			}

			if let Event::Key(e) = ev.as_ref() {
				if key_match(e, self.key_config.keys.enter) {
					self.rename_branch();
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

impl RenameBranchPopup {
	///
	pub fn new(env: &Environment) -> Self {
		Self {
			repo: env.repo.clone(),
			queue: env.queue.clone(),
			input: TextInputComponent::new(
				env,
				&strings::rename_branch_popup_title(&env.key_config),
				&strings::rename_branch_popup_msg(&env.key_config),
				true,
			)
			.with_input_type(InputType::Singleline),
			branch_ref: None,
			cur_name: None,
			key_config: env.key_config.clone(),
			theme: env.theme.clone(),
		}
	}

	///
	pub fn open(
		&mut self,
		branch_ref: String,
		cur_name: String,
	) -> Result<()> {
		self.branch_ref = Some(branch_ref);
		self.cur_name = Some(cur_name.clone());
		self.input.set_text(cur_name);
		self.show()?;

		Ok(())
	}

	fn is_valid_branch_name(&self) -> bool {
		let current_text = self.input.get_text();
		!current_text.is_empty()
			&& sync::validate_branch_name(current_text)
				.unwrap_or_default()
	}

	///
	pub fn rename_branch(&mut self) {
		if !self.is_valid_branch_name() {
			return;
		}

		if self.cur_name.as_deref() == Some(self.input.get_text()) {
			self.input.clear();
			self.branch_ref = None;
			self.cur_name = None;
			self.hide();
			return;
		}

		if let Some(br) = &self.branch_ref {
			let res = sync::rename_branch(
				&self.repo.borrow(),
				br,
				self.input.get_text(),
			);

			match res {
				Ok(()) => {
					self.input.clear();
					self.branch_ref = None;
					self.cur_name = None;
					self.queue.push(InternalEvent::Update(
						NeedsUpdate::ALL,
					));
					self.hide();
					self.queue.push(InternalEvent::SelectBranch);
				}
				Err(e) => {
					log::error!("rename branch: {e}");
					self.queue.push(InternalEvent::ShowErrorMsg(
						format!("rename branch error:\n{e}"),
					));
				}
			}
		} else {
			log::error!("rename branch: No branch selected");
			self.queue.push(InternalEvent::ShowErrorMsg(
				"rename branch error: No branch selected to rename"
					.to_string(),
			));
		}
	}

	fn draw_warnings(&self, f: &mut Frame) {
		let current_text = self.input.get_text();

		if !current_text.is_empty() {
			let valid = sync::validate_branch_name(current_text)
				.unwrap_or_default();

			if !valid {
				let msg = strings::branch_name_invalid();
				let msg_length: u16 = msg.len().cast();
				let w = Paragraph::new(msg)
					.style(self.theme.text_danger());

				let rect = {
					let mut rect = self.input.get_area();
					rect.y += rect.height.saturating_sub(1);
					rect.height = 1;
					let offset =
						rect.width.saturating_sub(msg_length + 1);
					rect.width =
						rect.width.saturating_sub(offset + 1);
					rect.x += offset;

					rect
				};

				f.render_widget(w, rect);
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

	fn esc_event() -> Event {
		Event::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE))
	}

	#[test]
	fn t8g_unchanged_rename_closes() {
		let (_td, repo) = git2_testing::repo_init();
		let root = repo.path().parent().unwrap();
		let env = env_for_repo(root);

		// Use a non-existent ref so any git call would fail with an error.
		let mut popup = RenameBranchPopup::new(&env);
		popup
			.open(
				"refs/heads/nonexistent".to_string(),
				"master".to_string(),
			)
			.unwrap();
		popup.event(&enter_event()).unwrap();

		assert!(
			!popup.is_visible(),
			"unchanged rename should close the popup"
		);
		assert!(
			env.queue.pop().is_none(),
			"unchanged rename should make no git call and emit no error"
		);
	}

	#[test]
	fn t8g_rename_failure_keeps_input() {
		let (_td, repo) = git2_testing::repo_init();
		let root = repo.path().parent().unwrap();
		let env = env_for_repo(root);

		let mut popup = RenameBranchPopup::new(&env);
		popup
			.open(
				"refs/heads/missing-branch".to_string(),
				"missing-branch".to_string(),
			)
			.unwrap();
		popup.input.set_text("new-name".to_string());
		popup.event(&enter_event()).unwrap();

		assert!(
			popup.is_visible(),
			"rename popup should stay open on git error"
		);
		assert_eq!(popup.input.get_text(), "new-name");
		assert_eq!(
			popup.branch_ref.as_deref(),
			Some("refs/heads/missing-branch")
		);
		assert!(matches!(
			env.queue.pop(),
			Some(InternalEvent::ShowErrorMsg(_))
		));
	}

	#[test]
	fn t8g_rename_opens_with_name() {
		let (_td, repo) = git2_testing::repo_init();
		let root = repo.path().parent().unwrap();
		let env = env_for_repo(root);

		let mut popup = RenameBranchPopup::new(&env);
		popup
			.open(
				"refs/heads/master".to_string(),
				"master".to_string(),
			)
			.unwrap();
		popup.input.set_text("edited".to_string());
		popup.event(&esc_event()).unwrap();
		assert!(!popup.is_visible());

		popup
			.open("refs/heads/feat".to_string(), "feat".to_string())
			.unwrap();
		assert!(popup.is_visible());
		assert_eq!(popup.input.get_text(), "feat");
	}
}
