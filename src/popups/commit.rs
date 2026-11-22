use crate::components::{
	visibility_blocking, CommandBlocking, CommandInfo, Component,
	DrawableComponent, EventState, TextInputComponent,
};
use crate::{
	app::Environment,
	keys::{key_match, SharedKeyConfig},
	options::SharedOptions,
	queue::{InternalEvent, NeedsUpdate, Queue},
	strings, try_or_popup,
	ui::style::SharedTheme,
};
use anyhow::{bail, Ok, Result};
use asyncgit::sync::commit::commit_message_prettify;
use asyncgit::{
	cached,
	sync::{
		self, get_config_string, CommitId, HookResult,
		PrepareCommitMsgSource, RepoPathRef, RepoState,
	},
	StatusItem, StatusItemType,
};
use crossterm::event::Event;
use easy_cast::Cast;
use ratatui::{
	layout::{Alignment, Rect},
	widgets::Paragraph,
	Frame,
};

use std::{
	fmt::Write as _,
	fs::{read_to_string, File},
	io::{Read, Write},
	path::PathBuf,
	str::FromStr,
};

use super::ExternalEditorPopup;

enum CommitResult {
	CommitDone,
	Aborted,
}

enum Mode {
	Normal,
	Amend(CommitId),
	Merge(Vec<CommitId>),
	Revert,
	Reword(CommitId),
}

pub struct CommitPopup {
	repo: RepoPathRef,
	input: TextInputComponent,
	mode: Mode,
	queue: Queue,
	key_config: SharedKeyConfig,
	git_branch_name: cached::BranchName,
	commit_template: Option<String>,
	theme: SharedTheme,
	commit_msg_history_idx: usize,
	options: SharedOptions,
	verify: bool,
}

const FIRST_LINE_LIMIT: usize = 50;

impl CommitPopup {
	///
	pub fn new(env: &Environment) -> Self {
		Self {
			queue: env.queue.clone(),
			mode: Mode::Normal,
			input: TextInputComponent::new(
				env,
				"",
				&strings::commit_msg(&env.key_config),
				true,
			),
			key_config: env.key_config.clone(),
			git_branch_name: cached::BranchName::new(
				env.repo.clone(),
			),
			commit_template: None,
			theme: env.theme.clone(),
			repo: env.repo.clone(),
			commit_msg_history_idx: 0,
			options: env.options.clone(),
			verify: true,
		}
	}

	///
	pub fn update(&mut self) {
		self.git_branch_name.lookup().ok();
	}

	#[cfg(test)]
	pub(crate) fn text(&self) -> &str {
		self.input.get_text()
	}

	fn draw_branch_name(&self, f: &mut Frame) {
		if let Some(name) = self.git_branch_name.last() {
			let w = Paragraph::new(format!("{{{name}}}"))
				.alignment(Alignment::Right);

			let rect = {
				let mut rect = self.input.get_area();
				rect.height = 1;
				rect.width = rect.width.saturating_sub(1);
				rect
			};

			f.render_widget(w, rect);
		}
	}

