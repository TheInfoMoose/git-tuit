use crate::components::{
	visibility_blocking, CommandBlocking, CommandInfo, Component,
	DrawableComponent, EventState, InputType, TextInputComponent,
};
use crate::{
	app::Environment,
	keys::{key_match, SharedKeyConfig},
	queue::{InternalEvent, NeedsUpdate, Queue},
	strings, try_or_popup,
	ui::style::SharedTheme,
};
use anyhow::Result;
use asyncgit::sync::{
	self, get_config_string, CommitId, RepoPathRef,
};
use crossterm::event::Event;
use easy_cast::Cast;
use ratatui::{layout::Rect, widgets::Paragraph, Frame};

enum Mode {
	Name,
	Annotation { tag_name: String },
}

pub struct TagCommitPopup {
	repo: RepoPathRef,
	mode: Mode,
	input: TextInputComponent,
	commit_id: Option<CommitId>,
	queue: Queue,
	key_config: SharedKeyConfig,
	theme: SharedTheme,
}

impl DrawableComponent for TagCommitPopup {
	fn draw(&self, f: &mut Frame, rect: Rect) -> Result<()> {
		if self.is_visible() {
			self.input.draw(f, rect)?;
			self.draw_warnings(f);
		}

		Ok(())
	}
}

impl Component for TagCommitPopup {
	fn commands(
		&self,
		out: &mut Vec<CommandInfo>,
		force_all: bool,
	) -> CommandBlocking {
		if self.is_visible() || force_all {
			self.input.commands(out, force_all);

			let is_annotation_mode =
				matches!(self.mode, Mode::Annotation { .. });

			out.push(CommandInfo::new(
				strings::commands::tag_commit_confirm_msg(
					&self.key_config,
					is_annotation_mode,
				),
				self.is_valid_tag(),
				true,
			));

			out.push(CommandInfo::new(
				strings::commands::tag_annotate_msg(&self.key_config),
				self.is_valid_tag(),
				matches!(self.mode, Mode::Name),
			));
		}

		visibility_blocking(self)
	}

	fn event(&mut self, ev: &Event) -> Result<EventState> {
		if self.is_visible() {
			if let Event::Key(e) = ev {
				let is_annotation_mode =
					matches!(self.mode, Mode::Annotation { .. });

				if !is_annotation_mode
					&& key_match(e, self.key_config.keys.enter)
				{
					if self.is_valid_tag() {
						try_or_popup!(self, "tag error:", self.tag());
					}
					return Ok(EventState::Consumed);
				}
				if is_annotation_mode
					&& key_match(e, self.key_config.keys.commit)
				{
					if self.is_valid_tag() {
						try_or_popup!(self, "tag error:", self.tag());
					}
					return Ok(EventState::Consumed);
				} else if !is_annotation_mode
					&& key_match(e, self.key_config.keys.tag_annotate)
				{
					if self.is_valid_tag() {
						try_or_popup!(
							self,
							"tag error:",
							self.start_annotate_mode()
						);
					}
					return Ok(EventState::Consumed);
				}
			}

			self.input.event(ev)?;
			return Ok(EventState::Consumed);
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
		self.mode = Mode::Name;
		self.input.clear();
		self.input.set_input_type(InputType::Singleline);
		self.input.set_title(strings::tag_popup_name_title());
		self.input.set_default_msg(strings::tag_popup_name_msg());
		self.input.show()?;

		Ok(())
	}
}

impl TagCommitPopup {
	///
	pub fn new(env: &Environment) -> Self {
		Self {
			queue: env.queue.clone(),
			input: TextInputComponent::new(
				env,
				&strings::tag_popup_name_title(),
				&strings::tag_popup_name_msg(),
				true,
			)
			.with_input_type(InputType::Singleline),
			commit_id: None,
			key_config: env.key_config.clone(),
			repo: env.repo.clone(),
			mode: Mode::Name,
			theme: env.theme.clone(),
		}
	}

	///
	pub fn open(&mut self, id: CommitId) -> Result<()> {
		self.commit_id = Some(id);
		self.show()?;

		Ok(())
	}

