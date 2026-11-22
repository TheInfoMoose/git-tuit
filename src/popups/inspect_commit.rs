use crate::components::{
	command_pump, event_pump, visibility_blocking, CommandBlocking,
	CommandInfo, CommitDetailsComponent, Component, DiffComponent,
	DrawableComponent, EventState,
};
use crate::{
	accessors,
	app::Environment,
	keys::{key_match, SharedKeyConfig},
	options::SharedOptions,
	queue::{InternalEvent, Queue, StackablePopupOpen},
	strings,
};
use anyhow::Result;
use asyncgit::{
	sync::{CommitId, CommitTags},
	AsyncDiff, AsyncGitNotification, DiffParams, DiffType,
};
use crossterm::event::Event;
use ratatui::{
	layout::{Constraint, Direction, Layout, Rect},
	widgets::Clear,
	Frame,
};

use super::FileTreeOpen;

#[derive(Clone, Debug)]
pub struct InspectCommitOpen {
	pub commit_id: CommitId,
	/// in case we wanna compare
	pub compare_id: Option<CommitId>,
	pub tags: Option<CommitTags>,
}

impl InspectCommitOpen {
	pub const fn new(commit_id: CommitId) -> Self {
		Self {
			commit_id,
			compare_id: None,
			tags: None,
		}
	}

	pub const fn new_with_tags(
		commit_id: CommitId,
		tags: Option<CommitTags>,
	) -> Self {
		Self {
			commit_id,
			compare_id: None,
			tags,
		}
	}
}

pub struct InspectCommitPopup {
	queue: Queue,
	open_request: Option<InspectCommitOpen>,
	diff: DiffComponent,
	details: CommitDetailsComponent,
	git_diff: AsyncDiff,
	visible: bool,
	key_config: SharedKeyConfig,
	options: SharedOptions,
}

impl DrawableComponent for InspectCommitPopup {
	fn draw(&self, f: &mut Frame, rect: Rect) -> Result<()> {
		if self.is_visible() {
			let percentages = if self.diff.focused() {
				(0, 100)
			} else {
				(50, 50)
			};

			let chunks = Layout::default()
				.direction(Direction::Horizontal)
				.constraints(
					[
						Constraint::Percentage(percentages.0),
						Constraint::Percentage(percentages.1),
					]
					.as_ref(),
				)
				.split(rect);

			f.render_widget(Clear, rect);

			self.details.draw(f, chunks[0])?;
			self.diff.draw(f, chunks[1])?;
		}

		Ok(())
	}
}

impl Component for InspectCommitPopup {
	fn commands(
		&self,
		out: &mut Vec<CommandInfo>,
		force_all: bool,
	) -> CommandBlocking {
		if self.is_visible() || force_all {
			command_pump(
				out,
				force_all,
				self.components().as_slice(),
			);

			out.push(
				CommandInfo::new(
					strings::commands::close_popup_left(
						&self.key_config,
					),
					true,
					!self.diff.focused() || force_all,
				)
				.order(1),
			);

			out.push(CommandInfo::new(
				strings::commands::diff_focus_right(&self.key_config),
				self.can_focus_diff(),
				!self.diff.focused() || force_all,
			));

			out.push(
				CommandInfo::new(
					strings::commands::diff_focus_left(
						&self.key_config,
					),
					true,
					self.diff.focused() || force_all,
				)
				.order(1),
			);

			out.push(CommandInfo::new(
				strings::commands::inspect_file_tree(
					&self.key_config,
				),
				true,
				true,
			));
		}

		visibility_blocking(self)
	}