	fn draw_warnings(&self, f: &mut Frame) {
		let first_line = self
			.input
			.get_text()
			.lines()
			.next()
			.map(str::len)
			.unwrap_or_default();

		if first_line > FIRST_LINE_LIMIT {
			let msg = strings::commit_first_line_warning(first_line);
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

	const fn item_status_char(
		item_type: StatusItemType,
	) -> &'static str {
		match item_type {
			StatusItemType::Modified => "modified",
			StatusItemType::New => "new file",
			StatusItemType::Deleted => "deleted",
			StatusItemType::Renamed => "renamed",
			StatusItemType::Typechange => " ",
			StatusItemType::Conflicted => "conflicted",
		}
	}

	pub fn show_editor(
		&mut self,
		changes: Vec<StatusItem>,
	) -> Result<()> {
		self.show_editor_with(
			changes,
			ExternalEditorPopup::open_file_in_editor,
		)
	}

	fn show_editor_with<F>(
		&mut self,
		changes: Vec<StatusItem>,
		open_editor: F,
	) -> Result<()>
	where
		F: FnOnce(
			&asyncgit::sync::RepoPath,
			&std::path::Path,
		) -> Result<()>,
	{
		let file_path = sync::repo_dir(&self.repo.borrow())?
			.join("COMMIT_EDITMSG");

		{
			let mut file = File::create(&file_path)?;
			file.write_fmt(format_args!(
				"{}\n",
				self.input.get_text()
			))?;
			file.write_all(
				strings::commit_editor_msg(&self.key_config)
					.as_bytes(),
			)?;

			file.write_all(b"\n#\n# Changes to be committed:")?;

			for change in changes {
				let status_char =
					Self::item_status_char(change.status);
				let message =
					format!("\n#\t{status_char}: {}", change.path);
				file.write_all(message.as_bytes())?;
			}
		}

		open_editor(&self.repo.borrow(), &file_path)?;

		let mut message = String::new();

		let mut file = File::open(&file_path)?;
		file.read_to_string(&mut message)?;
		drop(file);
		std::fs::remove_file(&file_path)?;

		message =
			commit_message_prettify(&self.repo.borrow(), message)?;
		self.input.set_text(message);
		self.input.show()?;

		Ok(())
	}

	fn commit(&mut self) -> Result<()> {
		let msg = self.input.get_text().to_string();

		if matches!(
			self.commit_with_msg(msg)?,
			CommitResult::CommitDone
		) {
			self.options
				.borrow_mut()
				.add_commit_msg(self.input.get_text());
			self.commit_msg_history_idx = 0;

			self.hide();
			self.queue.push(InternalEvent::Update(NeedsUpdate::ALL));
			self.queue.push(InternalEvent::StatusLastFileMoved);
			self.input.clear();
		}

		Ok(())
	}

	fn commit_with_msg(
		&mut self,
		msg: String,
	) -> Result<CommitResult> {
		let mut msg =
			commit_message_prettify(&self.repo.borrow(), msg)?;
		if msg.trim().is_empty() {
			self.queue.push(InternalEvent::ShowErrorMsg(
				strings::commit_err_empty_msg(),
			));
			return Ok(CommitResult::Aborted);
		}

		let verify = self.verify;

		if verify {
			// run pre commit hook - can reject commit
			if let HookResult::NotOk(e) =
				sync::hooks_pre_commit(&self.repo.borrow())?
			{
				log::error!("pre-commit hook error: {e}");
				self.queue.push(InternalEvent::ShowErrorMsg(
					format!("pre-commit hook error:\n{e}"),
				));
				return Ok(CommitResult::Aborted);
			}
		}

		if verify {
			// run commit message check hook - can reject commit
			if let HookResult::NotOk(e) =
				sync::hooks_commit_msg(&self.repo.borrow(), &mut msg)?
			{
				log::error!("commit-msg hook error: {e}");
				self.queue.push(InternalEvent::ShowErrorMsg(
					format!("commit-msg hook error:\n{e}"),
				));
				return Ok(CommitResult::Aborted);
			}

			msg = commit_message_prettify(&self.repo.borrow(), msg)?;
			if msg.trim().is_empty() {
				self.queue.push(InternalEvent::ShowErrorMsg(
					strings::commit_err_empty_msg(),
				));
				return Ok(CommitResult::Aborted);
			}
		}

		self.do_commit(&msg)?;
		self.verify = true;

		if let HookResult::NotOk(e) =
			sync::hooks_post_commit(&self.repo.borrow())?
		{
			log::error!("post-commit hook error: {e}");
			self.queue.push(InternalEvent::ShowErrorMsg(format!(
				"post-commit hook error:\n{e}"
			)));
		}

		Ok(CommitResult::CommitDone)
	}

	fn do_commit(&self, msg: &str) -> Result<()> {
		match &self.mode {
			Mode::Normal => sync::commit(&self.repo.borrow(), msg)?,
			Mode::Amend(amend) => {
				sync::amend(&self.repo.borrow(), *amend, msg)?
			}
			Mode::Merge(ids) => {
				sync::merge_commit(&self.repo.borrow(), msg, ids)?
			}
			Mode::Revert => {
				sync::commit_revert(&self.repo.borrow(), msg)?
			}
			Mode::Reword(id) => {
				let commit =
					sync::reword(&self.repo.borrow(), *id, msg)?;
				self.queue.push(InternalEvent::TabSwitchStatus);

				commit
			}
		};
		Ok(())
	}

	fn can_commit(&self) -> bool {
		self.has_content() && self.is_changed()
	}

	fn can_amend(&self) -> bool {
		matches!(self.mode, Mode::Normal)
			&& sync::get_head(&self.repo.borrow()).is_ok()
			&& (self.is_empty() || !self.is_changed())
	}

	fn is_empty(&self) -> bool {
		self.input.get_text().is_empty()
	}

	fn has_content(&self) -> bool {
		let text = self.input.get_text();
		if text.trim().is_empty() {
			return false;
		}
		commit_message_prettify(&self.repo.borrow(), text.to_owned())
			.map_or(true, |prettified| !prettified.trim().is_empty())
	}

	fn is_changed(&self) -> bool {
		!matches!(self.mode, Mode::Normal)
			|| Some(self.input.get_text().trim())
				!= self.commit_template.as_ref().map(|s| s.trim())
	}

	fn amend(&mut self) -> Result<()> {
		if self.can_amend() {
			let id = sync::get_head(&self.repo.borrow())?;
			self.mode = Mode::Amend(id);
			self.commit_template = None;

			let details =
				sync::get_commit_details(&self.repo.borrow(), id)?;

			self.input.set_title(strings::commit_title_amend());

			if let Some(msg) = details.message {
				self.input.set_text(msg.combine());
			}
		}

		Ok(())
	}

	fn signoff_commit(&mut self) {
		let msg = self.input.get_text();
		match self.add_sign_off(msg) {
			std::result::Result::Ok(signed_msg) => {
				self.input.set_text(signed_msg);
			}
			Err(e) => {
				log::error!("signoff error: {e}");
				self.queue.push(InternalEvent::ShowErrorMsg(
					format!("signoff error:\n{e}"),
				));
			}
		}
	}

	const fn toggle_verify(&mut self) {
		self.verify = !self.verify;
	}

	fn load_commit_template(&self) -> Option<String> {
		get_config_string(&self.repo.borrow(), "commit.template")
			.map_err(|e| {
				log::error!("load git-config failed: {e}");
				e
			})
			.ok()
			.flatten()
			.and_then(|path| {
				PathBuf::from_str(
					shellexpand::tilde(path.as_str()).as_ref(),
				)
				.ok()
			})
			.and_then(|path| {
				read_to_string(&path)
					.map_err(|e| {
						log::error!("read commit.template failed: {e} (path: '{path:?}')");
						e
					})
					.ok()
			})
	}

	fn prepare_open(
		&self,
		reword: Option<CommitId>,
	) -> Result<(
		Mode,
		String,
		Option<String>,
		String,
		PrepareCommitMsgSource,
	)> {
		let repo_state = sync::repo_state(&self.repo.borrow())?;

		if repo_state != RepoState::Clean && reword.is_some() {
			bail!("cannot reword while repo is not in a clean state");
		}
		if let Some(reword_id) = reword {
			let msg = sync::get_commit_details(
				&self.repo.borrow(),
				reword_id,
			)?
			.message
			.unwrap_or_default()
			.combine();
			return Ok((
				Mode::Reword(reword_id),
				strings::commit_reword_title(),
				None,
				msg,
				PrepareCommitMsgSource::Message,
			));
		}

		match repo_state {
			RepoState::Merge => {
				let ids = sync::mergehead_ids(&self.repo.borrow())?;
				let msg = sync::merge_msg(&self.repo.borrow())?;
				Ok((
					Mode::Merge(ids),
					strings::commit_title_merge(),
					None,
					msg,
					PrepareCommitMsgSource::Merge,
				))
			}
			RepoState::Revert => {
				let msg = sync::merge_msg(&self.repo.borrow())?;
				Ok((
					Mode::Revert,
					strings::commit_title_revert(),
					None,
					msg,
					PrepareCommitMsgSource::Message,
				))
			}
			_ => {
				let commit_template = self.load_commit_template();
				// only keep existing draft if previous mode was Normal
				let existing_draft =
					if matches!(self.mode, Mode::Normal) {
						self.input.get_text()
					} else {
						""
					};
				let (msg, msg_source) = if existing_draft.is_empty() {
					commit_template.as_ref().map_or_else(
						|| {
							(
								String::new(),
								PrepareCommitMsgSource::Message,
							)
						},
						|s| {
							(
								s.clone(),
								PrepareCommitMsgSource::Template,
							)
						},
					)
				} else {
					(
						existing_draft.to_owned(),
						PrepareCommitMsgSource::Message,
					)
				};
				Ok((
					Mode::Normal,
					strings::commit_title(),
					commit_template,
					msg,
					msg_source,
				))
			}
		}
	}

	pub fn open(&mut self, reword: Option<CommitId>) -> Result<()> {
		let (mode, title, commit_template, mut msg, msg_source) =
			self.prepare_open(reword)?;

		match sync::hooks_prepare_commit_msg(
			&self.repo.borrow(),
			msg_source,
			&mut msg,
		) {
			std::result::Result::Ok(HookResult::Ok) => {}
			std::result::Result::Ok(HookResult::NotOk(e)) => {
				log::error!("prepare-commit-msg hook error: {e}");
				self.queue.push(InternalEvent::ShowErrorMsg(
					format!("prepare-commit-msg hook error:\n{e}"),
				));
				return Ok(());
			}
			Err(e) => {
				log::error!("prepare-commit-msg hook error: {e}");
				self.queue.push(InternalEvent::ShowErrorMsg(
					format!("prepare-commit-msg hook error:\n{e}"),
				));
				return Ok(());
			}
		}

		self.mode = mode;
		self.commit_template = commit_template;
		self.input.set_title(title);
		self.input.set_text(msg);
		self.commit_msg_history_idx = 0;
		self.verify = true;
		self.input.show()?;

		Ok(())
	}

	fn add_sign_off(&self, msg: &str) -> Result<String> {
		const CONFIG_KEY_USER_NAME: &str = "user.name";
		const CONFIG_KEY_USER_MAIL: &str = "user.email";

		let user = get_config_string(
			&self.repo.borrow(),
			CONFIG_KEY_USER_NAME,
		)?
		.map(|s| s.trim().to_owned())
		.filter(|s| !s.is_empty());

		let mail = get_config_string(
			&self.repo.borrow(),
			CONFIG_KEY_USER_MAIL,
		)?
		.map(|s| s.trim().to_owned())
		.filter(|s| !s.is_empty());

		match (user, mail) {
			(Some(user), Some(mail)) => {
				let mut msg = msg.to_owned();
				let _ =
					write!(msg, "\n\nSigned-off-by: {user} <{mail}>");
				Ok(msg)
			}
			_ => bail!("{}", strings::commit_err_signoff_identity()),
		}
	}
}

impl DrawableComponent for CommitPopup {
	fn draw(&self, f: &mut Frame, rect: Rect) -> Result<()> {
		if self.is_visible() {
			self.input.draw(f, rect)?;
			self.draw_branch_name(f);
			self.draw_warnings(f);
		}

		Ok(())
	}
}

impl Component for CommitPopup {
	fn commands(
		&self,
		out: &mut Vec<CommandInfo>,
		force_all: bool,
	) -> CommandBlocking {
		self.input.commands(out, force_all);

		if self.is_visible() || force_all {
			out.push(CommandInfo::new(
				strings::commands::commit_submit(&self.key_config),
				self.can_commit(),
				true,
			));

			out.push(CommandInfo::new(
				strings::commands::toggle_verify(
					&self.key_config,
					self.verify,
				),
				self.can_commit(),
				true,
			));

			out.push(CommandInfo::new(
				strings::commands::commit_amend(&self.key_config),
				self.can_amend(),
				true,
			));

			out.push(CommandInfo::new(
				strings::commands::commit_signoff(&self.key_config),
				true,
				true,
			));

			out.push(CommandInfo::new(
				strings::commands::commit_open_editor(
					&self.key_config,
				),
				true,
				true,
			));

			out.push(CommandInfo::new(
				strings::commands::commit_next_msg_from_history(
					&self.key_config,
				),
				self.options.borrow().has_commit_msg_history(),
				true,
			));

			out.push(CommandInfo::new(
				strings::commands::newline(&self.key_config),
				true,
				true,
			));
		}

		visibility_blocking(self)
	}