	fn is_valid_tag(&self) -> bool {
		let text = self.input.get_text();
		match self.mode {
			Mode::Name => sync::validate_tag_name(text),
			Mode::Annotation { .. } => !text.trim().is_empty(),
		}
	}

	fn tag_info(&self) -> (String, Option<String>) {
		match &self.mode {
			Mode::Name => (self.input.get_text().into(), None),
			Mode::Annotation { tag_name } => {
				(tag_name.clone(), Some(self.input.get_text().into()))
			}
		}
	}

	pub fn tag(&mut self) -> Result<()> {
		if !self.is_valid_tag() {
			return Ok(());
		}

		let gpgsign =
			get_config_string(&self.repo.borrow(), "tag.gpgsign")
				.ok()
				.flatten()
				.and_then(|val| val.parse::<bool>().ok())
				.unwrap_or_default();

		anyhow::ensure!(!gpgsign, "config tag.gpgsign=true detected.\ngpg signing not supported.\ndeactivate in your repo/gitconfig to be able to tag without signing.");

		let (tag_name, tag_annotation) = self.tag_info();

		if let Some(commit_id) = self.commit_id {
			let result = sync::tag_commit(
				&self.repo.borrow(),
				&commit_id,
				&tag_name,
				tag_annotation.as_deref(),
			);
			match result {
				Ok(_) => {
					self.input.clear();
					self.mode = Mode::Name;
					self.commit_id = None;
					self.hide();

					self.queue.push(InternalEvent::Update(
						NeedsUpdate::ALL,
					));
				}
				Err(e) => {
					log::error!("e: {e}");
					self.queue.push(InternalEvent::ShowErrorMsg(
						format!("tag error:\n{e}"),
					));
				}
			}
		}

		Ok(())
	}

	fn start_annotate_mode(&mut self) -> Result<()> {
		if !matches!(self.mode, Mode::Name) || !self.is_valid_tag() {
			return Ok(());
		}

		let tag_name: String = self.input.get_text().into();
		if sync::tag_exists(&self.repo.borrow(), &tag_name)? {
			anyhow::bail!("tag '{tag_name}' already exists");
		}

		self.input.clear();
		self.input.set_input_type(InputType::Multiline);
		self.input.set_title(strings::tag_popup_annotation_title(
			&tag_name,
		));
		self.input
			.set_default_msg(strings::tag_popup_annotation_msg());
		self.mode = Mode::Annotation { tag_name };

		Ok(())
	}

