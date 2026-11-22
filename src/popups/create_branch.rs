use crate::components::{
	visibility_blocking, CommandBlocking, CommandInfo, Component,
	DrawableComponent, EventState, InputType, TextInputComponent,
};
use crate::{
	app::Environment,
	keys::{key_match, SharedKeyConfig},
	queue::{InternalEvent, NeedsUpdate, Queue},
	strings,
	ui::style::SharedTheme,
};
use anyhow::Result;
use asyncgit::sync::{self, RepoPathRef};
use crossterm::event::{Event, KeyCode, KeyEvent};
use easy_cast::Cast;
use ratatui::{layout::Rect, widgets::Paragraph, Frame};
use std::borrow::Cow;

pub struct CreateBranchPopup {
	repo: RepoPathRef,
	input: TextInputComponent,
	queue: Queue,
	key_config: SharedKeyConfig,
	theme: SharedTheme,
}

impl DrawableComponent for CreateBranchPopup {
	fn draw(&self, f: &mut Frame, rect: Rect) -> Result<()> {
		if self.is_visible() {
			self.input.draw(f, rect)?;
			self.draw_warnings(f);
		}

		Ok(())
	}
}

impl Component for CreateBranchPopup {
	fn commands(
		&self,
		out: &mut Vec<CommandInfo>,
		force_all: bool,
	) -> CommandBlocking {
		if self.is_visible() || force_all {
			self.input.commands(out, force_all);

			out.push(CommandInfo::new(
				strings::commands::create_branch_confirm_msg(
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
					self.create_branch();
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

impl CreateBranchPopup {
	///
	pub fn new(env: &Environment) -> Self {
		Self {
			queue: env.queue.clone(),
			input: TextInputComponent::new(
				env,
				&strings::create_branch_popup_title(&env.key_config),
				&strings::create_branch_popup_msg(&env.key_config),
				true,
			)
			.with_input_type(InputType::Singleline),
			theme: env.theme.clone(),
			key_config: env.key_config.clone(),
			repo: env.repo.clone(),
		}
	}

	///
	pub fn open(&mut self) -> Result<()> {
		self.input.clear();
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
	pub fn create_branch(&mut self) {
		if !self.is_valid_branch_name() {
			return;
		}

		let res = sync::create_branch(
			&self.repo.borrow(),
			self.input.get_text(),
		);

		match res {
			Ok(_) => {
				self.input.clear();
				self.hide();
				self.queue.push(InternalEvent::Update(
					NeedsUpdate::ALL | NeedsUpdate::BRANCHES,
				));
			}
			Err(e) => {
				log::error!("create branch: {e}");
				self.queue.push(InternalEvent::ShowErrorMsg(
					format!("create branch error:\n{e}"),
				));
			}
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
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::popups::{RenameBranchPopup, TagCommitPopup};
	use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
	use ratatui::{backend::TestBackend, Terminal};
	use std::cell::RefCell;

	fn env_for_repo(path: &std::path::Path) -> Environment {
		let mut env = Environment::test_env();
		env.repo =
			RefCell::new(sync::RepoPath::Path(path.to_path_buf()));
		env
	}

	fn render_popup<C: DrawableComponent>(popup: &C) -> String {
		let backend = TestBackend::new(80, 12);
		let mut terminal = Terminal::new(backend).unwrap();
		terminal
			.draw(|f| {
				popup.draw(f, f.area()).unwrap();
			})
			.unwrap();
		terminal
			.backend()
			.buffer()
			.content()
			.iter()
			.map(ratatui::buffer::Cell::symbol)
			.collect()
	}

	fn enter_event() -> Event {
		Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE))
	}

	fn esc_event() -> Event {
		Event::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE))
	}

	fn confirm_hint_enabled<C: Component>(
		popup: &C,
		confirm_name: &str,
	) -> bool {
		let mut cmds = Vec::new();
		popup.commands(&mut cmds, false);
		cmds.iter()
			.find(|c| c.text.name == confirm_name)
			.map(|c| c.enabled)
			.unwrap()
	}

	#[test]
	fn t8g_invalid_branch_blocked() {
		let (_td, repo) = git2_testing::repo_init();
		let root = repo.path().parent().unwrap();
		let env = env_for_repo(root);
		let create_confirm =
			strings::commands::create_branch_confirm_msg(
				&env.key_config,
			)
			.name;
		let rename_confirm =
			strings::commands::rename_branch_confirm_msg(
				&env.key_config,
			)
			.name;

		// CreateBranchPopup: empty, invalid ref, validator error (NUL)
		let mut create = CreateBranchPopup::new(&env);
		create.open().unwrap();
		for bad in ["", "bad..name", "bad\0name"] {
			create.input.set_text(bad.to_string());
			assert!(
				!confirm_hint_enabled(&create, &create_confirm),
				"create confirm hint should be disabled for {bad:?}"
			);
			create.event(&enter_event()).unwrap();
			assert!(
				create.is_visible(),
				"create popup should stay visible for {bad:?}"
			);
			assert_eq!(create.input.get_text(), bad);
			assert!(
				env.queue.pop().is_none(),
				"create should not queue events for {bad:?}"
			);
			if !bad.is_empty() {
				let rendered = render_popup(&create);
				assert!(
					rendered.contains(strings::branch_name_invalid()),
					"create should show inline warning for {bad:?}, got: {rendered}"
				);
			}
		}
		create.input.set_text("valid-branch".to_string());
		assert!(confirm_hint_enabled(&create, &create_confirm));

		// RenameBranchPopup: empty, invalid ref, validator error (NUL)
		let mut rename = RenameBranchPopup::new(&env);
		rename
			.open(
				"refs/heads/master".to_string(),
				"master".to_string(),
			)
			.unwrap();
		for bad in ["", "bad..name", "bad\0name"] {
			rename.set_input_text(bad.to_string());
			assert!(
				!confirm_hint_enabled(&rename, &rename_confirm),
				"rename confirm hint should be disabled for {bad:?}"
			);
			rename.event(&enter_event()).unwrap();
			assert!(
				rename.is_visible(),
				"rename popup should stay visible for {bad:?}"
			);
			assert_eq!(rename.input_text(), bad);
			assert!(
				env.queue.pop().is_none(),
				"rename should not queue events for {bad:?}"
			);
			if !bad.is_empty() {
				let rendered = render_popup(&rename);
				assert!(
					rendered.contains(strings::branch_name_invalid()),
					"rename should show inline warning for {bad:?}, got: {rendered}"
				);
			}
		}
		rename.set_input_text("valid-rename".to_string());
		assert!(confirm_hint_enabled(&rename, &rename_confirm));
	}

	#[test]
	fn t8g_create_failure_keeps_input() {
		let (_td, repo) = git2_testing::repo_init();
		let root = repo.path().parent().unwrap();
		let env = env_for_repo(root);

		let mut popup = CreateBranchPopup::new(&env);
		popup.open().unwrap();
		// "master" already exists, so sync::create_branch fails
		popup.input.set_text("master".to_string());
		popup.event(&enter_event()).unwrap();

		assert!(
			popup.is_visible(),
			"create popup should stay open on git error"
		);
		assert_eq!(popup.input.get_text(), "master");
		assert!(matches!(
			env.queue.pop(),
			Some(InternalEvent::ShowErrorMsg(_))
		));
	}

	#[test]
	fn t8g_reopen_is_clean() {
		let (_td, repo) = git2_testing::repo_init();
		let root = repo.path().parent().unwrap();
		let head: sync::CommitId =
			repo.head().unwrap().target().unwrap().into();
		let env = env_for_repo(root);

		// CreateBranchPopup: type text, read it, Esc, reopen -> empty
		let mut create = CreateBranchPopup::new(&env);
		create.open().unwrap();
		create.input.set_text("leftover".to_string());
		assert_eq!(create.input.get_text(), "leftover");
		create.event(&esc_event()).unwrap();
		assert!(!create.is_visible());
		create.open().unwrap();
		assert!(create.is_visible());
		assert_eq!(
			create.input.get_text(),
			"",
			"create branch popup should reopen empty after Esc"
		);

		// TagCommitPopup: type text in name mode, trigger get_text(), Esc, reopen -> empty
		let mut tag = TagCommitPopup::new(&env);
		tag.open(head).unwrap();
		tag.event(&Event::Key(KeyEvent::new(
			KeyCode::Char('v'),
			KeyModifiers::NONE,
		)))
		.unwrap();
		tag.event(&Event::Key(KeyEvent::new(
			KeyCode::Char('9'),
			KeyModifiers::NONE,
		)))
		.unwrap();
		tag.commands(&mut Vec::new(), false);
		let _ = render_popup(&tag);
		tag.event(&esc_event()).unwrap();
		assert!(!tag.is_visible());

		tag.open(head).unwrap();
		assert!(tag.is_visible());
		let rendered = render_popup(&tag);
		assert!(
			!rendered.contains("v9"),
			"tag popup should reopen empty after Esc in name mode, got: {rendered}"
		);

		// TagCommitPopup: enter annotation mode, type text, trigger get_text(), Esc, reopen -> empty + name mode
		tag.event(&Event::Key(KeyEvent::new(
			KeyCode::Char('v'),
			KeyModifiers::NONE,
		)))
		.unwrap();
		tag.event(&Event::Key(KeyEvent::new(
			KeyCode::Char('a'),
			KeyModifiers::CONTROL,
		)))
		.unwrap();
		tag.event(&Event::Key(KeyEvent::new(
			KeyCode::Char('x'),
			KeyModifiers::NONE,
		)))
		.unwrap();
		tag.commands(&mut Vec::new(), false);
		let _ = render_popup(&tag);
		tag.event(&esc_event()).unwrap();
		assert!(!tag.is_visible());

		tag.open(head).unwrap();
		assert!(tag.is_visible());
		let mut cmds = Vec::new();
		tag.commands(&mut cmds, false);
		let annotate_name =
			strings::commands::tag_annotate_msg(&env.key_config).name;
		let annotate_available = cmds
			.iter()
			.find(|c| c.text.name == annotate_name)
			.is_some_and(CommandInfo::show_in_quickbar);
		assert!(
			annotate_available,
			"tag popup should reopen in name mode after Esc"
		);
		let rendered = render_popup(&tag);
		assert!(
			!rendered.contains('x') && !rendered.contains("Tag Annotation"),
			"tag popup should reopen empty in name mode, got: {rendered}"
		);
	}
}