	fn event(&mut self, ev: &Event) -> Result<EventState> {
		if self.is_visible() {
			if let Event::Key(e) = ev {
				let input_consumed =
					if key_match(e, self.key_config.keys.commit)
						&& self.can_commit()
					{
						try_or_popup!(
							self,
							"commit error:",
							self.commit()
						);
						true
					} else if key_match(
						e,
						self.key_config.keys.toggle_verify,
					) && self.can_commit()
					{
						self.toggle_verify();
						true
					} else if key_match(
						e,
						self.key_config.keys.commit_amend,
					) && self.can_amend()
					{
						self.amend()?;
						true
					} else if key_match(
						e,
						self.key_config.keys.open_commit_editor,
					) {
						self.queue.push(
							InternalEvent::OpenExternalEditor(None),
						);
						self.input.hide();
						true
					} else if key_match(
						e,
						self.key_config.keys.commit_history_next,
					) {
						if let Some(msg) = self
							.options
							.borrow()
							.commit_msg(self.commit_msg_history_idx)
						{
							self.input.set_text(msg);
							self.commit_msg_history_idx += 1;
						}
						true
					} else if key_match(
						e,
						self.key_config.keys.toggle_signoff,
					) {
						self.signoff_commit();
						true
					} else if key_match(
						e,
						self.key_config.keys.exit_popup,
					) {
						self.hide();
						true
					} else {
						false
					};

				if !input_consumed {
					self.input.event(ev)?;
					if !self.input.is_visible() {
						self.verify = true;
					}
				}

				// stop key event propagation
				return Ok(EventState::Consumed);
			}
		}

		Ok(EventState::NotConsumed)
	}

