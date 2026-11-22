use crate::{
	app::Environment,
	components::{
		visibility_blocking, CommandBlocking, CommandInfo,
		CommitDetailsComponent, CommitList, Component,
		DrawableComponent, EventState,
	},
	keys::{key_match, SharedKeyConfig},
	popups::{FileTreeOpen, InspectCommitOpen},
	queue::{Action, InternalEvent, Queue, StackablePopupOpen},
	strings::{self, order},
	try_or_popup,
	ui::style::{SharedTheme, Theme},
};
use anyhow::Result;
use asyncgit::{
	asyncjob::AsyncSingleJob,
	sync::{
		self, filter_commit_by_search, CommitId, LogFilterSearch,
		LogFilterSearchOptions, RepoPathRef,
	},
	AsyncBranchesJob, AsyncCommitFilterJob, AsyncGitNotification,
	AsyncLog, AsyncTags, CommitFilesParams, FetchStatus,
	ProgressPercent,
};
use crossbeam_channel::Sender;
use crossterm::event::Event;
use indexmap::IndexSet;
use ratatui::{
	layout::{Alignment, Constraint, Direction, Layout, Rect},
	text::Span,
	widgets::{Block, Borders, Paragraph},
	Frame,
};
use std::{
	rc::Rc,
	sync::{
		atomic::{AtomicBool, Ordering},
		Arc,
	},
	time::Duration,
};
use sync::CommitTags;

struct LogSearchResult {
	options: LogFilterSearchOptions,
	duration: Duration,
}

//TODO: deserves its own component
enum LogSearch {
	Off,
	Searching(
		AsyncSingleJob<AsyncCommitFilterJob>,
		LogFilterSearchOptions,
		Option<ProgressPercent>,
		Arc<AtomicBool>,
	),
	Results(LogSearchResult),
}

///
pub struct Revlog {
	repo: RepoPathRef,
	commit_details: CommitDetailsComponent,
	list: CommitList,
	git_log: AsyncLog,
	search: LogSearch,
	git_tags: AsyncTags,
	git_local_branches: AsyncSingleJob<AsyncBranchesJob>,
	git_remote_branches: AsyncSingleJob<AsyncBranchesJob>,
	queue: Queue,
	visible: bool,
	key_config: SharedKeyConfig,
	sender: Sender<AsyncGitNotification>,
	theme: SharedTheme,
}

impl Revlog {
	///
	pub fn new(env: &Environment) -> Self {
		Self {
			repo: env.repo.clone(),
			queue: env.queue.clone(),
			commit_details: CommitDetailsComponent::new(env),
			list: CommitList::new(
				env,
				&strings::log_title(&env.key_config),
			),
			git_log: AsyncLog::new(
				env.repo.borrow().clone(),
				&env.sender_git,
				None,
			),
			search: LogSearch::Off,
			git_tags: AsyncTags::new(
				env.repo.borrow().clone(),
				&env.sender_git,
			),
			git_local_branches: AsyncSingleJob::new(
				env.sender_git.clone(),
			),
			git_remote_branches: AsyncSingleJob::new(
				env.sender_git.clone(),
			),
			visible: false,
			key_config: env.key_config.clone(),
			sender: env.sender_git.clone(),
			theme: env.theme.clone(),
		}
	}

	///
	pub fn any_work_pending(&self) -> bool {
		self.git_log.is_pending()
			|| self.is_search_pending()
			|| self.git_tags.is_pending()
			|| self.git_local_branches.is_pending()
			|| self.git_remote_branches.is_pending()
			|| self.commit_details.any_work_pending()
	}

	const fn is_search_pending(&self) -> bool {
		matches!(self.search, LogSearch::Searching(_, _, _, _))
	}

	///
	pub fn update(&mut self) -> Result<()> {
		if self.is_visible() {
			if self.git_log.fetch()? == FetchStatus::Started {
				self.list.clear();
			}

			let is_loading = self.git_log.is_pending();
			self.list
				.refresh_extend_data(self.git_log.extract_items()?);
			if !is_loading {
				self.list.finish_loading();
			}

			self.git_tags.request(Duration::from_secs(3), false)?;

			if self.commit_details.is_visible() {
				let commit = self.selected_commit();
				let tags = self.selected_commit_tags(commit.as_ref());

				self.commit_details.set_commits(
					commit.map(CommitFilesParams::from),
					tags.as_ref(),
				)?;
			}
		}

		Ok(())
	}