	fn event(&mut self, ev: &Event) -> Result<EventState> {
		if self.is_visible() {
			if event_pump(ev, self.components_mut().as_mut_slice())?
				.is_consumed()
			{
				if !self.details.is_visible() {
					self.hide_stacked(true);
				}

				return Ok(EventState::Consumed);
			}

			if let Event::Key(e) = ev {
				if key_match(e, self.key_config.keys.exit_popup) {
					if self.diff.focused() {
						self.details.focus(true);
						self.diff.focus(false);
					} else {
						self.hide_stacked(false);
					}
				} else if key_match(
					e,
					self.key_config.keys.move_right,
				) && self.can_focus_diff()
				{
					self.details.focus(false);
					self.diff.focus(true);
				} else if key_match(e, self.key_config.keys.move_left)
				{
					self.hide_stacked(false);
				} else if key_match(
					e,
					self.key_config.keys.open_file_tree,
				) {
					if let Some(commit_id) = self
						.open_request
						.as_ref()
						.map(|open_commit| open_commit.commit_id)
					{
						self.hide_stacked(true);
						self.queue.push(InternalEvent::OpenPopup(
							StackablePopupOpen::FileTree(
								FileTreeOpen::new(commit_id),
							),
						));
						return Ok(EventState::Consumed);
					}
					return Ok(EventState::Consumed);
				} else if key_match(
					e,
					self.key_config.keys.cmd_bar_toggle,
				) {
					return Ok(EventState::NotConsumed);
				}

				return Ok(EventState::Consumed);
			}
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
		self.details.show()?;
		self.details.focus(true);
		self.diff.focus(false);
		self.update()?;
		Ok(())
	}
}

impl InspectCommitPopup {
	accessors!(self, [diff, details]);

	///
	pub fn new(env: &Environment) -> Self {
		Self {
			queue: env.queue.clone(),
			details: CommitDetailsComponent::new(env),
			diff: DiffComponent::new(env, true),
			open_request: None,
			git_diff: AsyncDiff::new(
				env.repo.borrow().clone(),
				&env.sender_git,
			),
			visible: false,
			key_config: env.key_config.clone(),
			options: env.options.clone(),
		}
	}

	///
	pub fn open(&mut self, open: InspectCommitOpen) -> Result<()> {
		self.open_request = Some(open);
		self.show()?;

		Ok(())
	}

	///
	pub fn any_work_pending(&self) -> bool {
		self.git_diff.is_pending() || self.details.any_work_pending()
	}

	///
	pub fn update_git(
		&mut self,
		ev: AsyncGitNotification,
	) -> Result<()> {
		if self.is_visible() {
			if ev == AsyncGitNotification::CommitFiles {
				self.update()?;
			} else if ev == AsyncGitNotification::Diff {
				self.update_diff()?;
			}
		}

		Ok(())
	}

	/// called when any tree component changed selection
	pub fn update_diff(&mut self) -> Result<()> {
		if self.is_visible() {
			if let Some(request) = &self.open_request {
				if let Some(f) = self.details.files().selection_file()
				{
					let diff_params = DiffParams {
						path: f.path.clone(),
						diff_type: DiffType::Commit(
							request.commit_id,
						),
						options: self.options.borrow().diff_options(),
					};

					if let Some((params, last)) =
						self.git_diff.last()?
					{
						if params == diff_params {
							self.diff.update(f.path, false, last);
							return Ok(());
						}
					}

					self.git_diff.request(diff_params)?;
					self.diff.clear(true);
					return Ok(());
				}
			}

			self.diff.clear(false);
		}

		Ok(())
	}

	fn update(&mut self) -> Result<()> {
		if let Some(request) = &self.open_request {
			self.details.set_commits(
				Some(request.commit_id.into()),
				request.tags.as_ref(),
			)?;
			self.update_diff()?;
		}

		Ok(())
	}

	fn can_focus_diff(&self) -> bool {
		self.details.files().selection_file().is_some()
	}