	fn is_visible(&self) -> bool {
		self.input.is_visible()
	}

	fn hide(&mut self) {
		self.verify = true;
		self.input.hide();
	}

	fn show(&mut self) -> Result<()> {
		self.open(None)?;
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
	use std::cell::RefCell;
	use std::path::Path;

	fn env_for_repo(path: &Path) -> Environment {
		let mut env = Environment::test_env();
		env.repo =
			RefCell::new(sync::RepoPath::Path(path.to_path_buf()));
		env
	}

	fn key_event(code: KeyCode, modifiers: KeyModifiers) -> Event {
		Event::Key(KeyEvent::new(code, modifiers))
	}

	fn commit_key(popup: &CommitPopup) -> Event {
		let k = popup.key_config.keys.commit;
		key_event(k.code, k.modifiers)
	}

	fn toggle_verify_key(popup: &CommitPopup) -> Event {
		let k = popup.key_config.keys.toggle_verify;
		key_event(k.code, k.modifiers)
	}

	fn signoff_key(popup: &CommitPopup) -> Event {
		let k = popup.key_config.keys.toggle_signoff;
		key_event(k.code, k.modifiers)
	}

	fn editor_key(popup: &CommitPopup) -> Event {
		let k = popup.key_config.keys.open_commit_editor;
		key_event(k.code, k.modifiers)
	}

	fn esc_key(popup: &CommitPopup) -> Event {
		let k = popup.key_config.keys.exit_popup;
		key_event(k.code, k.modifiers)
	}

	fn hint_enabled(popup: &CommitPopup, name: &str) -> bool {
		let mut cmds = Vec::new();
		popup.commands(&mut cmds, false);
		cmds.iter()
			.find(|c| c.text.name == name)
			.map(|c| c.enabled)
			.unwrap()
	}

	fn drain_errors(queue: &Queue) -> Vec<String> {
		let mut errors = Vec::new();
		while let Some(ev) = queue.pop() {
			if let InternalEvent::ShowErrorMsg(msg) = ev {
				errors.push(msg);
			}
		}
		errors
	}

	fn install_hook(git_dir: &Path, name: &str, script: &[u8]) {
		let hooks_dir = git_dir.join("hooks");
		std::fs::create_dir_all(&hooks_dir).unwrap();
		let hook_path = hooks_dir.join(name);
		std::fs::write(&hook_path, script).unwrap();
		#[cfg(unix)]
		{
			use std::os::unix::fs::PermissionsExt;
			let mut perms =
				std::fs::metadata(&hook_path).unwrap().permissions();
			perms.set_mode(0o755);
			std::fs::set_permissions(&hook_path, perms).unwrap();
		}
	}

	fn stage_file(root: &Path, name: &str, content: &[u8]) {
		std::fs::write(root.join(name), content).unwrap();
		let repo_path = sync::RepoPath::Path(root.to_path_buf());
		sync::stage_add_file(&repo_path, Path::new(name)).unwrap();
	}

	#[test]
	fn t8j_whitespace_blocked() {
		let (_td, repo) = git2_testing::repo_init();
		let root = repo.path().parent().unwrap();
		let head: CommitId =
			repo.head().unwrap().target().unwrap().into();
		let env = env_for_repo(root);
		let submit_name =
			strings::commands::commit_submit(&env.key_config).name;

		let mut popup = CommitPopup::new(&env);
		popup.open(None).unwrap();

		let modes = [
			Mode::Normal,
			Mode::Amend(head),
			Mode::Reword(head),
			Mode::Merge(vec![head]),
			Mode::Revert,
		];

		for mode in modes {
			popup.mode = mode;
			for bad in ["", "   ", "\t\n  \n"] {
				popup.input.set_text(bad.to_string());
				assert!(
					!popup.can_commit(),
					"expected whitespace/empty {bad:?} to be blocked"
				);
				assert!(
					!hint_enabled(&popup, &submit_name),
					"expected submit hint disabled for {bad:?}"
				);
			}
		}
	}

	#[test]
	fn t8j_comments_only_blocked() {
		let (_td, repo) = git2_testing::repo_init();
		let root = repo.path().parent().unwrap();
		let env = env_for_repo(root);
		let submit_name =
			strings::commands::commit_submit(&env.key_config).name;

		let mut popup = CommitPopup::new(&env);
		popup.open(None).unwrap();

		popup.input.set_text(
			"# comment one\n# comment two\n   ".to_string(),
		);
		assert!(
			!popup.can_commit(),
			"default '#' comment-only message must be blocked"
		);
		assert!(!hint_enabled(&popup, &submit_name));

		repo.config()
			.unwrap()
			.set_str("core.commentChar", ";")
			.unwrap();
		popup
			.input
			.set_text("; custom comment\n\n; second".to_string());
		assert!(
			!popup.can_commit(),
			"custom ';' comment-only message must be blocked"
		);
		assert!(!hint_enabled(&popup, &submit_name));

		popup
			.input
			.set_text("# not a comment under ';'".to_string());
		assert!(
			popup.can_commit(),
			"'#' line must be allowed when core.commentChar is ';'"
		);
	}

	#[test]
	fn t8j_template_blocked() {
		let (_td, repo) = git2_testing::repo_init();
		let root = repo.path().parent().unwrap();
		let template_path = root.join("commit-template.txt");
		std::fs::write(&template_path, "feat: template\n").unwrap();
		repo.config()
			.unwrap()
			.set_str(
				"commit.template",
				template_path.to_str().unwrap(),
			)
			.unwrap();

		let env = env_for_repo(root);
		let submit_name =
			strings::commands::commit_submit(&env.key_config).name;

		let mut popup = CommitPopup::new(&env);
		popup.open(None).unwrap();

		assert_eq!(popup.input.get_text(), "feat: template\n");
		assert!(!popup.can_commit());
		assert!(!hint_enabled(&popup, &submit_name));

		popup.input.set_text("feat: actual change\n".to_string());
		assert!(popup.can_commit());
	}

	#[test]
	fn t8j_content_allowed() {
		let (_td, repo) = git2_testing::repo_init();
		let root = repo.path().parent().unwrap();
		let env = env_for_repo(root);
		let submit_name =
			strings::commands::commit_submit(&env.key_config).name;

		let mut popup = CommitPopup::new(&env);
		popup.open(None).unwrap();

		popup.input.set_text(
			"# leading comment\nreal subject\n# trailing comment\n"
				.to_string(),
		);
		assert!(popup.can_commit());
		assert!(hint_enabled(&popup, &submit_name));
	}

	#[test]
	fn t8j_prepare_rejection_blocks_open() {
		let (_td, repo) = git2_testing::repo_init();
		let root = repo.path().parent().unwrap();
		let head: CommitId =
			repo.head().unwrap().target().unwrap().into();
		let env = env_for_repo(root);

		let mut popup = CommitPopup::new(&env);
		popup.open(None).unwrap();
		popup.input.set_text("saved draft".to_string());
		let esc = esc_key(&popup);
		popup.event(&esc).unwrap();
		assert!(!popup.is_visible());

		// Case 1: hook exits non-zero after overwriting message file
		install_hook(
			repo.path(),
			"prepare-commit-msg",
			b"#!/bin/sh\necho clobbered > \"$1\"\necho 'hook denied' >&2\nexit 1\n",
		);
		popup.open(Some(head)).unwrap();
		assert!(
			!popup.is_visible(),
			"popup must not open when prepare-commit-msg rejects"
		);
		assert_eq!(popup.input.get_text(), "saved draft");
		assert!(matches!(popup.mode, Mode::Normal));
		let errors = drain_errors(&env.queue);
		assert!(
			errors.iter().any(|e| e.contains("hook denied")),
			"expected error containing hook rejection, got {errors:?}"
		);

		// Case 2: hook cannot run (missing interpreter)
		install_hook(
			repo.path(),
			"prepare-commit-msg",
			b"#!/nonexistent/git-tuit-interp\n",
		);
		popup.open(Some(head)).unwrap();
		assert!(
			!popup.is_visible(),
			"popup must not open when prepare-commit-msg cannot run"
		);
		assert_eq!(popup.input.get_text(), "saved draft");
		assert!(matches!(popup.mode, Mode::Normal));
		let errors = drain_errors(&env.queue);
		assert!(
			!errors.is_empty(),
			"expected error when prepare-commit-msg cannot run"
		);
	}

	#[test]
	fn t8j_prepare_success_opens() {
		let (_td, repo) = git2_testing::repo_init();
		let root = repo.path().parent().unwrap();
		let env = env_for_repo(root);

		let mut popup = CommitPopup::new(&env);
		// No hook: opens as today
		popup.open(None).unwrap();
		assert!(popup.is_visible());
		popup.hide();

		// Succeeding hook: edited message is used
		install_hook(
			repo.path(),
			"prepare-commit-msg",
			b"#!/bin/sh\nprintf 'prepared msg' > \"$1\"\nexit 0\n",
		);
		popup.open(None).unwrap();
		assert!(popup.is_visible());
		assert_eq!(popup.input.get_text(), "prepared msg");
	}

	#[test]
	fn t8j_no_verify_survives_failure() {
		let (_td, repo) = git2_testing::repo_init();
		let root = repo.path().parent().unwrap();
		stage_file(root, "staged.txt", b"hello");

		let env = env_for_repo(root);
		let mut popup = CommitPopup::new(&env);
		popup.open(None).unwrap();
		popup.input.set_text("attempt commit".to_string());

		let toggle = toggle_verify_key(&popup);
		popup.event(&toggle).unwrap();
		assert!(!popup.verify);

		// Force commit failure by removing user.email
		repo.config().unwrap().remove("user.email").unwrap();
		let submit = commit_key(&popup);
		popup.event(&submit).unwrap();

		assert!(popup.is_visible());
		assert_ne!(drain_errors(&env.queue), Vec::<String>::new());
		assert!(
			!popup.verify,
			"no-verify must stay on after a failed commit attempt"
		);
	}

	#[test]
	fn t8j_no_verify_resets_on_success() {
		let (_td, repo) = git2_testing::repo_init();
		let root = repo.path().parent().unwrap();
		stage_file(root, "staged.txt", b"hello");
		install_hook(
			repo.path(),
			"post-commit",
			b"#!/bin/sh\necho 'post hook failed' >&2\nexit 1\n",
		);

		let env = env_for_repo(root);
		let mut popup = CommitPopup::new(&env);
		popup.open(None).unwrap();
		popup.input.set_text("good commit".to_string());

		let toggle = toggle_verify_key(&popup);
		popup.event(&toggle).unwrap();
		assert!(!popup.verify);

		let submit = commit_key(&popup);
		popup.event(&submit).unwrap();

		assert!(!popup.is_visible());
		assert_eq!(popup.input.get_text(), "");
		assert!(
			popup.verify,
			"no-verify must reset after successful commit even when post-commit fails"
		);
		let errors = drain_errors(&env.queue);
		assert!(
			errors.iter().any(|e| e.contains("post hook failed")),
			"post-commit failure must still be reported, got {errors:?}"
		);
	}

	#[test]
	fn t8j_no_verify_resets_on_close() {
		let (_td, repo) = git2_testing::repo_init();
		let root = repo.path().parent().unwrap();
		let env = env_for_repo(root);
		let mut popup = CommitPopup::new(&env);

		// Case 1: Esc closes popup, keeps draft, resets no-verify
		popup.open(None).unwrap();
		popup.input.set_text("keep draft".to_string());
		let toggle = toggle_verify_key(&popup);
		popup.event(&toggle).unwrap();
		assert!(!popup.verify);

		let esc = esc_key(&popup);
		popup.event(&esc).unwrap();
		assert!(!popup.is_visible());
		assert_eq!(popup.input.get_text(), "keep draft");
		assert!(popup.verify, "Esc must reset no-verify");

		// Case 2: fresh open after popup was left any other way
		popup.open(None).unwrap();
		popup.event(&toggle).unwrap();
		assert!(!popup.verify);
		popup.input.hide();
		assert!(!popup.is_visible());

		popup.open(None).unwrap();
		assert!(popup.is_visible());
		assert!(
			popup.verify,
			"fresh open must reset no-verify when popup was left closed"
		);
	}

	#[test]
	fn t8j_no_verify_survives_editor() {
		let (_td, repo) = git2_testing::repo_init();
		let root = repo.path().parent().unwrap();

		let env = env_for_repo(root);
		let mut popup = CommitPopup::new(&env);
		popup.open(None).unwrap();
		popup.input.set_text("draft".to_string());

		let toggle = toggle_verify_key(&popup);
		popup.event(&toggle).unwrap();
		assert!(!popup.verify);

		let open_ed = editor_key(&popup);
		popup.event(&open_ed).unwrap();
		assert!(!popup.is_visible());

		popup
			.show_editor_with(Vec::new(), |_repo, path| {
				std::fs::write(path, "edited msg\n")?;
				Ok(())
			})
			.unwrap();
		assert!(popup.is_visible());
		assert_eq!(popup.input.get_text(), "edited msg\n");
		assert!(
			!popup.verify,
			"no-verify must survive a successful external-editor round trip"
		);
	}

	#[test]
	fn t8j_signoff_failure_is_shown() {
		let (_td, repo) = git2_testing::repo_init();
		let root = repo.path().parent().unwrap();
		let env = env_for_repo(root);

		let mut popup = CommitPopup::new(&env);
		popup.open(None).unwrap();
		popup.input.set_text("my message".to_string());
		let signoff = signoff_key(&popup);

		// Missing user.email
		repo.config().unwrap().remove("user.email").unwrap();
		popup.event(&signoff).unwrap();
		assert_eq!(popup.input.get_text(), "my message");
		assert!(
			!drain_errors(&env.queue).is_empty(),
			"missing user.email must show an error"
		);

		// Whitespace-only user.name
		repo.config()
			.unwrap()
			.set_str("user.email", "testuser@example.com")
			.unwrap();
		repo.config().unwrap().set_str("user.name", "   ").unwrap();
		popup.event(&signoff).unwrap();
		assert_eq!(popup.input.get_text(), "my message");
		assert!(
			!drain_errors(&env.queue).is_empty(),
			"blank user.name must show an error"
		);

		// Valid identity succeeds
		repo.config()
			.unwrap()
			.set_str("user.name", "testuser")
			.unwrap();
		popup.event(&signoff).unwrap();
		assert_eq!(
			popup.input.get_text(),
			"my message\n\nSigned-off-by: testuser <testuser@example.com>"
		);
		assert_eq!(drain_errors(&env.queue), Vec::<String>::new());
	}
}