	///
	pub fn update_git(
		&mut self,
		ev: AsyncGitNotification,
	) -> Result<()> {
		// search results must land even while the tab is hidden
		if ev == AsyncGitNotification::CommitFilter {
			self.update_search_state();
			return Ok(());
		}

		if self.visible {
			match ev {
				AsyncGitNotification::CommitFiles
				| AsyncGitNotification::Log => self.update()?,
				AsyncGitNotification::Tags => {
					if let Some(tags) = self.git_tags.last()? {
						self.list.set_tags(tags);
						self.update()?;
					}
				}
				AsyncGitNotification::Branches => {
					if let Some(local_branches) =
						self.git_local_branches.take_last()
					{
						if let Some(Ok(local_branches)) =
							local_branches.result()
						{
							self.list
								.set_local_branches(local_branches);
							self.update()?;
						}
					}

					if let Some(remote_branches) =
						self.git_remote_branches.take_last()
					{
						if let Some(Ok(remote_branches)) =
							remote_branches.result()
						{
							self.list
								.set_remote_branches(remote_branches);
							self.update()?;
						}
					}
				}
				_ => (),
			}
		}

		Ok(())
	}

	fn selected_commit(&self) -> Option<CommitId> {
		self.list.selected_entry().map(|e| e.id)
	}

	fn selected_commit_tags(
		&self,
		commit: Option<&CommitId>,
	) -> Option<CommitTags> {
		let tags = self.list.tags();

		commit.and_then(|commit| {
			tags.and_then(|tags| tags.get(commit).cloned())
		})
	}

	///
	pub fn select_commit(&mut self, id: CommitId) -> Result<()> {
		self.list.select_commit(id)
	}

	fn is_repo_state_clean(&self) -> bool {
		matches!(
			sync::repo_state(&self.repo.borrow()),
			Ok(sync::RepoState::Clean)
		)
	}

	fn request_revert_commit(&self) -> EventState {
		let Some(entry) = self.list.selected_entry() else {
			return EventState::NotConsumed;
		};
		match sync::repo_state(&self.repo.borrow()) {
			Ok(sync::RepoState::Clean) => {
				self.queue.push(InternalEvent::ConfirmAction(
					Action::RevertCommit {
						id: entry.id,
						subject: entry.msg.to_string(),
					},
				));
				EventState::Consumed
			}
			Ok(_) => EventState::NotConsumed,
			Err(err) => {
				self.queue.push(InternalEvent::ShowErrorMsg(
					format!("revert error:\n{err}"),
				));
				EventState::Consumed
			}
		}
	}

	fn request_checkout_commit(&self) -> EventState {
		let Some(entry) = self.list.selected_entry() else {
			return EventState::NotConsumed;
		};
		match sync::repo_state(&self.repo.borrow()) {
			Ok(sync::RepoState::Clean) => {
				self.queue.push(InternalEvent::ConfirmAction(
					Action::CheckoutCommit {
						id: entry.id,
						subject: entry.msg.to_string(),
					},
				));
				EventState::Consumed
			}
			Ok(_) => EventState::NotConsumed,
			Err(err) => {
				self.queue.push(InternalEvent::ShowErrorMsg(
					format!("failed to checkout commit:\n{err}"),
				));
				EventState::Consumed
			}
		}
	}

	fn inspect_commit(&self) {
		if let Some(commit_id) = self.selected_commit() {
			let tags =
				self.selected_commit_tags(Some(commit_id).as_ref());
			self.queue.push(InternalEvent::OpenPopup(
				StackablePopupOpen::InspectCommit(
					InspectCommitOpen::new_with_tags(commit_id, tags),
				),
			));
		}
	}

	pub fn search(&mut self, options: LogFilterSearchOptions) {
		if !self.can_start_search() {
			return;
		}

		if matches!(
			self.search,
			LogSearch::Off | LogSearch::Results(_)
		) {
			log::info!("start search: {options:?}");

			let filter = filter_commit_by_search(
				LogFilterSearch::new(options.clone()),
			);

			let cancellation_flag = Arc::new(AtomicBool::new(false));

			let job = AsyncSingleJob::new(self.sender.clone());
			job.spawn(AsyncCommitFilterJob::new(
				self.repo.borrow().clone(),
				self.list.copy_items(),
				filter,
				Arc::clone(&cancellation_flag),
			));

			self.search = LogSearch::Searching(
				job,
				options,
				None,
				Arc::clone(&cancellation_flag),
			);

			self.list.set_highlighting(None);
		}
	}

