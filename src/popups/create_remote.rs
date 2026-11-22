use anyhow::Result;
use asyncgit::sync::{self, validate_remote_name, RepoPathRef};
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

#[derive(Default)]
enum State {
	// first we ask for a name for a new remote
	#[default]
	Name,
	// second we ask for a url and carry with us the name previously entered
	Url {
		name: String,
	},
}

pub struct CreateRemotePopup {
	repo: RepoPathRef,
	input: TextInputComponent,
	queue: Queue,
	key_config: SharedKeyConfig,
	state: State,
	existing_remotes: Vec<String>,
	theme: SharedTheme,
}

impl DrawableComponent for CreateRemotePopup {
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

impl Component for CreateRemotePopup {
	fn commands(
		&self,
		out: &mut Vec<CommandInfo>,
		force_all: bool,
	) -> CommandBlocking {
		if self.is_visible() || force_all {
			self.input.commands(out, force_all);

			let confirm_msg = match self.state {
				State::Name => {
					strings::commands::remote_confirm_name_msg(
						&self.key_config,
					)
				}
				State::Url { .. } => {
					strings::commands::remote_confirm_url_msg(
						&self.key_config,
					)
				}
			};
			out.push(CommandInfo::new(
				confirm_msg,
				self.is_valid_input(),
				true,
			));
		}
		visibility_blocking(self)
	}