	fn hide_stacked(&mut self, stack: bool) {
		self.hide();

		if stack {
			if let Some(open_request) = self.open_request.take() {
				self.queue.push(InternalEvent::PopupStackPush(
					StackablePopupOpen::InspectCommit(open_request),
				));
			}
		} else {
			self.queue.push(InternalEvent::PopupStackPop);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::popups::{
		CompareCommitsPopup, FileRevOpen, FileRevlogPopup,
	};
	use asyncgit::sync::RepoPath;
	use crossbeam_channel::{unbounded, Receiver};
	use crossterm::event::{Event, KeyEvent};
	use std::{cell::RefCell, path::Path, time::Duration};

	fn repo_with_two_commits(
	) -> (tempfile::TempDir, CommitId, CommitId) {
		let (td, repo) = git2_testing::repo_init();
		let workdir = repo.workdir().unwrap().to_path_buf();
		let sig = repo.signature().unwrap();
		let c0 = repo.head().unwrap().peel_to_commit().unwrap();

		std::fs::write(workdir.join("a.txt"), "one\n").unwrap();
		let mut idx = repo.index().unwrap();
		idx.add_path(Path::new("a.txt")).unwrap();
		idx.write().unwrap();
		let tree1 =
			repo.find_tree(idx.write_tree().unwrap()).unwrap();
		let c1_oid = repo
			.commit(Some("HEAD"), &sig, &sig, "c1", &tree1, &[&c0])
			.unwrap();
		let c1 = repo.find_commit(c1_oid).unwrap();

		std::fs::write(workdir.join("a.txt"), "one\ntwo\n").unwrap();
		let mut idx = repo.index().unwrap();
		idx.add_path(Path::new("a.txt")).unwrap();
		idx.write().unwrap();
		let tree2 =
			repo.find_tree(idx.write_tree().unwrap()).unwrap();
		let c2_oid = repo
			.commit(Some("HEAD"), &sig, &sig, "c2", &tree2, &[&c1])
			.unwrap();

		(td, CommitId::new(c1_oid), CommitId::new(c2_oid))
	}

	fn env_for_repo(
		path: &Path,
	) -> (Environment, Receiver<AsyncGitNotification>) {
		let mut env = Environment::test_env();
		let (tx, rx) = unbounded();
		env.sender_git = tx;
		env.repo = RefCell::new(RepoPath::Path(path.to_path_buf()));
		(env, rx)
	}

	fn assert_diff_back_is_exit(
		cmds: &[CommandInfo],
		key_config: &SharedKeyConfig,
		label: &str,
	) {
		let exit_hint =
			key_config.get_hint(key_config.keys.exit_popup);
		let left_hint =
			key_config.get_hint(key_config.keys.move_left);
		let visible_back: Vec<_> = cmds
			.iter()
			.filter(|c| {
				c.show_in_quickbar()
					&& c.text.name.starts_with("Back")
			})
			.collect();
		assert_eq!(
			visible_back.len(),
			1,
			"{label} diff focus must show one Back hint"
		);
		assert!(
			visible_back[0].text.name.contains(&exit_hint)
				&& !visible_back[0].text.name.contains(&left_hint),
			"{label} Back hint '{}' must show exit_popup '{}'",
			visible_back[0].text.name,
			exit_hint
		);
	}

	#[test]
	fn t8l_diff_back_is_exit() {
		let (td, c1, c2) = repo_with_two_commits();
		let (env, rx) = env_for_repo(td.path());
		let right = env.key_config.keys.move_right;
		let right_ev =
			Event::Key(KeyEvent::new(right.code, right.modifiers));

		// 1. InspectCommitPopup
		let mut inspect = InspectCommitPopup::new(&env);
		inspect.open(InspectCommitOpen::new(c2)).unwrap();
		while inspect.any_work_pending() {
			if let Ok(ev) = rx.recv_timeout(Duration::from_secs(2)) {
				inspect.update_git(ev).unwrap();
			} else {
				break;
			}
		}
		inspect.update().unwrap();
		inspect.event(&right_ev).unwrap();
		assert!(inspect.diff.focused());

		let mut cmds = Vec::new();
		inspect.commands(&mut cmds, false);
		assert_diff_back_is_exit(&cmds, &env.key_config, "inspect");

		// 2. CompareCommitsPopup
		let mut compare = CompareCommitsPopup::new(&env);
		compare
			.open(InspectCommitOpen {
				commit_id: c2,
				compare_id: Some(c1),
				tags: None,
			})
			.unwrap();
		while compare.any_work_pending() {
			if let Ok(ev) = rx.recv_timeout(Duration::from_secs(2)) {
				compare.update_git(ev).unwrap();
			} else {
				break;
			}
		}
		compare
			.update_git(AsyncGitNotification::CommitFiles)
			.unwrap();
		compare.event(&right_ev).unwrap();

		cmds.clear();
		compare.commands(&mut cmds, false);
		assert_diff_back_is_exit(&cmds, &env.key_config, "compare");

		// 3. FileRevlogPopup
		let mut history = FileRevlogPopup::new(&env);
		history.open(FileRevOpen::new("a.txt".into())).unwrap();
		while history.any_work_pending() {
			if let Ok(ev) = rx.recv_timeout(Duration::from_secs(2)) {
				history.update_git(ev).unwrap();
			} else {
				break;
			}
		}
		history.update().unwrap();
		history.event(&right_ev).unwrap();

		cmds.clear();
		history.commands(&mut cmds, false);
		assert_diff_back_is_exit(
			&cmds,
			&env.key_config,
			"file history",
		);
	}

	#[test]
	fn t8l_list_close_hints() {
		let (td, c1, c2) = repo_with_two_commits();
		let (env, rx) = env_for_repo(td.path());
		let exit_hint =
			env.key_config.get_hint(env.key_config.keys.exit_popup);
		let left_hint =
			env.key_config.get_hint(env.key_config.keys.move_left);

		// InspectCommitPopup with file list focused
		let mut inspect = InspectCommitPopup::new(&env);
		inspect.open(InspectCommitOpen::new(c2)).unwrap();
		while inspect.any_work_pending() {
			if let Ok(ev) = rx.recv_timeout(Duration::from_secs(2)) {
				inspect.update_git(ev).unwrap();
			} else {
				break;
			}
		}
		inspect.update().unwrap();
		assert!(!inspect.diff.focused());

		let mut cmds = Vec::new();
		inspect.commands(&mut cmds, false);
		let close_cmds: Vec<_> = cmds
			.iter()
			.filter(|c| {
				c.show_in_quickbar()
					&& c.text.name.starts_with("Close")
			})
			.collect();
		assert_eq!(
			close_cmds.len(),
			1,
			"inspect list focus must not list Close twice"
		);
		assert!(
			close_cmds[0].text.name.contains(&exit_hint)
				&& close_cmds[0].text.name.contains(&left_hint),
			"inspect Close hint '{}' must show both '{}' and '{}'",
			close_cmds[0].text.name,
			exit_hint,
			left_hint
		);

		// CompareCommitsPopup with file list focused
		let mut compare = CompareCommitsPopup::new(&env);
		compare
			.open(InspectCommitOpen {
				commit_id: c2,
				compare_id: Some(c1),
				tags: None,
			})
			.unwrap();
		while compare.any_work_pending() {
			if let Ok(ev) = rx.recv_timeout(Duration::from_secs(2)) {
				compare.update_git(ev).unwrap();
			} else {
				break;
			}
		}
		compare
			.update_git(AsyncGitNotification::CommitFiles)
			.unwrap();

		cmds.clear();
		compare.commands(&mut cmds, false);
		let close_cmds: Vec<_> = cmds
			.iter()
			.filter(|c| {
				c.show_in_quickbar()
					&& c.text.name.starts_with("Close")
			})
			.collect();
		assert_eq!(
			close_cmds.len(),
			1,
			"compare list focus must not list Close twice"
		);
		assert!(
			close_cmds[0].text.name.contains(&exit_hint)
				&& close_cmds[0].text.name.contains(&left_hint),
			"compare Close hint '{}' must show both '{}' and '{}'",
			close_cmds[0].text.name,
			exit_hint,
			left_hint
		);
	}
}