	fn cancel_search(&mut self) -> bool {
		if let LogSearch::Searching(_, _, _, cancellation_flag) =
			&self.search
		{
			cancellation_flag.store(true, Ordering::Relaxed);
			self.list.set_highlighting(None);
			return true;
		}

		false
	}

	fn update_search_state(&mut self) {
		match &mut self.search {
			LogSearch::Off | LogSearch::Results(_) => (),
			LogSearch::Searching(
				search,
				options,
				progress,
				cancel,
			) => {
				if search.is_pending() {
					//update progress
					*progress = search.progress();
				} else if let Some(search) = search
					.take_last()
					.and_then(|search| search.result())
				{
					match search {
						Ok(search) => {
							let was_aborted =
								cancel.load(Ordering::Relaxed);

							self.search = if was_aborted {
								LogSearch::Off
							} else {
								self.list.set_highlighting(Some(
									Rc::new(
										search
											.result
											.into_iter()
											.collect::<IndexSet<_>>(),
									),
								));

								LogSearch::Results(LogSearchResult {
									options: options.clone(),
									duration: search.duration,
								})
							};
						}
						Err(err) => {
							self.queue.push(
								InternalEvent::ShowErrorMsg(format!(
									"search error: {err}",
								)),
							);

							self.search = LogSearch::Off;
						}
					}
				}
			}
		}
	}

	const fn is_in_search_mode(&self) -> bool {
		!matches!(self.search, LogSearch::Off)
	}

	fn draw_search(&self, f: &mut Frame, area: Rect) {
		let (text, title) = match &self.search {
			LogSearch::Searching(_, options, progress, _) => (
				format!("'{}'", options.search_pattern.clone()),
				format!(
					"({}%)",
					progress
						.map(|progress| progress.progress)
						.unwrap_or_default()
				),
			),
			LogSearch::Results(results) => {
				let info = self.list.highlighted_selection_info();

				(
					format!(
						"'{}' (duration: {:?})",
						results.options.search_pattern.clone(),
						results.duration,
					),
					format!(
						"({}/{})",
						(info.0 + 1).min(info.1),
						info.1
					),
				)
			}
			LogSearch::Off => (String::new(), String::new()),
		};

		f.render_widget(
			Paragraph::new(text)
				.block(
					Block::default()
						.title(Span::styled(
							format!(
								"{} {}",
								strings::POPUP_TITLE_LOG_SEARCH,
								title
							),
							self.theme.title(true),
						))
						.borders(Borders::ALL)
						.border_style(Theme::attention_block()),
				)
				.alignment(Alignment::Left),
			area,
		);
	}

	const fn can_close_search(&self) -> bool {
		self.is_in_search_mode() && !self.is_search_pending()
	}

	fn can_start_search(&self) -> bool {
		!self.git_log.is_pending() && !self.is_search_pending()
	}

	fn push_commit_commands(
		&self,
		out: &mut Vec<CommandInfo>,
		force_all: bool,
	) {
		let repo_state_clean =
			(self.visible || force_all) && self.is_repo_state_clean();
		let has_selection = self.selected_commit().is_some();
		let show_not_searching =
			(self.visible && !self.is_search_pending()) || force_all;

		out.push(CommandInfo::new(
			strings::commands::log_checkout_commit(&self.key_config),
			has_selection && repo_state_clean,
			show_not_searching,
		));
		out.push(CommandInfo::new(
			strings::commands::open_tags_popup(&self.key_config),
			true,
			self.visible || force_all,
		));
		out.push(CommandInfo::new(
			strings::commands::push_tags(&self.key_config),
			true,
			self.visible || force_all,
		));
		out.push(CommandInfo::new(
			strings::commands::inspect_file_tree(&self.key_config),
			has_selection,
			show_not_searching,
		));
		out.push(CommandInfo::new(
			strings::commands::revert_commit(&self.key_config),
			has_selection && repo_state_clean,
			show_not_searching,
		));
		out.push(CommandInfo::new(
			strings::commands::log_reset_commit(&self.key_config),
			has_selection,
			show_not_searching,
		));
		out.push(CommandInfo::new(
			strings::commands::log_reword_commit(&self.key_config),
			has_selection,
			show_not_searching,
		));
		out.push(CommandInfo::new(
			strings::commands::log_find_commit(&self.key_config),
			self.can_start_search(),
			self.visible || force_all,
		));
	}
}