	fn draw_warnings(&self, f: &mut Frame) {
		let current_text = self.input.get_text();
		if current_text.is_empty() || self.is_valid_tag() {
			return;
		}

		let msg = match self.mode {
			Mode::Name => strings::tag_name_invalid(),
			Mode::Annotation { .. } => {
				strings::tag_annotation_invalid()
			}
		};
		let msg_length: u16 = msg.len().cast();
		let w = Paragraph::new(msg).style(self.theme.text_danger());

		let rect = {
			let mut rect = self.input.get_area();
			rect.y += rect.height.saturating_sub(1);
			rect.height = 1;
			let offset = rect.width.saturating_sub(msg_length + 1);
			rect.width = rect.width.saturating_sub(offset + 1);
			rect.x += offset;

			rect
		};

		f.render_widget(w, rect);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
	use ratatui::{backend::TestBackend, Terminal};
	use std::cell::RefCell;

	fn env_for_repo(path: &std::path::Path) -> Environment {
		let mut env = Environment::test_env();
		env.repo =
			RefCell::new(sync::RepoPath::Path(path.to_path_buf()));
		env
	}

	fn render_popup(popup: &TagCommitPopup) -> String {
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

	fn annotate_event() -> Event {
		Event::Key(KeyEvent::new(
			KeyCode::Char('a'),
			KeyModifiers::CONTROL,
		))
	}

	fn commit_event() -> Event {
		Event::Key(KeyEvent::new(
			KeyCode::Char('d'),
			KeyModifiers::CONTROL,
		))
	}

	fn hint_enabled(popup: &TagCommitPopup, name: &str) -> bool {
		let mut cmds = Vec::new();
		popup.commands(&mut cmds, false);
		cmds.iter()
			.find(|c| c.text.name == name)
			.map(|c| c.enabled)
			.unwrap()
	}

	#[test]
	fn t8g_invalid_tag_blocked() {
		let (_td, repo) = git2_testing::repo_init();
		let root = repo.path().parent().unwrap();
		let head: CommitId =
			repo.head().unwrap().target().unwrap().into();
		let env = env_for_repo(root);
		let confirm_name = strings::commands::tag_commit_confirm_msg(
			&env.key_config,
			false,
		)
		.name;
		let annotate_name =
			strings::commands::tag_annotate_msg(&env.key_config).name;

		let mut popup = TagCommitPopup::new(&env);
		popup.open(head).unwrap();

		for bad in ["", "   ", "v 1", "v1\0bad", "v1..2"] {
			popup.input.set_text(bad.to_string());
			assert!(
				!hint_enabled(&popup, &confirm_name),
				"tag confirm hint should be disabled for {bad:?}"
			);
			assert!(
				!hint_enabled(&popup, &annotate_name),
				"tag annotate hint should be disabled for {bad:?}"
			);

			popup.event(&enter_event()).unwrap();
			assert!(popup.is_visible());
			assert_eq!(popup.input.get_text(), bad);
			assert!(matches!(popup.mode, Mode::Name));
			assert!(env.queue.pop().is_none());

			popup.event(&annotate_event()).unwrap();
			assert!(popup.is_visible());
			assert_eq!(popup.input.get_text(), bad);
			assert!(
				matches!(popup.mode, Mode::Name),
				"annotate should be blocked for {bad:?}"
			);
			assert!(env.queue.pop().is_none());

			if !bad.is_empty() {
				let rendered = render_popup(&popup);
				assert!(
					rendered.contains("[invalid name]"),
					"inline warning should render for {bad:?}, got: {rendered}"
				);
			}
		}

		popup.input.set_text("v1.0".to_string());
		assert!(hint_enabled(&popup, &confirm_name));
		assert!(hint_enabled(&popup, &annotate_name));
	}

	#[test]
	fn t8g_empty_annotation_blocked() {
		let (_td, repo) = git2_testing::repo_init();
		let root = repo.path().parent().unwrap();
		let head: CommitId =
			repo.head().unwrap().target().unwrap().into();
		let env = env_for_repo(root);
		let confirm_name = strings::commands::tag_commit_confirm_msg(
			&env.key_config,
			true,
		)
		.name;

		let mut popup = TagCommitPopup::new(&env);
		popup.open(head).unwrap();
		popup.input.set_text("v1.0".to_string());
		popup.event(&annotate_event()).unwrap();
		assert!(matches!(popup.mode, Mode::Annotation { .. }));

		for bad in ["", "   ", " \n\t "] {
			popup.input.set_text(bad.to_string());
			assert!(
				!hint_enabled(&popup, &confirm_name),
				"annotation confirm hint should be disabled for {bad:?}"
			);
			popup.event(&commit_event()).unwrap();
			assert!(
				popup.is_visible(),
				"empty/whitespace annotation {bad:?} should stay open"
			);
			assert!(matches!(
				&popup.mode,
				Mode::Annotation { tag_name } if tag_name == "v1.0"
			));
			assert_eq!(popup.input.get_text(), bad);
			assert!(env.queue.pop().is_none());
		}

		// Multiline annotation still works
		popup.input.set_text("line 1\nline 2".to_string());
		assert!(hint_enabled(&popup, &confirm_name));
		popup.event(&commit_event()).unwrap();
		assert!(!popup.is_visible());
		let tags = sync::get_tags(&env.repo.borrow()).unwrap();
		assert_eq!(
			tags[&head][0].annotation.as_deref(),
			Some("line 1\nline 2")
		);
	}

	#[test]
	fn t8g_existing_tag_not_annotated() {
		let (_td, repo) = git2_testing::repo_init();
		let root = repo.path().parent().unwrap();
		let head: CommitId =
			repo.head().unwrap().target().unwrap().into();
		let env = env_for_repo(root);

		sync::tag_commit(&env.repo.borrow(), &head, "v1.0", None)
			.unwrap();

		let mut popup = TagCommitPopup::new(&env);
		popup.open(head).unwrap();
		popup.input.set_text("v1.0".to_string());
		popup.event(&annotate_event()).unwrap();

		assert!(
			matches!(popup.mode, Mode::Name),
			"existing tag should not switch to annotation mode"
		);
		assert!(popup.is_visible());
		assert_eq!(popup.input.get_text(), "v1.0");
	}

	#[test]
	fn t8g_tag_failure_keeps_input() {
		let (_td, repo) = git2_testing::repo_init();
		let root = repo.path().parent().unwrap();
		let head: CommitId =
			repo.head().unwrap().target().unwrap().into();
		let env = env_for_repo(root);

		sync::tag_commit(&env.repo.borrow(), &head, "v1.0", None)
			.unwrap();

		// 1. Name mode failure (tag already exists)
		let mut popup = TagCommitPopup::new(&env);
		popup.open(head).unwrap();
		popup.input.set_text("v1.0".to_string());
		popup.event(&enter_event()).unwrap();

		assert!(popup.is_visible());
		assert_eq!(popup.input.get_text(), "v1.0");
		assert!(matches!(popup.mode, Mode::Name));
		assert_eq!(popup.commit_id, Some(head));
		assert!(matches!(
			env.queue.pop(),
			Some(InternalEvent::ShowErrorMsg(_))
		));

		// 2. Annotation mode failure (git error: tag created before submit)
		popup.open(head).unwrap();
		popup.input.set_text("v2.0".to_string());
		popup.event(&annotate_event()).unwrap();
		popup.input.set_text("release notes\nline 2".to_string());
		sync::tag_commit(&env.repo.borrow(), &head, "v2.0", None)
			.unwrap();
		popup.event(&commit_event()).unwrap();

		assert!(popup.is_visible());
		assert_eq!(popup.input.get_text(), "release notes\nline 2");
		assert!(matches!(
			&popup.mode,
			Mode::Annotation { tag_name } if tag_name == "v2.0"
		));
		assert_eq!(popup.commit_id, Some(head));
		assert!(matches!(
			env.queue.pop(),
			Some(InternalEvent::ShowErrorMsg(_))
		));

		// 3. Annotation mode failure via tag.gpgsign=true refusal
		repo.config()
			.unwrap()
			.set_bool("tag.gpgsign", true)
			.unwrap();
		popup.open(head).unwrap();
		popup.input.set_text("v3.0".to_string());
		popup.event(&annotate_event()).unwrap();
		popup.input.set_text("signed note".to_string());
		popup.event(&commit_event()).unwrap();

		assert!(popup.is_visible());
		assert_eq!(popup.input.get_text(), "signed note");
		assert!(matches!(
			&popup.mode,
			Mode::Annotation { tag_name } if tag_name == "v3.0"
		));
		assert_eq!(popup.commit_id, Some(head));
		assert!(matches!(
			env.queue.pop(),
			Some(InternalEvent::ShowErrorMsg(msg))
				if msg.contains("tag.gpgsign=true")
		));
	}

	#[test]
	fn t8g_annotate_twice_keeps_name() {
		let (_td, repo) = git2_testing::repo_init();
		let root = repo.path().parent().unwrap();
		let head: CommitId =
			repo.head().unwrap().target().unwrap().into();
		let env = env_for_repo(root);

		let mut popup = TagCommitPopup::new(&env);
		popup.open(head).unwrap();
		popup.input.set_text("v1.0".to_string());
		popup.event(&annotate_event()).unwrap();
		popup.input.set_text("my annotation".to_string());

		// Pressing annotate key again in annotation mode must not replace tag_name
		popup.event(&annotate_event()).unwrap();

		assert!(matches!(
			&popup.mode,
			Mode::Annotation { tag_name } if tag_name == "v1.0"
		));
		assert_eq!(popup.input.get_text(), "my annotation");
	}
}