	fn event(
		&mut self,
		ev: &crossterm::event::Event,
	) -> Result<EventState> {
		if self.is_visible() {
			if self.input.event(ev)?.is_consumed() {
				return Ok(EventState::Consumed);
			}

			if let Event::Key(e) = ev {
				if key_match(e, self.key_config.keys.enter) {
					self.handle_submit();
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
		self.input.clear();
		self.input.set_title(
			strings::create_remote_popup_title_name(&self.key_config),
		);
		self.input.set_default_msg(
			strings::create_remote_popup_msg_name(&self.key_config),
		);

		self.input.show()?;

		Ok(())
	}
}

impl CreateRemotePopup {
	pub fn new(env: &Environment) -> Self {
		Self {
			repo: env.repo.clone(),
			queue: env.queue.clone(),
			input: TextInputComponent::new(env, "", "", true)
				.with_input_type(InputType::Singleline),
			key_config: env.key_config.clone(),
			state: State::Name,
			existing_remotes: Vec::new(),
			theme: env.theme.clone(),
		}
	}

	pub fn open(&mut self) -> Result<()> {
		self.state = State::Name;
		self.existing_remotes =
			sync::get_remotes(&self.repo.borrow())?;
		self.input.clear();
		self.show()?;

		Ok(())
	}

	fn is_valid_input(&self) -> bool {
		let text = self.input.get_text();
		match self.state {
			State::Name => {
				validate_remote_name(text)
					&& !self
						.existing_remotes
						.iter()
						.any(|r| r == text)
			}
			State::Url { .. } => !text.trim().is_empty(),
		}
	}

	fn draw_warnings(&self, f: &mut Frame) {
		let text = self.input.get_text();
		if text.is_empty() {
			return;
		}

		let msg = match self.state {
			State::Name => {
				if !validate_remote_name(text) {
					strings::remote_name_invalid()
				} else if self
					.existing_remotes
					.iter()
					.any(|r| r == text)
				{
					strings::remote_name_exists()
				} else {
					return;
				}
			}
			State::Url { .. } => {
				if text.trim().is_empty() {
					strings::remote_url_invalid()
				} else {
					return;
				}
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

	fn handle_submit(&mut self) {
		if !self.is_valid_input() {
			return;
		}

		match &self.state {
			State::Name => {
				let name = self.input.get_text().to_string();
				if let Ok(remotes) =
					sync::get_remotes(&self.repo.borrow())
				{
					self.existing_remotes = remotes;
					if self
						.existing_remotes
						.iter()
						.any(|r| r == &name)
					{
						self.queue.push(InternalEvent::ShowErrorMsg(
							format!("remote '{name}' already exists"),
						));
						return;
					}
				}

				self.state = State::Url { name };

				self.input.clear();
				self.input.set_title(
					strings::create_remote_popup_title_url(
						&self.key_config,
					),
				);
				self.input.set_default_msg(
					strings::create_remote_popup_msg_url(
						&self.key_config,
					),
				);
			}
			State::Url { name } => {
				let res = sync::add_remote(
					&self.repo.borrow(),
					name,
					self.input.get_text(),
				);

				match res {
					Ok(()) => {
						self.state = State::Name;
						self.existing_remotes.clear();
						self.input.clear();
						self.hide();
						self.queue.push(InternalEvent::Update(
							NeedsUpdate::ALL | NeedsUpdate::REMOTES,
						));
					}
					Err(e) => {
						log::error!("create remote: {e}");
						self.queue.push(InternalEvent::ShowErrorMsg(
							format!("create remote error:\n{e}"),
						));
					}
				}
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::popups::{RenameRemotePopup, UpdateRemoteUrlPopup};
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
	fn t8h_invalid_name_blocked() {
		let (_td, repo) = git2_testing::repo_init();
		let root = repo.path().parent().unwrap();
		let env = env_for_repo(root);
		let confirm_name =
			strings::commands::remote_confirm_name_msg(
				&env.key_config,
			)
			.name;

		// CreateRemotePopup: empty, invalid name, NUL byte
		let mut create = CreateRemotePopup::new(&env);
		create.open().unwrap();
		for bad in ["", "bad..name", "bad name", "bad\0name"] {
			create.input.set_text(bad.to_string());
			assert!(
				!confirm_hint_enabled(&create, &confirm_name),
				"create confirm hint should be disabled for {bad:?}"
			);
			create.event(&enter_event()).unwrap();
			assert!(
				create.is_visible(),
				"create popup should stay visible for {bad:?}"
			);
			assert!(
				matches!(create.state, State::Name),
				"create popup should stay at Name stage for {bad:?}"
			);
			assert_eq!(create.input.get_text(), bad);
			assert!(
				env.queue.pop().is_none(),
				"create should not queue events for {bad:?}"
			);
			if !bad.is_empty() {
				let rendered = render_popup(&create);
				assert!(
					rendered.contains(strings::remote_name_invalid()),
					"create should show remote-name warning for {bad:?}, got: {rendered}"
				);
			}
		}
		create.input.set_text("upstream".to_string());
		assert!(confirm_hint_enabled(&create, &confirm_name));

		// RenameRemotePopup: empty, invalid name, NUL byte
		let mut rename = RenameRemotePopup::new(&env);
		rename.open("origin".to_string()).unwrap();
		for bad in ["", "bad..name", "bad name", "bad\0name"] {
			rename.set_input_text(bad.to_string());
			assert!(
				!confirm_hint_enabled(&rename, &confirm_name),
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
					rendered.contains(strings::remote_name_invalid()),
					"rename should show remote-name warning for {bad:?}, got: {rendered}"
				);
			}
		}
		rename.set_input_text("upstream".to_string());
		assert!(confirm_hint_enabled(&rename, &confirm_name));
	}

	#[test]
	fn t8h_existing_name_blocked() {
		let (_td, repo) = git2_testing::repo_init();
		let root = repo.path().parent().unwrap();
		repo.remote("origin", "https://example.com/repo.git")
			.unwrap();
		let env = env_for_repo(root);
		let confirm_name =
			strings::commands::remote_confirm_name_msg(
				&env.key_config,
			)
			.name;

		let mut create = CreateRemotePopup::new(&env);
		create.open().unwrap();
		create.input.set_text("origin".to_string());
		assert!(
			!confirm_hint_enabled(&create, &confirm_name),
			"confirm hint should be disabled for existing remote name"
		);
		create.event(&enter_event()).unwrap();
		assert!(create.is_visible());
		assert!(
			matches!(create.state, State::Name),
			"existing remote name should be blocked at Name stage"
		);
		assert_eq!(create.input.get_text(), "origin");
	}

	#[test]
	fn t8h_empty_url_blocked() {
		let (_td, repo) = git2_testing::repo_init();
		let root = repo.path().parent().unwrap();
		repo.remote("origin", "https://example.com/repo.git")
			.unwrap();
		let env = env_for_repo(root);
		let confirm_url = strings::commands::remote_confirm_url_msg(
			&env.key_config,
		)
		.name;

		// CreateRemotePopup at URL stage
		let mut create = CreateRemotePopup::new(&env);
		create.open().unwrap();
		create.input.set_text("upstream".to_string());
		create.event(&enter_event()).unwrap();
		assert!(matches!(
			&create.state,
			State::Url { name } if name == "upstream"
		));

		for bad in ["", "   ", "\t "] {
			create.input.set_text(bad.to_string());
			assert!(
				!confirm_hint_enabled(&create, &confirm_url),
				"create url confirm hint should be disabled for {bad:?}"
			);
			create.event(&enter_event()).unwrap();
			assert!(
				create.is_visible(),
				"create popup should stay visible for {bad:?}"
			);
			assert!(matches!(
				&create.state,
				State::Url { name } if name == "upstream"
			));
			assert_eq!(create.input.get_text(), bad);
			assert!(
				env.queue.pop().is_none(),
				"create should not queue events for {bad:?}"
			);
		}

		// UpdateRemoteUrlPopup
		let mut update = UpdateRemoteUrlPopup::new(&env);
		update
			.open(
				"origin".to_string(),
				"https://example.com/repo.git".to_string(),
			)
			.unwrap();
		for bad in ["", "   ", "\t "] {
			update.set_input_text(bad.to_string());
			assert!(
				!confirm_hint_enabled(&update, &confirm_url),
				"update url confirm hint should be disabled for {bad:?}"
			);
			update.event(&enter_event()).unwrap();
			assert!(
				update.is_visible(),
				"update popup should stay visible for {bad:?}"
			);
			assert_eq!(update.input_text(), bad);
			assert!(
				env.queue.pop().is_none(),
				"update should not queue events for {bad:?}"
			);
		}
	}

	#[test]
	fn t8h_path_url_accepted() {
		let (_td, repo) = git2_testing::repo_init();
		let root = repo.path().parent().unwrap();
		let env = env_for_repo(root);
		let confirm_url = strings::commands::remote_confirm_url_msg(
			&env.key_config,
		)
		.name;
		let local_path = "/tmp/my remote repo";
		let updated_path = "/tmp/other remote repo";

		// Create with local path containing spaces
		let mut create = CreateRemotePopup::new(&env);
		create.open().unwrap();
		create.input.set_text("local".to_string());
		create.event(&enter_event()).unwrap();
		create.input.set_text(local_path.to_string());
		assert!(confirm_hint_enabled(&create, &confirm_url));
		create.event(&enter_event()).unwrap();

		assert!(!create.is_visible());
		assert_eq!(
			sync::get_remote_url(&env.repo.borrow(), "local")
				.unwrap()
				.as_deref(),
			Some(local_path)
		);
		assert!(matches!(
			env.queue.pop(),
			Some(InternalEvent::Update(flags))
				if flags.contains(NeedsUpdate::REMOTES)
		));

		// Update with another local path containing spaces
		let mut update = UpdateRemoteUrlPopup::new(&env);
		update
			.open("local".to_string(), local_path.to_string())
			.unwrap();
		update.set_input_text(updated_path.to_string());
		assert!(confirm_hint_enabled(&update, &confirm_url));
		update.event(&enter_event()).unwrap();

		assert!(!update.is_visible());
		assert_eq!(
			sync::get_remote_url(&env.repo.borrow(), "local")
				.unwrap()
				.as_deref(),
			Some(updated_path)
		);
		assert!(matches!(
			env.queue.pop(),
			Some(InternalEvent::Update(flags))
				if flags.contains(NeedsUpdate::REMOTES)
		));
	}

	#[test]
	fn t8h_create_failure_keeps_input() {
		let (_td, repo) = git2_testing::repo_init();
		let root = repo.path().parent().unwrap();
		let env = env_for_repo(root);

		let mut create = CreateRemotePopup::new(&env);
		create.open().unwrap();
		create.input.set_text("upstream".to_string());
		create.event(&enter_event()).unwrap();
		assert!(matches!(
			&create.state,
			State::Url { name } if name == "upstream"
		));
		create
			.input
			.set_text("https://example.com/upstream.git".to_string());

		// Create "upstream" in git before submitting so add_remote fails
		repo.remote("upstream", "https://example.com/existing.git")
			.unwrap();

		create.event(&enter_event()).unwrap();

		assert!(
			create.is_visible(),
			"create popup should stay open on git error"
		);
		assert!(matches!(
			&create.state,
			State::Url { name } if name == "upstream"
		));
		assert_eq!(
			create.input.get_text(),
			"https://example.com/upstream.git"
		);
		assert!(matches!(
			env.queue.pop(),
			Some(InternalEvent::ShowErrorMsg(_))
		));
	}

	#[test]
	fn t8h_reopen_is_clean() {
		let (_td, repo) = git2_testing::repo_init();
		let root = repo.path().parent().unwrap();
		let env = env_for_repo(root);

		// CreateRemotePopup: Esc from URL stage, reopen -> empty Name stage
		let mut create = CreateRemotePopup::new(&env);
		create.open().unwrap();
		create.input.set_text("upstream".to_string());
		create.event(&enter_event()).unwrap();
		create
			.input
			.set_text("https://example.com/draft.git".to_string());
		create.event(&esc_event()).unwrap();
		assert!(!create.is_visible());

		create.open().unwrap();
		assert!(create.is_visible());
		assert!(matches!(create.state, State::Name));
		assert_eq!(create.input.get_text(), "");

		// RenameRemotePopup: edit, Esc, reopen for another remote -> current name
		let mut rename = RenameRemotePopup::new(&env);
		rename.open("origin".to_string()).unwrap();
		rename.set_input_text("edited".to_string());
		rename.event(&esc_event()).unwrap();
		assert!(!rename.is_visible());

		rename.open("upstream".to_string()).unwrap();
		assert!(rename.is_visible());
		assert_eq!(rename.input_text(), "upstream");

		// UpdateRemoteUrlPopup: edit, Esc, reopen for another remote -> current URL
		let mut update = UpdateRemoteUrlPopup::new(&env);
		update
			.open(
				"origin".to_string(),
				"https://example.com/old.git".to_string(),
			)
			.unwrap();
		update.set_input_text(
			"https://example.com/draft.git".to_string(),
		);
		update.event(&esc_event()).unwrap();
		assert!(!update.is_visible());

		update
			.open(
				"upstream".to_string(),
				"https://example.com/upstream.git".to_string(),
			)
			.unwrap();
		assert!(update.is_visible());
		assert_eq!(
			update.input_text(),
			"https://example.com/upstream.git"
		);
	}

	#[test]
	fn t8h_confirm_hint_follows_stage() {
		let (_td, repo) = git2_testing::repo_init();
		let root = repo.path().parent().unwrap();
		let env = env_for_repo(root);
		let confirm_name =
			strings::commands::remote_confirm_name_msg(
				&env.key_config,
			)
			.name;
		let confirm_url = strings::commands::remote_confirm_url_msg(
			&env.key_config,
		)
		.name;

		let mut create = CreateRemotePopup::new(&env);
		create.open().unwrap();

		let mut cmds = Vec::new();
		create.commands(&mut cmds, false);
		assert!(
			cmds.iter().any(|c| c.text.name == confirm_name),
			"Name stage should show confirm-name hint"
		);
		assert!(
			!cmds.iter().any(|c| c.text.name == confirm_url),
			"Name stage should not show confirm-url hint"
		);

		create.input.set_text("upstream".to_string());
		create.event(&enter_event()).unwrap();
		assert!(matches!(create.state, State::Url { .. }));

		cmds.clear();
		create.commands(&mut cmds, false);
		assert!(
			cmds.iter().any(|c| c.text.name == confirm_url),
			"URL stage should show confirm-url hint"
		);
		assert!(
			!cmds.iter().any(|c| c.text.name == confirm_name),
			"URL stage should not show confirm-name hint"
		);
	}
}