impl DrawableComponent for Revlog {
	fn draw(&self, f: &mut Frame, area: Rect) -> Result<()> {
		let area = if self.is_in_search_mode() {
			Layout::default()
				.direction(Direction::Vertical)
				.constraints(
					[Constraint::Min(1), Constraint::Length(3)]
						.as_ref(),
				)
				.split(area)
		} else {
			Rc::new([area])
		};

		let chunks = Layout::default()
			.direction(Direction::Horizontal)
			.constraints(
				[
					Constraint::Percentage(60),
					Constraint::Percentage(40),
				]
				.as_ref(),
			)
			.split(area[0]);

		if self.commit_details.is_visible() {
			self.list.draw(f, chunks[0])?;
			self.commit_details.draw(f, chunks[1])?;
		} else {
			self.list.draw(f, area[0])?;
		}

		if self.is_in_search_mode() {
			self.draw_search(f, area[1]);
		}

		Ok(())
	}
}

impl Component for Revlog {
	//TODO: cleanup
	#[allow(clippy::too_many_lines)]
	fn event(&mut self, ev: &Event) -> Result<EventState> {
		if self.visible {
			let event_used = self.list.event(ev)?;

			if event_used.is_consumed() {
				self.update()?;
				return Ok(EventState::Consumed);
			} else if let Event::Key(k) = ev {
				if key_match(k, self.key_config.keys.enter) {
					self.commit_details.toggle_visible()?;
					self.update()?;
					return Ok(EventState::Consumed);
				} else if key_match(
					k,
					self.key_config.keys.exit_popup,
				) {
					if self.is_search_pending() {
						self.cancel_search();
					} else if self.can_close_search() {
						self.list.set_highlighting(None);
						self.search = LogSearch::Off;
					} else if self.commit_details.is_visible() {
						self.commit_details.hide();
					}
					return Ok(EventState::Consumed);
				} else if key_match(k, self.key_config.keys.copy) {
					try_or_popup!(
						self,
						strings::POPUP_FAIL_COPY,
						self.list.copy_commit_hash()
					);
					return Ok(EventState::Consumed);
				} else if key_match(k, self.key_config.keys.push) {
					self.queue.push(InternalEvent::PushTags);
					return Ok(EventState::Consumed);
				} else if key_match(
					k,
					self.key_config.keys.log_tag_commit,
				) {
					return self.selected_commit().map_or(
						Ok(EventState::NotConsumed),
						|id| {
							self.queue
								.push(InternalEvent::TagCommit(id));
							Ok(EventState::Consumed)
						},
					);
				} else if key_match(
					k,
					self.key_config.keys.move_right,
				) && self.commit_details.is_visible()
				{
					self.inspect_commit();
					return Ok(EventState::Consumed);
				} else if key_match(
					k,
					self.key_config.keys.select_branch,
				) && !self.is_search_pending()
				{
					self.queue.push(InternalEvent::SelectBranch);
					return Ok(EventState::Consumed);
				} else if key_match(
					k,
					self.key_config.keys.status_reset_item,
				) && !self.is_search_pending()
				{
					return Ok(self.request_revert_commit());
				} else if key_match(
					k,
					self.key_config.keys.log_checkout_commit,
				) && !self.is_search_pending()
				{
					return Ok(self.request_checkout_commit());
				} else if key_match(
					k,
					self.key_config.keys.open_file_tree,
				) && !self.is_search_pending()
				{
					return self.selected_commit().map_or(
						Ok(EventState::NotConsumed),
						|id| {
							self.queue.push(
								InternalEvent::OpenPopup(
									StackablePopupOpen::FileTree(
										FileTreeOpen::new(id),
									),
								),
							);
							Ok(EventState::Consumed)
						},
					);
				} else if key_match(k, self.key_config.keys.tags) {
					self.queue.push(InternalEvent::Tags);
					return Ok(EventState::Consumed);
				} else if key_match(
					k,
					self.key_config.keys.log_reset_commit,
				) && !self.is_search_pending()
				{
					return self.selected_commit().map_or(
						Ok(EventState::NotConsumed),
						|id| {
							self.queue.push(
								InternalEvent::OpenResetPopup(id),
							);
							Ok(EventState::Consumed)
						},
					);
				} else if key_match(
					k,
					self.key_config.keys.log_reword_commit,
				) && !self.is_search_pending()
				{
					return self.selected_commit().map_or(
						Ok(EventState::NotConsumed),
						|id| {
							self.queue.push(
								InternalEvent::RewordCommit(id),
							);
							Ok(EventState::Consumed)
						},
					);
				} else if key_match(k, self.key_config.keys.log_find)
					&& self.can_start_search()
				{
					self.queue
						.push(InternalEvent::OpenLogSearchPopup);
					return Ok(EventState::Consumed);
				} else if key_match(
					k,
					self.key_config.keys.compare_commits,
				) && self.list.marked_count() > 0
					&& !self.is_search_pending()
				{
					if self.list.marked_count() == 1 {
						// compare against head
						self.queue.push(InternalEvent::OpenPopup(
							StackablePopupOpen::CompareCommits(
								InspectCommitOpen::new(
									self.list.marked_commits()[0],
								),
							),
						));
						return Ok(EventState::Consumed);
					} else if self.list.marked_count() == 2 {
						//compare two marked commits
						let marked = self.list.marked_commits();
						self.queue.push(InternalEvent::OpenPopup(
							StackablePopupOpen::CompareCommits(
								InspectCommitOpen {
									commit_id: marked[0],
									compare_id: Some(marked[1]),
									tags: None,
								},
							),
						));
						return Ok(EventState::Consumed);
					}
				}
			}
		}

		Ok(EventState::NotConsumed)
	}

	fn commands(
		&self,
		out: &mut Vec<CommandInfo>,
		force_all: bool,
	) -> CommandBlocking {
		if self.visible || force_all {
			self.list.commands(out, force_all);
		}

		out.push(
			CommandInfo::new(
				strings::commands::log_close_search(&self.key_config),
				true,
				(self.visible
					&& (self.can_close_search()
						|| self.is_search_pending()))
					|| force_all,
			)
			.order(order::PRIORITY),
		);

		out.push(CommandInfo::new(
			strings::commands::log_details_toggle(&self.key_config),
			true,
			self.visible,
		));

		out.push(CommandInfo::new(
			strings::commands::commit_details_open(&self.key_config),
			true,
			(self.visible && self.commit_details.is_visible())
				|| force_all,
		));

		out.push(CommandInfo::new(
			strings::commands::open_branch_select_popup(
				&self.key_config,
			),
			true,
			(self.visible && !self.is_search_pending()) || force_all,
		));

		out.push(CommandInfo::new(
			strings::commands::compare_with_head(&self.key_config),
			self.list.marked_count() == 1,
			(self.visible
				&& !self.is_search_pending()
				&& self.list.marked_count() <= 1)
				|| force_all,
		));

		out.push(CommandInfo::new(
			strings::commands::compare_commits(&self.key_config),
			true,
			(self.visible
				&& !self.is_search_pending()
				&& self.list.marked_count() == 2)
				|| force_all,
		));

		out.push(CommandInfo::new(
			strings::commands::copy_hash(&self.key_config),
			self.selected_commit().is_some(),
			self.visible || force_all,
		));

		out.push(CommandInfo::new(
			strings::commands::log_tag_commit(&self.key_config),
			self.selected_commit().is_some(),
			self.visible || force_all,
		));

		self.push_commit_commands(out, force_all);

		visibility_blocking(self)
	}

	fn is_visible(&self) -> bool {
		self.visible
	}

	fn hide(&mut self) {
		self.visible = false;
		self.git_log.set_background();
	}

	fn show(&mut self) -> Result<()> {
		self.visible = true;

		self.git_local_branches.spawn(AsyncBranchesJob::new(
			self.repo.borrow().clone(),
			true,
		));

		self.git_remote_branches.spawn(AsyncBranchesJob::new(
			self.repo.borrow().clone(),
			false,
		));

		self.update()?;

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crossbeam_channel::{unbounded, Receiver};
	use std::cell::RefCell;

	fn env_for_repo(
		path: &std::path::Path,
	) -> (Environment, Receiver<AsyncGitNotification>) {
		let mut env = Environment::test_env();
		let (tx, rx) = unbounded();
		env.sender_git = tx;
		env.repo =
			RefCell::new(sync::RepoPath::Path(path.to_path_buf()));
		(env, rx)
	}

	fn drain_log(
		revlog: &mut Revlog,
		rx: &Receiver<AsyncGitNotification>,
	) {
		while revlog.git_log.is_pending() {
			if rx.recv_timeout(Duration::from_secs(2)).is_err() {
				break;
			}
		}
		revlog.update().unwrap();
	}

	#[test]
	fn t7n_reload_missing_commit_clamps() {
		let (td, repo) = git2_testing::repo_init();
		let sig = repo.signature().unwrap();
		let tree = repo
			.find_tree(repo.index().unwrap().write_tree().unwrap())
			.unwrap();
		let c0 = repo.head().unwrap().peel_to_commit().unwrap();
		let c1_oid = repo
			.commit(Some("HEAD"), &sig, &sig, "c1", &tree, &[&c0])
			.unwrap();
		let c1 = repo.find_commit(c1_oid).unwrap();
		let c2_oid = repo
			.commit(Some("HEAD"), &sig, &sig, "c2", &tree, &[&c1])
			.unwrap();
		let c2 = repo.find_commit(c2_oid).unwrap();
		let c3_oid = repo
			.commit(Some("HEAD"), &sig, &sig, "c3", &tree, &[&c2])
			.unwrap();
		let c3 = repo.find_commit(c3_oid).unwrap();
		let _c4_oid = repo
			.commit(Some("HEAD"), &sig, &sig, "c4", &tree, &[&c3])
			.unwrap();

		let (env, rx) = env_for_repo(td.path());
		let mut revlog = Revlog::new(&env);
		revlog.show().unwrap();
		drain_log(&mut revlog, &rx);

		// History is [c4, c3, c2, c1, c0]. Select c2 (index 2).
		revlog.select_commit(CommitId::new(c2_oid)).unwrap();
		revlog.update().unwrap();
		assert_eq!(
			revlog.selected_commit(),
			Some(CommitId::new(c2_oid))
		);

		// Rewrite HEAD to a 2-commit history [c1_alt, c0] where c2 is gone.
		let c1_alt_oid = repo
			.commit(None, &sig, &sig, "c1_alt", &tree, &[&c0])
			.unwrap();
		let head_ref =
			repo.head().unwrap().name().unwrap().to_owned();
		repo.reference(&head_ref, c1_alt_oid, true, "rewrite")
			.unwrap();

		revlog.update().unwrap();
		drain_log(&mut revlog, &rx);

		// Old index 2 clamped to len - 1 = 1 -> c0.
		assert_eq!(
			revlog.selected_commit(),
			Some(CommitId::new(c0.id()))
		);

		// Empty history after clear is safe.
		revlog.list.clear();
		assert!(revlog.selected_commit().is_none());
	}

	fn drain_all(
		revlog: &mut Revlog,
		rx: &Receiver<AsyncGitNotification>,
	) {
		while revlog.any_work_pending() {
			if rx.recv_timeout(Duration::from_secs(2)).is_err() {
				break;
			}
		}
		revlog.update().unwrap();
	}

	#[test]
	fn t8c_no_selection_no_prompt() {
		let (td, repo) = git2_testing::repo_init();
		let sig = repo.signature().unwrap();
		let tree = repo
			.find_tree(repo.index().unwrap().write_tree().unwrap())
			.unwrap();
		let c0 = repo.head().unwrap().peel_to_commit().unwrap();
		let c1_oid = repo
			.commit(Some("HEAD"), &sig, &sig, "c1", &tree, &[&c0])
			.unwrap();

		let (env, rx) = env_for_repo(td.path());
		let mut revlog = Revlog::new(&env);
		revlog.visible = true;

		let revert_key = Event::Key(crossterm::event::KeyEvent::new(
			env.key_config.keys.status_reset_item.code,
			env.key_config.keys.status_reset_item.modifiers,
		));
		let checkout_key =
			Event::Key(crossterm::event::KeyEvent::new(
				env.key_config.keys.log_checkout_commit.code,
				env.key_config.keys.log_checkout_commit.modifiers,
			));

		// Empty list: neither key queues a confirmation.
		revlog.event(&revert_key).unwrap();
		revlog.event(&checkout_key).unwrap();
		assert!(env.queue.pop().is_none());

		// Load commits, select non-zero commit, then clear to enter pending_restore.
		revlog.show().unwrap();
		drain_all(&mut revlog, &rx);
		revlog.select_commit(CommitId::new(c0.id())).unwrap();
		revlog.update().unwrap();
		revlog.list.clear();
		// Feed only the newest commit so restore for c0 is still pending.
		revlog.list.refresh_extend_data(vec![CommitId::new(c1_oid)]);
		assert!(revlog.selected_commit().is_none());

		revlog.event(&revert_key).unwrap();
		revlog.event(&checkout_key).unwrap();
		assert!(env.queue.pop().is_none());
		drain_all(&mut revlog, &rx);
	}

	fn set_pending_state(
		git_dir: &std::path::Path,
		kind: &str,
		oid: &str,
	) {
		let _ = std::fs::remove_file(git_dir.join("MERGE_HEAD"));
		let _ = std::fs::remove_file(git_dir.join("REVERT_HEAD"));
		let _ =
			std::fs::remove_file(git_dir.join("CHERRY_PICK_HEAD"));
		let _ = std::fs::remove_dir_all(git_dir.join("rebase-merge"));
		match kind {
			"merge" => std::fs::write(
				git_dir.join("MERGE_HEAD"),
				format!("{oid}\n"),
			)
			.unwrap(),
			"revert" => std::fs::write(
				git_dir.join("REVERT_HEAD"),
				format!("{oid}\n"),
			)
			.unwrap(),
			"rebase" => {
				std::fs::create_dir_all(git_dir.join("rebase-merge"))
					.unwrap();
			}
			"cherry-pick" => std::fs::write(
				git_dir.join("CHERRY_PICK_HEAD"),
				format!("{oid}\n"),
			)
			.unwrap(),
			_ => unreachable!(),
		}
	}

	#[test]
	fn t8c_pending_state_blocks() {
		let (td, repo) = git2_testing::repo_init();
		let workdir = repo.workdir().unwrap().to_path_buf();
		std::fs::write(workdir.join("a.txt"), "one").unwrap();
		let mut idx = repo.index().unwrap();
		idx.add_path(std::path::Path::new("a.txt")).unwrap();
		idx.write().unwrap();
		let tree = repo.find_tree(idx.write_tree().unwrap()).unwrap();
		let sig = repo.signature().unwrap();
		let c0 = repo.head().unwrap().peel_to_commit().unwrap();
		let c1_oid = repo
			.commit(Some("HEAD"), &sig, &sig, "c1", &tree, &[&c0])
			.unwrap()
			.to_string();

		let (env, rx) = env_for_repo(td.path());
		let mut revlog = Revlog::new(&env);
		revlog.show().unwrap();
		drain_all(&mut revlog, &rx);

		let git_dir = repo.path().to_path_buf();
		let revert_key = Event::Key(crossterm::event::KeyEvent::new(
			env.key_config.keys.status_reset_item.code,
			env.key_config.keys.status_reset_item.modifiers,
		));
		let checkout_key =
			Event::Key(crossterm::event::KeyEvent::new(
				env.key_config.keys.log_checkout_commit.code,
				env.key_config.keys.log_checkout_commit.modifiers,
			));
		let revert_hint =
			strings::commands::revert_commit(&env.key_config).name;
		let checkout_hint =
			strings::commands::log_checkout_commit(&env.key_config)
				.name;

		for label in ["merge", "revert", "rebase", "cherry-pick"] {
			set_pending_state(&git_dir, label, &c1_oid);
			assert_ne!(
				sync::repo_state(&env.repo.borrow()).unwrap(),
				sync::RepoState::Clean,
				"expected non-clean state for {label}"
			);

			let mut cmds = Vec::new();
			revlog.commands(&mut cmds, false);
			assert!(
				!cmds
					.iter()
					.find(|c| c.text.name == revert_hint)
					.is_some_and(|c| c.enabled),
				"revert hint must be disabled during {label}"
			);
			assert!(
				!cmds
					.iter()
					.find(|c| c.text.name == checkout_hint)
					.is_some_and(|c| c.enabled),
				"checkout hint must be disabled during {label}"
			);
			assert!(!revlog
				.event(&revert_key)
				.unwrap()
				.is_consumed());
			assert!(!revlog
				.event(&checkout_key)
				.unwrap()
				.is_consumed());
			assert!(env.queue.pop().is_none());
			assert!(!repo.head_detached().unwrap());
		}
	}
}
