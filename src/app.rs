use crate::{
	accessors,
	args::CliArgs,
	cmdbar::CommandBar,
	components::{
		command_pump, event_pump, CommandInfo, Component,
		DrawableComponent, EventState, FuzzyFinderTarget,
	},
	input::{Input, InputEvent, InputState},
	keys::{key_match, KeyConfig, SharedKeyConfig},
	options::{Options, SharedOptions},
	popup_stack::PopupStack,
	popups::{
		AppOption, BlameFilePopup, BranchListPopup,
		CheckoutOptionPopup, CommitPopup, CompareCommitsPopup,
		ConfirmPopup, CreateBranchPopup, CreateRemotePopup,
		ExternalEditorPopup, FetchPopup, FileRevlogPopup,
		FuzzyFindPopup, GotoLinePopup, HelpPopup, InspectCommitPopup,
		LogSearchPopupPopup, MsgPopup, OptionsPopup, PullPopup,
		PushPopup, PushTagsPopup, RemoteListPopup, RenameBranchPopup,
		RenameRemotePopup, ResetPopup, RevisionFilesPopup,
		StashMsgPopup, SubmodulesListPopup, TagCommitPopup,
		TagListPopup, UpdateRemoteUrlPopup,
	},
	queue::{
		Action, AppTabs, InternalEvent, NeedsUpdate, Queue,
		StackablePopupOpen,
	},
	setup_popups,
	strings::{self, ellipsis_trim_start, order},
	tabs::{FilesTab, Revlog, StashList, Stashing, Status},
	try_or_popup,
	ui::style::{SharedTheme, Theme},
	AsyncAppNotification, AsyncNotification,
};
use anyhow::{bail, Result};
use asyncgit::{
	sync::{
		self,
		utils::{repo_work_dir, undo_last_commit},
		RepoPath, RepoPathRef,
	},
	AsyncGitNotification, PushType,
};
use crossbeam_channel::Sender;
use crossterm::event::{Event, KeyEvent};
use ratatui::{
	layout::{
		Alignment, Constraint, Direction, Layout, Margin, Rect,
	},
	text::{Line, Span},
	widgets::{Block, Borders, Paragraph, Tabs},
	Frame,
};
use std::{
	cell::{Cell, RefCell},
	path::{Path, PathBuf},
	rc::Rc,
};
use unicode_width::UnicodeWidthStr;

#[derive(Clone)]
pub enum QuitState {
	None,
	Close,
	OpenSubmodule(RepoPath),
}

/// the main app type
pub struct App {
	repo: RepoPathRef,
	do_quit: QuitState,
	help_popup: HelpPopup,
	msg_popup: MsgPopup,
	confirm_popup: ConfirmPopup,
	commit_popup: CommitPopup,
	blame_file_popup: BlameFilePopup,
	file_revlog_popup: FileRevlogPopup,
	stashmsg_popup: StashMsgPopup,
	inspect_commit_popup: InspectCommitPopup,
	compare_commits_popup: CompareCommitsPopup,
	external_editor_popup: ExternalEditorPopup,
	revision_files_popup: RevisionFilesPopup,
	fuzzy_find_popup: FuzzyFindPopup,
	log_search_popup: LogSearchPopupPopup,
	push_popup: PushPopup,
	push_tags_popup: PushTagsPopup,
	pull_popup: PullPopup,
	fetch_popup: FetchPopup,
	tag_commit_popup: TagCommitPopup,
	create_branch_popup: CreateBranchPopup,
	create_remote_popup: CreateRemotePopup,
	rename_remote_popup: RenameRemotePopup,
	update_remote_url_popup: UpdateRemoteUrlPopup,
	remotes_popup: RemoteListPopup,
	rename_branch_popup: RenameBranchPopup,
	select_branch_popup: BranchListPopup,
	options_popup: OptionsPopup,
	submodule_popup: SubmodulesListPopup,
	tags_popup: TagListPopup,
	reset_popup: ResetPopup,
	checkout_option_popup: CheckoutOptionPopup,
	cmdbar: RefCell<CommandBar>,
	tab: usize,
	revlog: Revlog,
	status_tab: Status,
	stashing_tab: Stashing,
	stashlist_tab: StashList,
	files_tab: FilesTab,
	queue: Queue,
	theme: SharedTheme,
	key_config: SharedKeyConfig,
	input: Input,
	popup_stack: PopupStack,
	options: SharedOptions,
	repo_path_text: String,
	goto_line_popup: GotoLinePopup,

	// "Flags"
	requires_redraw: Cell<bool>,
	file_to_open: Option<String>,
}

pub struct Environment {
	pub queue: Queue,
	pub theme: SharedTheme,
	pub key_config: SharedKeyConfig,
	pub repo: RepoPathRef,
	pub options: SharedOptions,
	pub sender_git: Sender<AsyncGitNotification>,
	pub sender_app: Sender<AsyncAppNotification>,
}

/// The need to construct a "whatever" environment only arises in testing right now
#[cfg(test)]
impl Environment {
	pub fn test_env() -> Self {
		use crossbeam_channel::unbounded;
		Self {
			queue: Queue::new(),
			theme: Rc::default(),
			key_config: Rc::default(),
			repo: RefCell::new(RepoPath::Path(PathBuf::default())),
			options: Rc::new(RefCell::new(Options::test_env())),
			sender_git: unbounded().0,
			sender_app: unbounded().0,
		}
	}
}

// public interface
impl App {
	///
	#[allow(clippy::too_many_lines)]
	pub fn new(
		cliargs: CliArgs,
		sender_git: Sender<AsyncGitNotification>,
		sender_app: Sender<AsyncAppNotification>,
		input: Input,
		theme: Theme,
		key_config: KeyConfig,
	) -> Result<Self> {
		let repo = RefCell::new(cliargs.repo_path.clone());
		log::trace!("open repo at: {repo:?}");

		let repo_path_text =
			repo_work_dir(&repo.borrow()).unwrap_or_default();

		let env = Environment {
			queue: Queue::new(),
			theme: Rc::new(theme),
			key_config: Rc::new(key_config),
			options: Options::new(repo.clone()),
			repo,
			sender_git,
			sender_app,
		};

		let mut select_file: Option<PathBuf> = None;
		let tab = if let Some(file) = cliargs.select_file {
			// convert to relative git path
			if let Ok(abs) = file.canonicalize() {
				if let Ok(path) = abs.strip_prefix(
					env.repo.borrow().gitpath().canonicalize()?,
				) {
					select_file = Some(Path::new(".").join(path));
				}
			}
			2
		} else {
			env.options.borrow().current_tab()
		};

		let mut app = Self {
			input,
			confirm_popup: ConfirmPopup::new(&env),
			commit_popup: CommitPopup::new(&env),
			blame_file_popup: BlameFilePopup::new(
				&env,
				&strings::blame_title(&env.key_config),
			),
			file_revlog_popup: FileRevlogPopup::new(&env),
			revision_files_popup: RevisionFilesPopup::new(&env),
			stashmsg_popup: StashMsgPopup::new(&env),
			inspect_commit_popup: InspectCommitPopup::new(&env),
			compare_commits_popup: CompareCommitsPopup::new(&env),
			external_editor_popup: ExternalEditorPopup::new(&env),
			push_popup: PushPopup::new(&env),
			push_tags_popup: PushTagsPopup::new(&env),
			reset_popup: ResetPopup::new(&env),
			pull_popup: PullPopup::new(&env),
			fetch_popup: FetchPopup::new(&env),
			tag_commit_popup: TagCommitPopup::new(&env),
			create_branch_popup: CreateBranchPopup::new(&env),
			create_remote_popup: CreateRemotePopup::new(&env),
			rename_remote_popup: RenameRemotePopup::new(&env),
			update_remote_url_popup: UpdateRemoteUrlPopup::new(&env),
			remotes_popup: RemoteListPopup::new(&env),
			rename_branch_popup: RenameBranchPopup::new(&env),
			select_branch_popup: BranchListPopup::new(&env),
			tags_popup: TagListPopup::new(&env),
			options_popup: OptionsPopup::new(&env),
			submodule_popup: SubmodulesListPopup::new(&env),
			log_search_popup: LogSearchPopupPopup::new(&env),
			fuzzy_find_popup: FuzzyFindPopup::new(&env),
			do_quit: QuitState::None,
			cmdbar: RefCell::new(CommandBar::new(
				env.theme.clone(),
				env.key_config.clone(),
			)),
			help_popup: HelpPopup::new(&env),
			msg_popup: MsgPopup::new(&env),
			revlog: Revlog::new(&env),
			status_tab: Status::new(&env),
			stashing_tab: Stashing::new(&env),
			stashlist_tab: StashList::new(&env),
			files_tab: FilesTab::new(&env, select_file),
			checkout_option_popup: CheckoutOptionPopup::new(&env),
			goto_line_popup: GotoLinePopup::new(&env),
			tab: 0,
			queue: env.queue,
			theme: env.theme,
			options: env.options,
			key_config: env.key_config,
			requires_redraw: Cell::new(false),
			file_to_open: None,
			repo: env.repo,
			repo_path_text,
			popup_stack: PopupStack::default(),
		};

		app.set_tab(tab)?;

		Ok(app)
	}

	///
	pub fn draw(&self, f: &mut Frame) -> Result<()> {
		let fsize = f.area();

		self.cmdbar.borrow_mut().refresh_width(fsize.width);

		let chunks_main = Layout::default()
			.direction(Direction::Vertical)
			.constraints(
				[
					Constraint::Length(2),
					Constraint::Min(2),
					Constraint::Length(self.cmdbar.borrow().height()),
				]
				.as_ref(),
			)
			.split(fsize);

		self.cmdbar.borrow().draw(f, chunks_main[2]);

		self.draw_top_bar(f, chunks_main[0]);

		//TODO: component property + a macro `fullscreen_popup_open!`
		// to make this scale better?
		let fullscreen_popup_open =
			self.revision_files_popup.is_visible()
				|| self.inspect_commit_popup.is_visible()
				|| self.compare_commits_popup.is_visible()
				|| self.blame_file_popup.is_visible()
				|| self.file_revlog_popup.is_visible();

		if !fullscreen_popup_open {
			//TODO: macro because of generic draw call
			match self.tab {
				0 => self.status_tab.draw(f, chunks_main[1])?,
				1 => self.revlog.draw(f, chunks_main[1])?,
				2 => self.files_tab.draw(f, chunks_main[1])?,
				3 => self.stashing_tab.draw(f, chunks_main[1])?,
				4 => self.stashlist_tab.draw(f, chunks_main[1])?,
				_ => bail!("unknown tab"),
			}
		}

		self.draw_popups(f)?;

		Ok(())
	}

	///
	pub fn event(&mut self, ev: InputEvent) -> Result<()> {
		log::trace!("event: {ev:?}");

		if let InputEvent::Input(ev) = ev {
			if self.check_hard_exit(&ev) || self.check_quit(&ev) {
				return Ok(());
			}

			let mut flags = NeedsUpdate::empty();

			let popup_open = self.any_popup_visible();
			let pump_result =
				self.dispatch_component_event(&ev, popup_open);
			if let Err(err) = &pump_result {
				log::error!("event handler failed: {err:?}");
				self.queue.push(InternalEvent::ShowErrorMsg(
					format!("{err}"),
				));
				flags.insert(NeedsUpdate::ALL);
			} else if matches!(
				pump_result,
				Ok(state) if state.is_consumed()
			) {
				flags.insert(NeedsUpdate::COMMANDS);
			} else if let Event::Key(k) = &ev {
				let popup_open =
					popup_open || self.any_popup_visible();
				let new_flags = if !popup_open
					&& key_match(k, self.key_config.keys.tab_toggle)
				{
					self.toggle_tabs(false)?;
					NeedsUpdate::COMMANDS
				} else if !popup_open
					&& key_match(
						k,
						self.key_config.keys.tab_toggle_reverse,
					) {
					self.toggle_tabs(true)?;
					NeedsUpdate::COMMANDS
				} else if !popup_open
					&& (key_match(k, self.key_config.keys.tab_status)
						|| key_match(k, self.key_config.keys.tab_log)
						|| key_match(
							k,
							self.key_config.keys.tab_files,
						) || key_match(
						k,
						self.key_config.keys.tab_stashing,
					) || key_match(
						k,
						self.key_config.keys.tab_stashes,
					)) {
					self.switch_tab(k)?;
					NeedsUpdate::COMMANDS
				} else if key_match(
					k,
					self.key_config.keys.cmd_bar_toggle,
				) {
					self.cmdbar.borrow_mut().toggle_more();
					NeedsUpdate::empty()
				} else if !popup_open
					&& key_match(k, self.key_config.keys.open_options)
				{
					self.options_popup.show()?;
					NeedsUpdate::ALL
				} else {
					NeedsUpdate::empty()
				};

				flags.insert(new_flags);
			}

			self.process_queue(flags)?;
		} else if let InputEvent::State(polling_state) = ev {
			self.external_editor_popup.hide();
			if matches!(polling_state, InputState::Paused) {
				let result =
					if let Some(path) = self.file_to_open.take() {
						ExternalEditorPopup::open_file_in_editor(
							&self.repo.borrow(),
							Path::new(&path),
						)
					} else {
						self.status_tab.get_files_changes().and_then(
							|changes| {
								self.commit_popup.show_editor(changes)
							},
						)
					};

				if let Err(e) = result {
					let msg =
						format!("failed to launch editor:\n{e}");
					log::error!("{}", msg.as_str());
					self.msg_popup.show_error(msg.as_str())?;
				}

				self.requires_redraw.set(true);
				self.input.set_polling(true);
			}
		}

		Ok(())
	}

	//TODO: do we need this?
	/// forward ticking to components that require it
	pub fn update(&mut self) -> Result<()> {
		log::trace!("update");

		self.commit_popup.update();
		self.status_tab.update()?;
		self.revlog.update()?;
		self.files_tab.update();
		self.stashing_tab.update()?;
		self.stashlist_tab.update()?;
		self.reset_popup.update()?;

		self.update_commands();

		Ok(())
	}

	///
	pub fn update_async(
		&mut self,
		ev: AsyncNotification,
	) -> Result<()> {
		log::trace!("update_async: {ev:?}");

		if let AsyncNotification::Git(ev) = ev {
			self.status_tab.update_git(ev)?;
			self.stashing_tab.update_git(ev)?;
			self.revlog.update_git(ev)?;
			self.file_revlog_popup.update_git(ev)?;
			self.inspect_commit_popup.update_git(ev)?;
			self.compare_commits_popup.update_git(ev)?;
			self.push_popup.update_git(ev)?;
			self.push_tags_popup.update_git(ev)?;
			self.pull_popup.update_git(ev);
			self.fetch_popup.update_git(ev);
			self.select_branch_popup.update_git(ev)?;
		}

		self.files_tab.update_async(ev)?;
		self.blame_file_popup.update_async(ev)?;
		self.revision_files_popup.update(ev)?;
		self.tags_popup.update(ev);

		//TODO: better system for this
		// can we simply process the queue here and everyone just uses the queue to schedule a cmd update?
		self.process_queue(NeedsUpdate::COMMANDS)?;

		Ok(())
	}

	///
	pub fn is_quit(&self) -> bool {
		!matches!(self.do_quit, QuitState::None)
			|| self.input.is_aborted()
	}

	///
	pub fn quit_state(&self) -> QuitState {
		self.do_quit.clone()
	}

	///
	pub fn any_work_pending(&self) -> bool {
		self.status_tab.anything_pending()
			|| self.revlog.any_work_pending()
			|| self.stashing_tab.anything_pending()
			|| self.files_tab.anything_pending()
			|| self.blame_file_popup.any_work_pending()
			|| self.file_revlog_popup.any_work_pending()
			|| self.inspect_commit_popup.any_work_pending()
			|| self.compare_commits_popup.any_work_pending()
			|| self.input.is_state_changing()
			|| self.push_popup.any_work_pending()
			|| self.push_tags_popup.any_work_pending()
			|| self.pull_popup.any_work_pending()
			|| self.fetch_popup.any_work_pending()
			|| self.revision_files_popup.any_work_pending()
			|| self.tags_popup.any_work_pending()
	}

	///
	pub fn requires_redraw(&self) -> bool {
		if self.requires_redraw.get() {
			self.requires_redraw.set(false);
			true
		} else {
			false
		}
	}

	#[cfg(test)]
	pub(crate) fn is_confirm_popup_visible(&self) -> bool {
		self.confirm_popup.is_visible()
	}

	#[cfg(test)]
	pub(crate) fn is_msg_popup_visible(&self) -> bool {
		self.msg_popup.is_visible()
	}

	#[cfg(test)]
	pub(crate) const fn current_tab(&self) -> usize {
		self.tab
	}

	#[cfg(test)]
	pub(crate) const fn revlog_mut(&mut self) -> &mut Revlog {
		&mut self.revlog
	}

	#[cfg(test)]
	pub(crate) fn cmdbar_height(&self) -> u16 {
		self.cmdbar.borrow().height()
	}

	#[cfg(test)]
	pub(crate) fn show_msg_popup(&mut self, msg: &str) {
		self.msg_popup.show_info(msg).unwrap();
		self.update_commands();
	}

	#[cfg(test)]
	pub(crate) fn show_options_popup(&mut self) {
		self.options_popup.show().unwrap();
		self.update_commands();
	}

	#[cfg(test)]
	pub(crate) fn open_commit_popup(&mut self) {
		self.commit_popup.open(None).unwrap();
		self.update_commands();
	}

	#[cfg(test)]
	pub(crate) fn commit_popup_text(&self) -> &str {
		self.commit_popup.text()
	}

	#[cfg(test)]
	pub(crate) fn open_branch_list_popup(&mut self) {
		self.select_branch_popup.open().unwrap();
		self.update_commands();
	}

	#[cfg(test)]
	pub(crate) fn open_goto_line_popup(&mut self) {
		self.goto_line_popup.open(10);
		self.update_commands();
	}

	#[cfg(test)]
	pub(crate) fn log_search_text(&self) -> &str {
		self.log_search_popup.search_text()
	}
}

// private impls
impl App {
	// event order: reverse of the draw order in setup_popups! below (topmost popup first)
	accessors!(
		self,
		[
			goto_line_popup,
			msg_popup,
			confirm_popup,
			options_popup,
			fetch_popup,
			pull_popup,
			push_tags_popup,
			push_popup,
			log_search_popup,
			fuzzy_find_popup,
			revision_files_popup,
			rename_branch_popup,
			create_branch_popup,
			checkout_option_popup,
			reset_popup,
			tags_popup,
			submodule_popup,
			update_remote_url_popup,
			rename_remote_popup,
			create_remote_popup,
			remotes_popup,
			select_branch_popup,
			tag_commit_popup,
			external_editor_popup,
			file_revlog_popup,
			blame_file_popup,
			compare_commits_popup,
			inspect_commit_popup,
			help_popup,
			stashmsg_popup,
			commit_popup,
			revlog,
			status_tab,
			files_tab,
			stashing_tab,
			stashlist_tab
		]
	);

	setup_popups!(
		self,
		[
			commit_popup,
			stashmsg_popup,
			help_popup,
			inspect_commit_popup,
			compare_commits_popup,
			blame_file_popup,
			file_revlog_popup,
			external_editor_popup,
			tag_commit_popup,
			select_branch_popup,
			remotes_popup,
			create_remote_popup,
			rename_remote_popup,
			update_remote_url_popup,
			submodule_popup,
			tags_popup,
			reset_popup,
			checkout_option_popup,
			create_branch_popup,
			rename_branch_popup,
			revision_files_popup,
			fuzzy_find_popup,
			log_search_popup,
			push_popup,
			push_tags_popup,
			pull_popup,
			fetch_popup,
			options_popup,
			confirm_popup,
			msg_popup,
			goto_line_popup
		]
	);

	fn dispatch_component_event(
		&mut self,
		ev: &Event,
		popup_open: bool,
	) -> Result<EventState> {
		if popup_open {
			self.components_mut()
				.into_iter()
				.find(|c| c.is_visible())
				.map_or(Ok(EventState::NotConsumed), |top| {
					top.event(ev)
				})
		} else {
			event_pump(ev, self.components_mut().as_mut_slice())
		}
	}

	fn check_quit(&mut self, ev: &Event) -> bool {
		if self.any_popup_visible() {
			return false;
		}
		if let Event::Key(e) = ev {
			if key_match(e, self.key_config.keys.quit) {
				self.do_quit = QuitState::Close;
				return true;
			}
		}
		false
	}

	fn check_hard_exit(&mut self, ev: &Event) -> bool {
		if let Event::Key(e) = ev {
			if key_match(e, self.key_config.keys.exit) {
				self.do_quit = QuitState::Close;
				return true;
			}
		}
		false
	}

	fn get_tabs(&mut self) -> Vec<&mut dyn Component> {
		vec![
			&mut self.status_tab,
			&mut self.revlog,
			&mut self.files_tab,
			&mut self.stashing_tab,
			&mut self.stashlist_tab,
		]
	}

	fn toggle_tabs(&mut self, reverse: bool) -> Result<()> {
		let tabs_len = self.get_tabs().len();
		let new_tab = if reverse {
			self.tab.wrapping_sub(1).min(tabs_len.saturating_sub(1))
		} else {
			self.tab.saturating_add(1) % tabs_len
		};

		self.set_tab(new_tab)
	}

	fn switch_tab(&mut self, k: &KeyEvent) -> Result<()> {
		if key_match(k, self.key_config.keys.tab_status) {
			self.switch_to_tab(&AppTabs::Status)?;
		} else if key_match(k, self.key_config.keys.tab_log) {
			self.switch_to_tab(&AppTabs::Log)?;
		} else if key_match(k, self.key_config.keys.tab_files) {
			self.switch_to_tab(&AppTabs::Files)?;
		} else if key_match(k, self.key_config.keys.tab_stashing) {
			self.switch_to_tab(&AppTabs::Stashing)?;
		} else if key_match(k, self.key_config.keys.tab_stashes) {
			self.switch_to_tab(&AppTabs::Stashlist)?;
		}

		Ok(())
	}

	fn set_tab(&mut self, tab: usize) -> Result<()> {
		let tabs = self.get_tabs();
		for (i, t) in tabs.into_iter().enumerate() {
			if tab == i {
				t.show()?;
			} else {
				t.hide();
			}
		}

		self.tab = tab;
		self.options.borrow_mut().set_current_tab(tab);

		Ok(())
	}

	fn switch_to_tab(&mut self, tab: &AppTabs) -> Result<()> {
		match tab {
			AppTabs::Status => self.set_tab(0)?,
			AppTabs::Log => self.set_tab(1)?,
			AppTabs::Files => self.set_tab(2)?,
			AppTabs::Stashing => self.set_tab(3)?,
			AppTabs::Stashlist => self.set_tab(4)?,
		}
		Ok(())
	}

	fn update_commands(&mut self) {
		if self.help_popup.is_visible() {
			self.help_popup.set_cmds(self.commands(true));
		}
		self.cmdbar.borrow_mut().set_cmds(self.commands(false));
	}

	fn process_queue(&mut self, flags: NeedsUpdate) -> Result<()> {
		let mut flags = flags;
		let new_flags = self.process_internal_events();
		flags.insert(new_flags);

		if flags.contains(NeedsUpdate::ALL) {
			self.update()?;
		}
		//TODO: make this a queue event?
		//NOTE: set when any tree component changed selection
		if flags.contains(NeedsUpdate::DIFF) {
			self.status_tab.update_diff()?;
			self.inspect_commit_popup.update_diff()?;
			self.compare_commits_popup.update_diff()?;
			self.file_revlog_popup.update_diff()?;
		}
		if flags.contains(NeedsUpdate::COMMANDS) {
			self.update_commands();
		}
		if flags.contains(NeedsUpdate::BRANCHES) {
			self.select_branch_popup.update_branches()?;
		}
		if flags.contains(NeedsUpdate::REMOTES) {
			self.remotes_popup.update_remotes()?;
		}

		Ok(())
	}

	fn open_popup(
		&mut self,
		popup: StackablePopupOpen,
	) -> Result<()> {
		match popup {
			StackablePopupOpen::BlameFile(params) => {
				self.blame_file_popup.open(params)?;
			}
			StackablePopupOpen::FileRevlog(param) => {
				self.file_revlog_popup.open(param)?;
			}
			StackablePopupOpen::FileTree(param) => {
				self.revision_files_popup.open(param)?;
			}
			StackablePopupOpen::InspectCommit(param) => {
				self.inspect_commit_popup.open(param)?;
			}
			StackablePopupOpen::CompareCommits(param) => {
				self.compare_commits_popup.open(param)?;
			}
		}

		Ok(())
	}

	fn process_internal_events(&mut self) -> NeedsUpdate {
		let mut flags = NeedsUpdate::empty();

		loop {
			let front = self.queue.pop();
			if let Some(e) = front {
				let is_error_msg =
					matches!(e, InternalEvent::ShowErrorMsg(_));
				let result = self.process_internal_event(e);
				if let Err(err) = &result {
					log::error!("internal event failed: {err:?}");
					if !is_error_msg {
						self.queue.push(InternalEvent::ShowErrorMsg(
							format!("{err}"),
						));
					}
				}
				flags.insert(result.unwrap_or(NeedsUpdate::ALL));
			} else {
				break;
			}
		}
		self.queue.clear();

		flags
	}

	#[allow(clippy::too_many_lines)]
	fn process_internal_event(
		&mut self,
		ev: InternalEvent,
	) -> Result<NeedsUpdate> {
		let mut flags = NeedsUpdate::empty();
		match ev {
			InternalEvent::ConfirmedAction(action) => {
				self.process_confirmed_action(action, &mut flags)?;
			}
			InternalEvent::ConfirmAction(action) => {
				self.confirm_popup.open(action)?;
				flags.insert(NeedsUpdate::COMMANDS);
			}
			InternalEvent::ShowErrorMsg(msg) => {
				self.msg_popup.show_error(msg.as_str())?;
				flags
					.insert(NeedsUpdate::ALL | NeedsUpdate::COMMANDS);
			}
			InternalEvent::ShowInfoMsg(msg) => {
				self.msg_popup.show_info(msg.as_str())?;
				flags
					.insert(NeedsUpdate::ALL | NeedsUpdate::COMMANDS);
			}
			InternalEvent::Update(u) => flags.insert(u),
			InternalEvent::OpenCommit => self.commit_popup.show()?,
			InternalEvent::RewordCommit(id) => {
				self.commit_popup.open(Some(id))?;
			}
			InternalEvent::PopupStashing(opts) => {
				self.stashmsg_popup.options(opts);
				self.stashmsg_popup.show()?;
			}
			InternalEvent::TagCommit(id) => {
				self.tag_commit_popup.open(id)?;
			}
			InternalEvent::CreateRemote => {
				self.create_remote_popup.open()?;
			}
			InternalEvent::RenameRemote(cur_name) => {
				self.rename_remote_popup.open(cur_name)?;
			}
			InternalEvent::UpdateRemoteUrl(remote_name, cur_url) => {
				self.update_remote_url_popup
					.open(remote_name, cur_url)?;
			}
			InternalEvent::ViewRemotes => {
				self.remotes_popup.open()?;
			}
			InternalEvent::CreateBranch => {
				self.create_branch_popup.open()?;
			}
			InternalEvent::RenameBranch(branch_ref, cur_name) => {
				self.rename_branch_popup
					.open(branch_ref, cur_name)?;
			}
			InternalEvent::SelectBranch => {
				self.select_branch_popup.open()?;
			}
			InternalEvent::ViewSubmodules => {
				self.submodule_popup.open()?;
			}
			InternalEvent::Tags => {
				self.tags_popup.open()?;
			}
			InternalEvent::TabSwitchStatus => {
				self.set_tab(0)?;
				flags.insert(NeedsUpdate::ALL);
			}
			InternalEvent::TabSwitch(tab) => {
				self.switch_to_tab(&tab)?;
				flags.insert(NeedsUpdate::ALL);
			}
			InternalEvent::SelectCommitInRevlog(id) => {
				if let Err(error) = self.revlog.select_commit(id) {
					self.queue.push(InternalEvent::ShowErrorMsg(
						error.to_string(),
					));
				} else {
					self.tags_popup.hide();
					flags.insert(NeedsUpdate::ALL);
				}
			}
			InternalEvent::OpenExternalEditor(path) => {
				self.input.set_polling(false);
				self.external_editor_popup.show()?;
				self.file_to_open = path;
				flags.insert(NeedsUpdate::COMMANDS);
			}
			InternalEvent::Push(branch, push_type, force, delete) => {
				self.push_popup
					.push(branch, push_type, force, delete)?;
				flags.insert(NeedsUpdate::ALL);
			}
			InternalEvent::Pull(branch) => {
				if let Err(error) = self.pull_popup.fetch(branch) {
					self.queue.push(InternalEvent::ShowErrorMsg(
						error.to_string(),
					));
				}
				flags.insert(NeedsUpdate::ALL);
			}
			InternalEvent::FetchRemotes => {
				if let Err(error) = self.fetch_popup.fetch() {
					self.queue.push(InternalEvent::ShowErrorMsg(
						error.to_string(),
					));
				}
				flags.insert(NeedsUpdate::ALL);
			}
			InternalEvent::PushTags => {
				match sync::get_default_remote(&self.repo.borrow()) {
					Ok(remote) => {
						self.queue.push(
							InternalEvent::ConfirmAction(
								Action::PushTags(remote),
							),
						);
					}
					Err(err) => {
						self.queue.push(InternalEvent::ShowErrorMsg(
							format!("push tags failed:\n{err}"),
						));
					}
				}
			}
			InternalEvent::StatusLastFileMoved => {
				self.status_tab.last_file_moved()?;
			}
			InternalEvent::OpenFuzzyFinder(contents, target) => {
				self.fuzzy_find_popup.open(contents, target)?;
				flags
					.insert(NeedsUpdate::ALL | NeedsUpdate::COMMANDS);
			}
			InternalEvent::OpenLogSearchPopup => {
				self.log_search_popup.open()?;
				flags
					.insert(NeedsUpdate::ALL | NeedsUpdate::COMMANDS);
			}
			InternalEvent::OptionSwitched(o) => {
				match o {
					AppOption::StatusShowUntracked => {
						self.status_tab.update()?;
					}
					AppOption::DiffContextLines
					| AppOption::DiffIgnoreWhitespaces
					| AppOption::DiffInterhunkLines => {
						self.status_tab.update_diff()?;
					}
				}

				flags.insert(NeedsUpdate::ALL);
			}
			InternalEvent::FuzzyFinderChanged(
				idx,
				content,
				target,
			) => {
				match target {
					FuzzyFinderTarget::Branches => self
						.select_branch_popup
						.branch_finder_update(idx)?,
					FuzzyFinderTarget::Files => {
						// fuzzy file selection goes only to the view that opened the finder
						if self.revision_files_popup.is_visible() {
							self.revision_files_popup
								.file_finder_update(&PathBuf::from(
									content,
								));
						} else {
							self.files_tab.file_finder_update(
								&PathBuf::from(content),
							);
						}
					}
				}

				flags
					.insert(NeedsUpdate::ALL | NeedsUpdate::COMMANDS);
			}
			InternalEvent::OpenPopup(popup) => {
				self.open_popup(popup)?;
				flags
					.insert(NeedsUpdate::ALL | NeedsUpdate::COMMANDS);
			}
			InternalEvent::PopupStackPop => {
				if let Some(popup) = self.popup_stack.pop() {
					self.open_popup(popup)?;
					flags.insert(
						NeedsUpdate::ALL | NeedsUpdate::COMMANDS,
					);
				}
			}
			InternalEvent::PopupStackPush(popup) => {
				self.popup_stack.push(popup);
				flags
					.insert(NeedsUpdate::ALL | NeedsUpdate::COMMANDS);
			}
			InternalEvent::PopupStackClear => {
				self.popup_stack.clear();
				flags
					.insert(NeedsUpdate::ALL | NeedsUpdate::COMMANDS);
			}
			InternalEvent::OpenRepo { path } => {
				let submodule_repo_path = RepoPath::Path(
					Path::new(&repo_work_dir(&self.repo.borrow())?)
						.join(path),
				);
				//TODO: validate this is a valid repo first, so we can show proper error otherwise
				self.do_quit =
					QuitState::OpenSubmodule(submodule_repo_path);
			}
			InternalEvent::OpenResetPopup(id) => {
				self.reset_popup.open(id)?;
			}
			InternalEvent::CommitSearch(options) => {
				self.revlog.search(options);
			}
			InternalEvent::OpenGotoLinePopup(max_line) => {
				self.goto_line_popup.open(max_line);
			}
			InternalEvent::GotoLine(line) => {
				if self.blame_file_popup.is_visible() {
					self.blame_file_popup.goto_line(line);
				}
			}
			InternalEvent::CheckoutOption(branch) => {
				self.checkout_option_popup.open(branch)?;
			}
		}

		Ok(flags)
	}

	fn process_confirmed_action(
		&mut self,
		action: Action,
		flags: &mut NeedsUpdate,
	) -> Result<()> {
		match action {
			Action::Reset(r) => {
				self.status_tab.reset(&r);
			}
			Action::StashDrop(_) | Action::StashPop { .. } => {
				if let Err(e) = self
					.stashlist_tab
					.action_confirmed(&self.repo.borrow(), &action)
				{
					self.queue.push(InternalEvent::ShowErrorMsg(
						e.to_string(),
					));
				}
			}
			Action::ResetHunk(path, hash) => {
				sync::reset_hunk(
					&self.repo.borrow(),
					&path,
					hash,
					Some(self.options.borrow().diff_options()),
				)?;
			}
			Action::ResetLines(path, lines) => {
				sync::discard_lines(
					&self.repo.borrow(),
					&path,
					&lines,
				)?;
			}
			Action::DeleteLocalBranch(branch_ref) => {
				if let Err(e) = sync::delete_branch(
					&self.repo.borrow(),
					&branch_ref,
				) {
					self.queue.push(InternalEvent::ShowErrorMsg(
						e.to_string(),
					));
				}

				self.select_branch_popup.update_branches()?;
			}
			Action::DeleteRemoteBranch(branch_ref) => {
				self.delete_remote_branch(&branch_ref)?;
			}
			Action::DeleteRemote(remote_name) => {
				self.delete_remote(&remote_name);
			}
			Action::DeleteTag(tag_name) => {
				self.delete_tag(tag_name)?;
			}
			Action::DeleteRemoteTag(tag_name, remote) => {
				self.push_popup
					.delete_remote_tag(remote, tag_name)?;
			}
			Action::ForcePush(branch, remote, force) => {
				self.push_popup.push_with_remote(
					remote,
					branch,
					PushType::Branch,
					force,
					false,
				)?;
			}
			Action::PushTags(remote) => {
				self.push_tags_popup.push_tags(remote)?;
			}
			Action::PullMerge { rebase, .. } => {
				self.pull_popup.try_conflict_free_merge(rebase);
			}
			Action::AbortRevert | Action::AbortMerge => {
				self.status_tab.revert_pending_state();
			}
			Action::AbortRebase => {
				self.status_tab.abort_rebase();
			}
			Action::RevertCommit { id, .. } => {
				self.revert_commit(id);
			}
			Action::CheckoutCommit { id, .. } => {
				try_or_popup!(
					self,
					"failed to checkout commit:",
					sync::checkout_commit(&self.repo.borrow(), id)
				);
			}
			Action::UndoCommit { .. }
			| Action::HardReset { .. }
			| Action::CheckoutBranchDiscard(_)
			| Action::MergeBranch { .. }
			| Action::RebaseBranch { .. }
			| Action::UpdateSubmodule { .. } => {
				self.process_confirmed_repo_action(action)?;
			}
		}

		flags.insert(NeedsUpdate::ALL);

		Ok(())
	}

	fn process_confirmed_repo_action(
		&mut self,
		action: Action,
	) -> Result<()> {
		match action {
			Action::UndoCommit { .. } => {
				try_or_popup!(
					self,
					"undo commit failed:",
					undo_last_commit(&self.repo.borrow())
				);
			}
			Action::HardReset { id, .. } => {
				self.reset_popup.confirm_hard_reset(id);
			}
			Action::CheckoutBranchDiscard(branch) => {
				self.checkout_option_popup.confirm_discard(&branch);
				if self.select_branch_popup.is_visible() {
					self.select_branch_popup.update_branches()?;
				}
			}
			Action::MergeBranch {
				name, branch_type, ..
			} => {
				try_or_popup!(
					self,
					"merge branch error:",
					self.select_branch_popup
						.merge_branch(&name, branch_type)
				);
			}
			Action::RebaseBranch {
				name, branch_type, ..
			} => {
				try_or_popup!(
					self,
					"rebase error:",
					self.select_branch_popup
						.rebase_branch(&name, branch_type)
				);
			}
			Action::UpdateSubmodule { name, .. } => {
				try_or_popup!(
					self,
					"update submodule:",
					sync::update_submodule(
						&self.repo.borrow(),
						&name
					)
				);
				self.submodule_popup.update_submodules()?;
			}
			_ => {}
		}

		Ok(())
	}

	fn revert_commit(&self, id: asyncgit::sync::CommitId) {
		match sync::revert_commit(&self.repo.borrow(), id) {
			Ok(()) => {
				self.queue.push(InternalEvent::TabSwitchStatus);
			}
			Err(e) => {
				self.queue.push(InternalEvent::ShowErrorMsg(
					format!("revert error:\n{e}"),
				));
			}
		}
	}

	fn delete_tag(&mut self, tag_name: String) -> Result<()> {
		if let Err(error) =
			sync::delete_tag(&self.repo.borrow(), &tag_name)
		{
			self.queue
				.push(InternalEvent::ShowErrorMsg(error.to_string()));
		} else {
			self.tags_popup.update_tags()?;

			let repo = self.repo.borrow();
			if let Ok(remote) = sync::get_default_remote(&repo) {
				self.queue.push(InternalEvent::ConfirmAction(
					Action::DeleteRemoteTag(tag_name, remote),
				));
			}
		}
		Ok(())
	}

	fn delete_remote_branch(
		&mut self,
		branch_ref: &str,
	) -> Result<()> {
		let remotes = match sync::get_remotes(&self.repo.borrow()) {
			Ok(remotes) => remotes,
			Err(error) => {
				self.queue.push(InternalEvent::ShowErrorMsg(
					error.to_string(),
				));
				return Ok(());
			}
		};

		if let Some((remote, branch)) =
			split_remote_branch_ref(branch_ref, &remotes)
		{
			self.push_popup
				.delete_remote_branch(remote, branch.to_string())?;
		} else {
			self.queue.push(InternalEvent::ShowErrorMsg(format!(
				"Failed to find the remote and branch in {branch_ref}"
			)));
		}

		self.select_branch_popup.update_branches()?;

		Ok(())
	}

	fn delete_remote(&self, remote_name: &str) {
		let res =
			sync::delete_remote(&self.repo.borrow(), remote_name);
		match res {
			Ok(()) => {
				self.queue.push(InternalEvent::Update(
					NeedsUpdate::ALL | NeedsUpdate::REMOTES,
				));
			}
			Err(e) => {
				log::error!("delete remote: {e:?}");
				self.queue.push(InternalEvent::ShowErrorMsg(
					format!("delete remote error:\n{e}"),
				));
			}
		}
	}

	fn commands(&self, force_all: bool) -> Vec<CommandInfo> {
		let mut res = Vec::new();

		command_pump(&mut res, force_all, &self.components());

		res.push(CommandInfo::new(
			strings::commands::find_file(&self.key_config),
			!self.fuzzy_find_popup.is_visible(),
			(!self.any_popup_visible()
				&& self.files_tab.is_visible())
				|| self.revision_files_popup.is_visible()
				|| force_all,
		));

		res.push(
			CommandInfo::new(
				strings::commands::toggle_tabs(&self.key_config),
				true,
				!self.any_popup_visible(),
			)
			.order(order::NAV),
		);
		res.push(
			CommandInfo::new(
				strings::commands::toggle_tabs_direct(
					&self.key_config,
				),
				true,
				!self.any_popup_visible(),
			)
			.order(order::NAV),
		);
		res.push(
			CommandInfo::new(
				strings::commands::options_popup(&self.key_config),
				true,
				!self.any_popup_visible(),
			)
			.order(order::NAV),
		);

		res.push(
			CommandInfo::new(
				strings::commands::quit(&self.key_config),
				true,
				!self.any_popup_visible(),
			)
			.order(100),
		);

		res
	}

	//TODO: make this dynamic
	fn draw_top_bar(&self, f: &mut Frame, r: Rect) {
		const DIVIDER_PAD_SPACES: usize = 2;
		const SIDE_PADS: usize = 2;
		const MARGIN_LEFT_AND_RIGHT: usize = 2;

		let r = r.inner(Margin {
			vertical: 0,
			horizontal: 1,
		});

		let tab_labels = [
			Span::raw(strings::tab_status(&self.key_config)),
			Span::raw(strings::tab_log(&self.key_config)),
			Span::raw(strings::tab_files(&self.key_config)),
			Span::raw(strings::tab_stashing(&self.key_config)),
			Span::raw(strings::tab_stashes(&self.key_config)),
		];
		let divider = strings::tab_divider(&self.key_config);

		// heuristic, since tui doesn't provide a way to know
		// how much space is needed to draw a `Tabs`
		let tabs_len: usize =
			tab_labels.iter().map(Span::width).sum::<usize>()
				+ tab_labels.len().saturating_sub(1)
					* (divider.width() + DIVIDER_PAD_SPACES)
				+ SIDE_PADS + MARGIN_LEFT_AND_RIGHT;

		let left_right = Layout::default()
			.direction(Direction::Horizontal)
			.constraints(vec![
				Constraint::Length(
					u16::try_from(tabs_len).unwrap_or(r.width),
				),
				Constraint::Min(0),
			])
			.split(r);

		let table_area = r; // use entire area to allow drawing the horizontal separator line
		let text_area = left_right[1];

		let tabs: Vec<Line> =
			tab_labels.into_iter().map(Line::from).collect();

		f.render_widget(
			Tabs::new(tabs)
				.block(
					Block::default()
						.borders(Borders::BOTTOM)
						.border_style(self.theme.block(false)),
				)
				.style(self.theme.tab(false))
				.highlight_style(self.theme.tab(true))
				.divider(divider)
				.select(self.tab),
			table_area,
		);

		f.render_widget(
			Paragraph::new(Line::from(vec![Span::styled(
				ellipsis_trim_start(
					&self.repo_path_text,
					text_area.width as usize,
				),
				self.theme.title(false),
			)]))
			.alignment(Alignment::Right),
			text_area,
		);
	}
}

/// split `refs/remotes/<remote>/<branch>` into (remote, branch),
/// matching the longest configured remote name (remote and branch
/// names may both contain `/`)
fn split_remote_branch_ref<'a>(
	branch_ref: &'a str,
	remotes: &[String],
) -> Option<(String, &'a str)> {
	let rest = branch_ref.strip_prefix("refs/remotes/")?;
	remotes
		.iter()
		.filter(|r| {
			rest.len() > r.len() + 1
				&& rest.starts_with(r.as_str())
				&& rest.as_bytes()[r.len()] == b'/'
		})
		.max_by_key(|r| r.len())
		.map(|r| (r.clone(), &rest[r.len() + 1..]))
}

#[cfg(test)]
mod tests {
	use super::*;
	use asyncgit::sync::{CommitId, RepoState};

	#[test]
	fn test_split_remote_branch_ref() {
		let remotes = vec![
			String::from("origin"),
			String::from("upstream"),
			String::from("team/fork"),
		];
		assert_eq!(
			split_remote_branch_ref(
				"refs/remotes/upstream/feat/login",
				&remotes
			),
			Some((String::from("upstream"), "feat/login"))
		);
		assert_eq!(
			split_remote_branch_ref(
				"refs/remotes/origin/main",
				&remotes
			),
			Some((String::from("origin"), "main"))
		);
		assert_eq!(
			split_remote_branch_ref(
				"refs/remotes/team/fork/x/y",
				&remotes
			),
			Some((String::from("team/fork"), "x/y"))
		);
		assert_eq!(
			split_remote_branch_ref(
				"refs/remotes/unknown/main",
				&remotes
			),
			None
		);
		assert_eq!(
			split_remote_branch_ref("refs/heads/main", &remotes),
			None
		);
		assert_eq!(
			split_remote_branch_ref("refs/remotes/origin/", &remotes),
			None
		);
	}

	fn make_test_app(repo_path: &Path) -> (App, KeyConfig) {
		let (tx_git, _rx_git) = crossbeam_channel::unbounded();
		let (tx_app, _rx_app) = crossbeam_channel::unbounded();
		let key_config = KeyConfig::default();
		let app = App::new(
			CliArgs {
				theme: PathBuf::default(),
				select_file: None,
				repo_path: RepoPath::Path(repo_path.to_path_buf()),
				notify_watcher: false,
				key_bindings_path: None,
				key_symbols_path: None,
			},
			tx_git,
			tx_app,
			Input::new(),
			Theme::default(),
			key_config.clone(),
		)
		.unwrap();
		(app, key_config)
	}

	macro_rules! send_key {
		($app:expr, $ev:expr) => {{
			let ev = $ev;
			$app.event(InputEvent::Input(Event::Key(KeyEvent::new(
				ev.code,
				ev.modifiers,
			))))
			.unwrap();
		}};
	}

	fn commit_file(
		repo_path: &Path,
		file: &str,
		content: &str,
		msg: &str,
	) -> CommitId {
		std::fs::write(repo_path.join(file), content).unwrap();
		let rp = RepoPath::Path(repo_path.to_path_buf());
		sync::stage_add_file(&rp, Path::new(file)).unwrap();
		sync::commit(&rp, msg).unwrap()
	}

	fn create_diverged_branch(
		repo_path: &Path,
		orig_branch: &str,
		branch_name: &str,
		file: &str,
		content: &str,
	) -> CommitId {
		let rp = RepoPath::Path(repo_path.to_path_buf());
		sync::create_branch(&rp, branch_name).unwrap();
		let cid = commit_file(repo_path, file, content, branch_name);
		sync::checkout_branch(&rp, orig_branch).unwrap();
		cid
	}

	fn open_branch_popup_on(
		app: &mut App,
		kc: &KeyConfig,
		target_name: &str,
	) {
		app.select_branch_popup.open().unwrap();
		for _ in 0..10 {
			if app.select_branch_popup.selected_branch_name()
				== Some(target_name)
			{
				return;
			}
			send_key!(app, kc.keys.move_down);
		}
		panic!("branch {target_name} not found in popup");
	}

	#[test]
	fn t8d_merge_asks_first() {
		let (td, repo) = git2_testing::repo_init();
		let orig_branch =
			repo.head().unwrap().shorthand().unwrap().to_string();
		let rp = RepoPath::Path(td.path().to_path_buf());
		let base = commit_file(td.path(), "base.txt", "base", "base");
		let feat = create_diverged_branch(
			td.path(),
			&orig_branch,
			"feat",
			"feat.txt",
			"feat",
		);
		let main_head =
			commit_file(td.path(), "main.txt", "main", "main");
		assert_ne!(base, main_head);

		let (mut app, kc) = make_test_app(td.path());
		open_branch_popup_on(&mut app, &kc, "feat");

		send_key!(&mut app, kc.keys.merge_branch);
		assert!(
			app.confirm_popup.is_visible(),
			"merge must open confirm popup"
		);
		let (title, msg) = app.confirm_popup.get_text();
		assert!(title.contains("Merge"));
		assert!(msg.contains("feat"));
		assert!(msg.contains(&feat.get_short_string()));
		assert!(msg.contains(&orig_branch));
		assert!(
			app.select_branch_popup.is_visible(),
			"branch list must stay open under confirm popup"
		);
		assert_eq!(sync::repo_state(&rp).unwrap(), RepoState::Clean);

		send_key!(&mut app, kc.keys.exit_popup);
		assert!(!app.confirm_popup.is_visible());
		assert!(
			app.select_branch_popup.is_visible(),
			"Esc must leave branch list open"
		);
		assert_eq!(sync::repo_state(&rp).unwrap(), RepoState::Clean);
		assert_eq!(sync::get_head(&rp).unwrap(), main_head);

		send_key!(&mut app, kc.keys.merge_branch);
		assert!(app.confirm_popup.is_visible());
		send_key!(&mut app, kc.keys.enter);
		assert!(!app.confirm_popup.is_visible());
		assert!(!app.select_branch_popup.is_visible());
		assert_eq!(sync::repo_state(&rp).unwrap(), RepoState::Merge);
	}

	#[test]
	fn t8d_rebase_asks_first() {
		let (td, repo) = git2_testing::repo_init();
		let orig_branch =
			repo.head().unwrap().shorthand().unwrap().to_string();
		let rp = RepoPath::Path(td.path().to_path_buf());
		let _base =
			commit_file(td.path(), "base.txt", "base", "base");
		let feat = create_diverged_branch(
			td.path(),
			&orig_branch,
			"feat",
			"feat.txt",
			"feat",
		);
		let main_head =
			commit_file(td.path(), "main.txt", "main", "main");

		let (mut app, kc) = make_test_app(td.path());
		open_branch_popup_on(&mut app, &kc, "feat");

		send_key!(&mut app, kc.keys.rebase_branch);
		assert!(
			app.confirm_popup.is_visible(),
			"rebase must open confirm popup"
		);
		let (title, msg) = app.confirm_popup.get_text();
		assert!(title.contains("Rebase"));
		assert!(msg.contains("feat"));
		assert!(msg.contains(&feat.get_short_string()));
		assert!(msg.contains(&orig_branch));
		assert!(
			app.select_branch_popup.is_visible(),
			"branch list must stay open under confirm popup"
		);
		assert_eq!(sync::get_head(&rp).unwrap(), main_head);

		send_key!(&mut app, kc.keys.exit_popup);
		assert!(!app.confirm_popup.is_visible());
		assert!(
			app.select_branch_popup.is_visible(),
			"Esc must leave branch list open"
		);
		assert_eq!(sync::get_head(&rp).unwrap(), main_head);

		send_key!(&mut app, kc.keys.rebase_branch);
		assert!(app.confirm_popup.is_visible());
		send_key!(&mut app, kc.keys.enter);
		assert!(!app.confirm_popup.is_visible());
		assert!(!app.select_branch_popup.is_visible());
		assert_ne!(sync::get_head(&rp).unwrap(), main_head);
	}

	#[test]
	fn t8d_prompt_pins_branch() {
		let (td, repo) = git2_testing::repo_init();
		let orig_branch =
			repo.head().unwrap().shorthand().unwrap().to_string();
		let rp = RepoPath::Path(td.path().to_path_buf());
		let _base =
			commit_file(td.path(), "base.txt", "base", "base");
		let _beta = create_diverged_branch(
			td.path(),
			&orig_branch,
			"beta",
			"beta.txt",
			"beta",
		);
		let _main_head =
			commit_file(td.path(), "main.txt", "main", "main");

		let (mut app, kc) = make_test_app(td.path());
		open_branch_popup_on(&mut app, &kc, "beta");

		send_key!(&mut app, kc.keys.merge_branch);
		assert!(
			app.confirm_popup.is_visible(),
			"merge must open confirm popup"
		);

		// Add a branch that sorts before `beta` and refresh the list
		// while the confirmation prompt is open.
		let _alpha = create_diverged_branch(
			td.path(),
			&orig_branch,
			"alpha",
			"alpha.txt",
			"alpha",
		);
		app.select_branch_popup.update_branches().unwrap();

		send_key!(&mut app, kc.keys.enter);
		assert!(!app.confirm_popup.is_visible());
		assert_eq!(sync::repo_state(&rp).unwrap(), RepoState::Merge);
		assert!(
			td.path().join("beta.txt").exists(),
			"pinned branch `beta` must be merged"
		);
		assert!(
			!td.path().join("alpha.txt").exists(),
			"newly sorted branch `alpha` must not be merged"
		);
	}

	#[test]
	fn t8d_moved_branch_refuses() {
		let (td, repo) = git2_testing::repo_init();
		let orig_branch =
			repo.head().unwrap().shorthand().unwrap().to_string();
		let rp = RepoPath::Path(td.path().to_path_buf());
		let _base =
			commit_file(td.path(), "base.txt", "base", "base");
		let _feat = create_diverged_branch(
			td.path(),
			&orig_branch,
			"feat",
			"feat.txt",
			"feat",
		);
		let main_head =
			commit_file(td.path(), "main.txt", "main", "main");

		let (mut app, kc) = make_test_app(td.path());
		open_branch_popup_on(&mut app, &kc, "feat");

		// 1. Target branch moves while prompt is open.
		send_key!(&mut app, kc.keys.merge_branch);
		assert!(
			app.confirm_popup.is_visible(),
			"merge must open confirm popup"
		);
		let new_feat = create_diverged_branch(
			td.path(),
			&orig_branch,
			"feat-extra",
			"extra.txt",
			"extra",
		);
		repo.reference(
			"refs/heads/feat",
			new_feat.into(),
			true,
			"move feat",
		)
		.unwrap();

		send_key!(&mut app, kc.keys.enter);
		assert!(!app.confirm_popup.is_visible());
		assert!(
			app.msg_popup.is_visible(),
			"moved branch must show repo-changed error"
		);
		assert!(app.select_branch_popup.is_visible());
		assert_eq!(sync::repo_state(&rp).unwrap(), RepoState::Clean);
		send_key!(&mut app, kc.keys.exit_popup);

		// 2. Target branch deleted while prompt is open.
		app.select_branch_popup.update_branches().unwrap();
		open_branch_popup_on(&mut app, &kc, "feat");
		send_key!(&mut app, kc.keys.merge_branch);
		assert!(app.confirm_popup.is_visible());
		sync::delete_branch(&rp, "refs/heads/feat").unwrap();

		send_key!(&mut app, kc.keys.enter);
		assert!(
			app.msg_popup.is_visible(),
			"deleted branch must show repo-changed error"
		);
		assert_eq!(sync::repo_state(&rp).unwrap(), RepoState::Clean);
		assert_eq!(sync::get_head(&rp).unwrap(), main_head);
		send_key!(&mut app, kc.keys.exit_popup);

		// 3. HEAD advances while prompt is open.
		app.select_branch_popup.update_branches().unwrap();
		open_branch_popup_on(&mut app, &kc, "feat-extra");
		send_key!(&mut app, kc.keys.merge_branch);
		assert!(app.confirm_popup.is_visible());
		let advanced_head =
			commit_file(td.path(), "adv.txt", "adv", "advance");
		send_key!(&mut app, kc.keys.enter);
		assert!(
			app.msg_popup.is_visible(),
			"HEAD change must show repo-changed error"
		);
		assert_eq!(sync::repo_state(&rp).unwrap(), RepoState::Clean);
		assert_eq!(sync::get_head(&rp).unwrap(), advanced_head);
	}

	#[test]
	fn t8d_invalid_branch_no_prompt() {
		let (td, repo) = git2_testing::repo_init();
		let _base =
			commit_file(td.path(), "base.txt", "base", "base");
		let cur_branch =
			repo.head().unwrap().shorthand().unwrap().to_string();

		let (mut app, kc) = make_test_app(td.path());
		open_branch_popup_on(&mut app, &kc, &cur_branch);

		// Current branch selected: no prompt.
		send_key!(&mut app, kc.keys.merge_branch);
		assert!(!app.confirm_popup.is_visible());
		send_key!(&mut app, kc.keys.rebase_branch);
		assert!(!app.confirm_popup.is_visible());

		// Switch to empty remote list: no prompt.
		send_key!(&mut app, kc.keys.tab_toggle);
		send_key!(&mut app, kc.keys.merge_branch);
		assert!(!app.confirm_popup.is_visible());
		send_key!(&mut app, kc.keys.rebase_branch);
		assert!(!app.confirm_popup.is_visible());
	}

	fn set_pending_state(git_dir: &Path, kind: &str, oid: &str) {
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
	fn t8d_pending_state_blocks() {
		let (td, repo) = git2_testing::repo_init();
		let orig_branch =
			repo.head().unwrap().shorthand().unwrap().to_string();
		let _base =
			commit_file(td.path(), "base.txt", "base", "base");
		let _feat = create_diverged_branch(
			td.path(),
			&orig_branch,
			"feat",
			"feat.txt",
			"feat",
		);
		let main_head =
			commit_file(td.path(), "main.txt", "main", "main");
		let oid_str = main_head.to_string();
		let git_dir = repo.path().to_path_buf();

		let (mut app, kc) = make_test_app(td.path());
		open_branch_popup_on(&mut app, &kc, "feat");

		let merge_hint =
			strings::commands::merge_branch_popup(&app.key_config)
				.name;
		let rebase_hint =
			strings::commands::branch_popup_rebase(&app.key_config)
				.name;

		for kind in ["merge", "revert", "rebase", "cherry-pick"] {
			set_pending_state(&git_dir, kind, &oid_str);

			let mut cmds = Vec::new();
			app.select_branch_popup.commands(&mut cmds, false);
			assert!(
				!cmds
					.iter()
					.find(|c| c.text.name == merge_hint)
					.is_some_and(|c| c.enabled),
				"merge hint must be disabled during {kind}"
			);
			assert!(
				!cmds
					.iter()
					.find(|c| c.text.name == rebase_hint)
					.is_some_and(|c| c.enabled),
				"rebase hint must be disabled during {kind}"
			);

			send_key!(&mut app, kc.keys.merge_branch);
			assert!(!app.confirm_popup.is_visible());
			assert!(!app.msg_popup.is_visible());

			send_key!(&mut app, kc.keys.rebase_branch);
			assert!(!app.confirm_popup.is_visible());
			assert!(!app.msg_popup.is_visible());
		}
	}

	fn init_repo_with_submodule(
	) -> (tempfile::TempDir, tempfile::TempDir, CommitId, CommitId) {
		let (sub_td, _sub_repo) = git2_testing::repo_init();
		let sub_c1 =
			commit_file(sub_td.path(), "sub.txt", "v1", "sub v1");
		let sub_c2 =
			commit_file(sub_td.path(), "sub.txt", "v2", "sub v2");

		let (parent_td, parent_repo) = git2_testing::repo_init();
		let sub_url = sub_td.path().to_str().unwrap();
		{
			let mut s = parent_repo
				.submodule(sub_url, Path::new("sub"), false)
				.unwrap();
			let sub_clone = s.clone(None).unwrap();
			{
				let mut cfg = sub_clone.config().unwrap();
				cfg.set_str("user.name", "testuser").unwrap();
				cfg.set_str("user.email", "testuser@example.com")
					.unwrap();
			}
			let sub_rp = RepoPath::Path(parent_td.path().join("sub"));
			sync::checkout_commit(&sub_rp, sub_c1).unwrap();
			s.add_finalize().unwrap();
		}
		commit_file(
			parent_td.path(),
			"README",
			"hi",
			"add submodule",
		);

		// Move submodule workdir to sub_c2 so update has work to do
		// (bringing workdir back to recorded sub_c1).
		let sub_rp = RepoPath::Path(parent_td.path().join("sub"));
		sync::checkout_commit(&sub_rp, sub_c2).unwrap();

		(parent_td, sub_td, sub_c1, sub_c2)
	}

	#[test]
	fn t8d_submodule_asks_first() {
		let (parent_td, _sub_td, sub_c1, sub_c2) =
			init_repo_with_submodule();
		let sub_rp = RepoPath::Path(parent_td.path().join("sub"));
		let (mut app, kc) = make_test_app(parent_td.path());
		app.submodule_popup.open().unwrap();

		send_key!(&mut app, kc.keys.update_submodule);
		assert!(
			app.confirm_popup.is_visible(),
			"submodule update must open confirm popup"
		);
		let (title, msg) = app.confirm_popup.get_text();
		assert!(title.contains("Submodule"));
		assert!(msg.contains("sub"));
		assert!(msg.contains(&sub_c1.get_short_string()));
		assert_eq!(
			sync::get_head(&sub_rp).unwrap(),
			sub_c2,
			"submodule must not update before confirmation"
		);

		send_key!(&mut app, kc.keys.exit_popup);
		assert!(!app.confirm_popup.is_visible());
		assert_eq!(
			sync::get_head(&sub_rp).unwrap(),
			sub_c2,
			"Esc must leave submodule unchanged"
		);

		send_key!(&mut app, kc.keys.update_submodule);
		assert!(app.confirm_popup.is_visible());
		send_key!(&mut app, kc.keys.enter);
		assert!(!app.confirm_popup.is_visible());
		assert_eq!(
			sync::get_head(&sub_rp).unwrap(),
			sub_c1,
			"confirmed update must check out recorded commit"
		);
	}

	#[test]
	fn t8d_moved_submodule_refuses() {
		let (parent_td, _sub_td, sub_c1, _sub_c2) =
			init_repo_with_submodule();
		let sub_wd = parent_td.path().join("sub");
		let sub_rp = RepoPath::Path(sub_wd.clone());
		let (mut app, kc) = make_test_app(parent_td.path());
		app.submodule_popup.open().unwrap();

		send_key!(&mut app, kc.keys.update_submodule);
		assert!(
			app.confirm_popup.is_visible(),
			"submodule update must open confirm popup"
		);
		let (title, msg) = app.confirm_popup.get_text();
		assert!(title.contains("Submodule"));
		assert!(msg.contains("sub"));
		assert!(msg.contains(&sub_c1.get_short_string()));

		// Externally change the submodule's checked-out commit while prompt is open.
		let external =
			commit_file(&sub_wd, "ext.txt", "ext", "external");

		send_key!(&mut app, kc.keys.enter);
		assert!(!app.confirm_popup.is_visible());
		assert!(
			app.msg_popup.is_visible(),
			"moved submodule must show repo-changed error"
		);
		assert_eq!(
			sync::get_head(&sub_rp).unwrap(),
			external,
			"refused update must not change submodule checkout"
		);
		assert_ne!(external, sub_c1);
	}

	#[test]
	fn t8e_hard_reset_asks_first() {
		let (td, _repo) = git2_testing::repo_init();
		let rp = RepoPath::Path(td.path().to_path_buf());
		let c1 = commit_file(td.path(), "a.txt", "v1", "first");
		let c2 = commit_file(td.path(), "a.txt", "v2", "second");
		std::fs::write(td.path().join("a.txt"), "dirty").unwrap();

		let (mut app, kc) = make_test_app(td.path());
		app.reset_popup.open(c1).unwrap();
		send_key!(&mut app, kc.keys.move_down);
		send_key!(&mut app, kc.keys.move_down);
		assert_eq!(app.reset_popup.kind(), sync::ResetType::Hard);

		send_key!(&mut app, kc.keys.enter);
		assert!(
			app.confirm_popup.is_visible(),
			"hard reset must open confirm popup"
		);
		assert!(
			app.reset_popup.is_visible(),
			"reset picker must stay open under confirm popup"
		);
		let (_title, msg) = app.confirm_popup.get_text();
		assert!(
			msg.contains(&c1.get_short_string()),
			"confirm msg must name target commit: {msg}"
		);
		assert!(
			msg.to_lowercase().contains("uncommitted")
				&& msg.to_lowercase().contains("lost"),
			"confirm msg must warn uncommitted changes will be lost: {msg}"
		);
		assert_eq!(
			sync::get_head(&rp).unwrap(),
			c2,
			"HEAD must not change before confirmation"
		);
		assert_eq!(
			std::fs::read_to_string(td.path().join("a.txt")).unwrap(),
			"dirty"
		);

		send_key!(&mut app, kc.keys.enter);
		assert!(!app.confirm_popup.is_visible());
		assert!(!app.reset_popup.is_visible());
		assert_eq!(sync::get_head(&rp).unwrap(), c1);
		assert_eq!(
			std::fs::read_to_string(td.path().join("a.txt")).unwrap(),
			"v1"
		);
	}

	#[test]
	fn t8e_discard_asks_first() {
		let (td, repo) = git2_testing::repo_init();
		let orig_branch =
			repo.head().unwrap().shorthand().unwrap().to_string();
		let rp = RepoPath::Path(td.path().to_path_buf());
		let _base = commit_file(td.path(), "a.txt", "base", "base");
		let other_id = create_diverged_branch(
			td.path(),
			&orig_branch,
			"other",
			"a.txt",
			"other_val",
		);
		std::fs::write(td.path().join("a.txt"), "dirty").unwrap();

		let target = sync::get_branches_info(&rp, true)
			.unwrap()
			.into_iter()
			.find(|b| b.name == "other")
			.unwrap();

		let (mut app, kc) = make_test_app(td.path());
		app.checkout_option_popup.open(target).unwrap();
		send_key!(&mut app, kc.keys.move_down);
		assert!(
			app.checkout_option_popup.option()
				== strings::CheckoutOptions::DiscardAllLocalChagnes
		);

		send_key!(&mut app, kc.keys.enter);
		assert!(
			app.confirm_popup.is_visible(),
			"discard checkout must open confirm popup"
		);
		assert!(
			app.checkout_option_popup.is_visible(),
			"checkout picker must stay open under confirm popup"
		);
		let (_title, msg) = app.confirm_popup.get_text();
		assert!(
			msg.contains("other"),
			"confirm msg must name target branch: {msg}"
		);
		assert!(
			msg.to_lowercase().contains("uncommitted")
				&& msg.to_lowercase().contains("lost"),
			"confirm msg must warn uncommitted changes will be lost: {msg}"
		);
		assert_ne!(
			sync::get_head(&rp).unwrap(),
			other_id,
			"HEAD must not switch before confirmation"
		);
		assert_eq!(
			std::fs::read_to_string(td.path().join("a.txt")).unwrap(),
			"dirty"
		);

		send_key!(&mut app, kc.keys.enter);
		assert!(!app.confirm_popup.is_visible());
		assert!(!app.checkout_option_popup.is_visible());
		assert_eq!(sync::get_head(&rp).unwrap(), other_id);
		assert_eq!(
			std::fs::read_to_string(td.path().join("a.txt")).unwrap(),
			"other_val"
		);
	}

	#[test]
	fn t8e_cancel_returns_to_picker() {
		let (td, repo) = git2_testing::repo_init();
		let orig_branch =
			repo.head().unwrap().shorthand().unwrap().to_string();
		let rp = RepoPath::Path(td.path().to_path_buf());
		let c1 = commit_file(td.path(), "a.txt", "v1", "first");
		let _other = create_diverged_branch(
			td.path(),
			&orig_branch,
			"other",
			"b.txt",
			"other",
		);
		let c2 = commit_file(td.path(), "a.txt", "v2", "second");
		std::fs::write(td.path().join("a.txt"), "dirty").unwrap();

		let (mut app, kc) = make_test_app(td.path());

		// 1. ResetPopup on Hard -> Enter -> Esc returns to picker with Hard selected
		app.reset_popup.open(c1).unwrap();
		send_key!(&mut app, kc.keys.move_down);
		send_key!(&mut app, kc.keys.move_down);
		send_key!(&mut app, kc.keys.enter);
		assert!(app.confirm_popup.is_visible());

		send_key!(&mut app, kc.keys.exit_popup);
		assert!(!app.confirm_popup.is_visible());
		assert!(app.reset_popup.is_visible());
		assert_eq!(app.reset_popup.kind(), sync::ResetType::Hard);
		assert_eq!(sync::get_head(&rp).unwrap(), c2);
		assert_eq!(
			std::fs::read_to_string(td.path().join("a.txt")).unwrap(),
			"dirty"
		);
		send_key!(&mut app, kc.keys.exit_popup);
		assert!(!app.reset_popup.is_visible());

		// 2. CheckoutOptionPopup on Discard -> Enter -> Esc returns to picker with Discard selected
		let target = sync::get_branches_info(&rp, true)
			.unwrap()
			.into_iter()
			.find(|b| b.name == "other")
			.unwrap();
		app.checkout_option_popup.open(target).unwrap();
		send_key!(&mut app, kc.keys.move_down);
		send_key!(&mut app, kc.keys.enter);
		assert!(app.confirm_popup.is_visible());

		send_key!(&mut app, kc.keys.exit_popup);
		assert!(!app.confirm_popup.is_visible());
		assert!(app.checkout_option_popup.is_visible());
		assert!(
			app.checkout_option_popup.option()
				== strings::CheckoutOptions::DiscardAllLocalChagnes
		);
		assert_eq!(sync::get_head(&rp).unwrap(), c2);
		assert_eq!(
			std::fs::read_to_string(td.path().join("a.txt")).unwrap(),
			"dirty"
		);
	}

	#[test]
	fn t8e_stale_pick_refuses() {
		let (td, repo) = git2_testing::repo_init();
		let orig_branch =
			repo.head().unwrap().shorthand().unwrap().to_string();
		let rp = RepoPath::Path(td.path().to_path_buf());
		let c1 = commit_file(td.path(), "a.txt", "v1", "first");
		let _other1 = create_diverged_branch(
			td.path(),
			&orig_branch,
			"other",
			"other.txt",
			"v1",
		);
		let _c2 = commit_file(td.path(), "a.txt", "v2", "second");

		let (mut app, kc) = make_test_app(td.path());

		// 1. Hard reset: HEAD changes while confirm is open -> refuse, picker stays open, next Enter works
		app.reset_popup.open(c1).unwrap();
		send_key!(&mut app, kc.keys.move_down);
		send_key!(&mut app, kc.keys.move_down);
		send_key!(&mut app, kc.keys.enter);
		assert!(app.confirm_popup.is_visible());

		let c3 = commit_file(td.path(), "a.txt", "v3", "third");
		send_key!(&mut app, kc.keys.enter);
		assert!(!app.confirm_popup.is_visible());
		assert!(app.msg_popup.is_visible());
		assert!(app.reset_popup.is_visible());
		assert_eq!(sync::get_head(&rp).unwrap(), c3);

		send_key!(&mut app, kc.keys.exit_popup); // dismiss error
		assert!(!app.msg_popup.is_visible());
		assert!(app.reset_popup.is_visible());

		send_key!(&mut app, kc.keys.enter); // re-confirm against current HEAD
		assert!(app.confirm_popup.is_visible());
		send_key!(&mut app, kc.keys.enter);
		assert!(!app.confirm_popup.is_visible());
		assert!(!app.reset_popup.is_visible());
		assert_eq!(sync::get_head(&rp).unwrap(), c1);

		// 2. Discard checkout: target branch moves while confirm is open -> refuse, picker stays open, next Enter confirms against moved branch
		std::fs::write(td.path().join("a.txt"), "dirty").unwrap();
		let target = sync::get_branches_info(&rp, true)
			.unwrap()
			.into_iter()
			.find(|b| b.name == "other")
			.unwrap();
		app.checkout_option_popup.open(target).unwrap();
		send_key!(&mut app, kc.keys.move_down);
		send_key!(&mut app, kc.keys.enter);
		assert!(app.confirm_popup.is_visible());

		let other2 = create_diverged_branch(
			td.path(),
			&orig_branch,
			"other-new",
			"other2.txt",
			"v2",
		);
		repo.reference(
			"refs/heads/other",
			other2.into(),
			true,
			"move other",
		)
		.unwrap();

		send_key!(&mut app, kc.keys.enter);
		assert!(!app.confirm_popup.is_visible());
		assert!(app.msg_popup.is_visible());
		assert!(app.checkout_option_popup.is_visible());
		assert_eq!(sync::get_head(&rp).unwrap(), c1);

		send_key!(&mut app, kc.keys.exit_popup); // dismiss error
		assert!(!app.msg_popup.is_visible());
		assert!(app.checkout_option_popup.is_visible());

		// Next Enter confirms against `other` as it is now (`other2`)
		send_key!(&mut app, kc.keys.enter);
		assert!(app.confirm_popup.is_visible());
		let (_title, msg) = app.confirm_popup.get_text();
		assert!(
			msg.contains(&other2.get_short_string()),
			"re-opened confirm must show updated target commit: {msg}"
		);
		send_key!(&mut app, kc.keys.enter);
		assert!(!app.confirm_popup.is_visible());
		assert!(!app.checkout_option_popup.is_visible());
		assert_eq!(sync::get_head(&rp).unwrap(), other2);
	}

	#[test]
	fn t8e_pickers_do_not_wrap() {
		let (td, repo) = git2_testing::repo_init();
		let orig_branch =
			repo.head().unwrap().shorthand().unwrap().to_string();
		let rp = RepoPath::Path(td.path().to_path_buf());
		let c1 = commit_file(td.path(), "a.txt", "v1", "first");
		let _other = create_diverged_branch(
			td.path(),
			&orig_branch,
			"other",
			"b.txt",
			"other",
		);

		let (mut app, kc) = make_test_app(td.path());

		// ResetPopup: Soft -> Up stays Soft; Down -> Mixed -> Hard -> Down stays Hard
		app.reset_popup.open(c1).unwrap();
		assert_eq!(app.reset_popup.kind(), sync::ResetType::Soft);
		send_key!(&mut app, kc.keys.move_up);
		assert_eq!(app.reset_popup.kind(), sync::ResetType::Soft);
		send_key!(&mut app, kc.keys.move_down);
		assert_eq!(app.reset_popup.kind(), sync::ResetType::Mixed);
		send_key!(&mut app, kc.keys.move_down);
		assert_eq!(app.reset_popup.kind(), sync::ResetType::Hard);
		send_key!(&mut app, kc.keys.move_down);
		assert_eq!(app.reset_popup.kind(), sync::ResetType::Hard);
		send_key!(&mut app, kc.keys.move_up);
		assert_eq!(app.reset_popup.kind(), sync::ResetType::Mixed);
		send_key!(&mut app, kc.keys.move_up);
		assert_eq!(app.reset_popup.kind(), sync::ResetType::Soft);
		send_key!(&mut app, kc.keys.move_up);
		assert_eq!(app.reset_popup.kind(), sync::ResetType::Soft);
		send_key!(&mut app, kc.keys.exit_popup);

		// CheckoutOptionPopup: KeepLocalChanges -> Up stays Keep; Down -> Discard -> Down stays Discard
		let target = sync::get_branches_info(&rp, true)
			.unwrap()
			.into_iter()
			.find(|b| b.name == "other")
			.unwrap();
		app.checkout_option_popup.open(target).unwrap();
		assert!(
			app.checkout_option_popup.option()
				== strings::CheckoutOptions::KeepLocalChanges
		);
		send_key!(&mut app, kc.keys.move_up);
		assert!(
			app.checkout_option_popup.option()
				== strings::CheckoutOptions::KeepLocalChanges
		);
		send_key!(&mut app, kc.keys.move_down);
		assert!(
			app.checkout_option_popup.option()
				== strings::CheckoutOptions::DiscardAllLocalChagnes
		);
		send_key!(&mut app, kc.keys.move_down);
		assert!(
			app.checkout_option_popup.option()
				== strings::CheckoutOptions::DiscardAllLocalChagnes
		);
		send_key!(&mut app, kc.keys.move_up);
		assert!(
			app.checkout_option_popup.option()
				== strings::CheckoutOptions::KeepLocalChanges
		);
	}

	#[test]
	fn t8e_reset_requests_refresh() {
		let (td, _repo) = git2_testing::repo_init();
		let rp = RepoPath::Path(td.path().to_path_buf());
		let c1 = commit_file(td.path(), "a.txt", "v1", "first");
		let _c2 = commit_file(td.path(), "a.txt", "v2", "second");

		let mut env = Environment::test_env();
		env.repo = RefCell::new(rp.clone());
		let enter_ev = Event::Key(KeyEvent::new(
			env.key_config.keys.enter.code,
			env.key_config.keys.enter.modifiers,
		));
		let down_ev = Event::Key(KeyEvent::new(
			env.key_config.keys.move_down.code,
			env.key_config.keys.move_down.modifiers,
		));

		let mut popup = ResetPopup::new(&env);

		// Soft reset requests NeedsUpdate::ALL
		popup.open(c1).unwrap();
		popup.event(&enter_ev).unwrap();
		assert!(!popup.is_visible());
		assert_eq!(sync::get_head(&rp).unwrap(), c1);
		let mut saw_update = false;
		while let Some(ev) = env.queue.pop() {
			if matches!(
				ev,
				InternalEvent::Update(u) if u.contains(NeedsUpdate::ALL)
			) {
				saw_update = true;
			}
		}
		assert!(
			saw_update,
			"soft reset must request NeedsUpdate::ALL"
		);

		// Mixed reset requests NeedsUpdate::ALL
		let _c3 = commit_file(td.path(), "a.txt", "v3", "third");
		popup.open(c1).unwrap();
		popup.event(&down_ev).unwrap();
		assert_eq!(popup.kind(), sync::ResetType::Mixed);
		popup.event(&enter_ev).unwrap();
		assert!(!popup.is_visible());
		assert_eq!(sync::get_head(&rp).unwrap(), c1);
		let mut saw_update = false;
		while let Some(ev) = env.queue.pop() {
			if matches!(
				ev,
				InternalEvent::Update(u) if u.contains(NeedsUpdate::ALL)
			) {
				saw_update = true;
			}
		}
		assert!(
			saw_update,
			"mixed reset must request NeedsUpdate::ALL"
		);
	}

	#[test]
	fn t8e_undo_blocked_when_pending() {
		let (td, repo) = git2_testing::repo_init();
		let rp = RepoPath::Path(td.path().to_path_buf());
		let _c1 = commit_file(td.path(), "a.txt", "v1", "first");
		let c2 = commit_file(td.path(), "a.txt", "v2", "second");
		let oid_str = c2.to_string();
		let git_dir = repo.path().to_path_buf();

		let (mut app, kc) = make_test_app(td.path());
		let undo_hint =
			strings::commands::undo_commit(&app.key_config).name;

		for kind in ["merge", "revert", "rebase", "cherry-pick"] {
			set_pending_state(&git_dir, kind, &oid_str);

			let mut cmds = Vec::new();
			app.status_tab.commands(&mut cmds, false);
			assert!(
				!cmds
					.iter()
					.find(|c| c.text.name == undo_hint)
					.is_some_and(|c| c.enabled),
				"undo hint must be disabled during {kind}"
			);

			send_key!(&mut app, kc.keys.undo_commit);
			assert!(
				!app.confirm_popup.is_visible(),
				"undo key must not open confirm during {kind}"
			);
			assert!(!app.msg_popup.is_visible());
			assert_eq!(sync::get_head(&rp).unwrap(), c2);
		}
	}

	#[test]
	fn t8e_stale_undo_refuses() {
		let (td, repo) = git2_testing::repo_init();
		let rp = RepoPath::Path(td.path().to_path_buf());
		let _c1 = commit_file(td.path(), "a.txt", "v1", "first");
		let c2 =
			commit_file(td.path(), "a.txt", "v2", "second commit");
		let git_dir = repo.path().to_path_buf();

		let (mut app, kc) = make_test_app(td.path());

		// 1. HEAD changes while undo prompt is open
		send_key!(&mut app, kc.keys.undo_commit);
		assert!(app.confirm_popup.is_visible());
		let (_title, msg) = app.confirm_popup.get_text();
		assert!(
			msg.contains(&c2.get_short_string()),
			"undo confirm must name short id: {msg}"
		);
		assert!(
			msg.contains("second commit"),
			"undo confirm must name subject: {msg}"
		);
		assert!(
			msg.to_lowercase().contains("staged"),
			"undo confirm must say changes stay staged: {msg}"
		);

		let c3 = commit_file(td.path(), "a.txt", "v3", "third");
		send_key!(&mut app, kc.keys.enter);
		assert!(!app.confirm_popup.is_visible());
		assert!(
			app.msg_popup.is_visible(),
			"HEAD change must show repo-changed error"
		);
		assert_eq!(sync::get_head(&rp).unwrap(), c3);
		send_key!(&mut app, kc.keys.exit_popup);

		// 2. Repo state changes while undo prompt is open
		send_key!(&mut app, kc.keys.undo_commit);
		assert!(app.confirm_popup.is_visible());
		set_pending_state(&git_dir, "merge", &c3.to_string());
		send_key!(&mut app, kc.keys.enter);
		assert!(!app.confirm_popup.is_visible());
		assert!(
			app.msg_popup.is_visible(),
			"state change must show repo-changed error"
		);
		assert_eq!(sync::get_head(&rp).unwrap(), c3);
	}

	#[test]
	fn t8e_undo_without_parent() {
		// 1. Initial commit (no parent)
		{
			let (td, _repo) = git2_testing::repo_init();
			let (mut app, kc) = make_test_app(td.path());

			send_key!(&mut app, kc.keys.undo_commit);
			assert!(
				!app.confirm_popup.is_visible(),
				"undo without parent must not open confirm popup"
			);
			assert!(
				app.msg_popup.is_visible(),
				"undo without parent must show error"
			);
		}

		// 2. Unborn HEAD (empty repo)
		{
			let td = tempfile::TempDir::new().unwrap();
			let status = std::process::Command::new("git")
				.arg("init")
				.current_dir(td.path())
				.status()
				.unwrap();
			assert!(status.success());
			let (mut app, kc) = make_test_app(td.path());

			send_key!(&mut app, kc.keys.undo_commit);
			assert!(
				!app.confirm_popup.is_visible(),
				"undo on unborn HEAD must not open confirm popup"
			);
			assert!(
				app.msg_popup.is_visible(),
				"undo on unborn HEAD must show error"
			);
		}
	}

	fn make_test_app_with_rx(
		repo_path: &Path,
	) -> (
		App,
		KeyConfig,
		crossbeam_channel::Receiver<AsyncGitNotification>,
	) {
		let (tx_git, rx_git) = crossbeam_channel::unbounded();
		let (tx_app, _rx_app) = crossbeam_channel::unbounded();
		let key_config = KeyConfig::default();
		let app = App::new(
			CliArgs {
				theme: PathBuf::default(),
				select_file: None,
				repo_path: RepoPath::Path(repo_path.to_path_buf()),
				notify_watcher: false,
				key_bindings_path: None,
				key_symbols_path: None,
			},
			tx_git,
			tx_app,
			Input::new(),
			Theme::default(),
			key_config.clone(),
		)
		.unwrap();
		(app, key_config, rx_git)
	}

	fn drain_app_work(
		app: &mut App,
		rx_git: &crossbeam_channel::Receiver<AsyncGitNotification>,
	) {
		let deadline = std::time::Instant::now()
			+ std::time::Duration::from_secs(5);
		while app.any_work_pending()
			&& std::time::Instant::now() < deadline
		{
			if let Ok(ev) = rx_git
				.recv_timeout(std::time::Duration::from_millis(50))
			{
				let _ = app.update_async(AsyncNotification::Git(ev));
			}
		}
	}

	#[test]
	fn t8f_push_tags_asks_first() {
		let (bare_td, bare_repo) = git2_testing::repo_init_bare();
		let (td, repo) = git2_testing::repo_init();
		let head_oid = repo.head().unwrap().target().unwrap();
		let head_obj = repo.find_object(head_oid, None).unwrap();
		repo.tag_lightweight("v1.0", &head_obj, false).unwrap();
		repo.remote("origin", bare_td.path().to_str().unwrap())
			.unwrap();
		repo.reference(
			"refs/remotes/origin/main",
			head_oid,
			true,
			"test",
		)
		.unwrap();

		let (mut app, kc, rx_git) = make_test_app_with_rx(td.path());

		// 1. From Log tab: pressing push opens confirm naming the remote; Esc pushes nothing.
		app.set_tab(1).unwrap();
		send_key!(&mut app, kc.keys.push);
		assert!(
			app.confirm_popup.is_visible(),
			"push tags from Log must open confirm popup"
		);
		let (_, msg) = app.confirm_popup.get_text();
		assert!(
			msg.contains("origin"),
			"push tags confirm must name remote: {msg}"
		);
		assert!(
			bare_repo.tag_names(None).unwrap().is_empty(),
			"nothing may be pushed before confirmation"
		);

		send_key!(&mut app, kc.keys.exit_popup);
		assert!(!app.confirm_popup.is_visible());
		assert!(!app.push_tags_popup.is_visible());
		assert!(
			bare_repo.tag_names(None).unwrap().is_empty(),
			"Esc must not push tags"
		);

		// 2. From TagList popup: pressing push opens confirm naming the remote; Esc pushes nothing.
		app.tags_popup.open().unwrap();
		send_key!(&mut app, kc.keys.push);
		assert!(
			app.confirm_popup.is_visible(),
			"push tags from tag list must open confirm popup"
		);
		let (_, msg) = app.confirm_popup.get_text();
		assert!(
			msg.contains("origin"),
			"push tags confirm from tag list must name remote: {msg}"
		);
		assert!(bare_repo.tag_names(None).unwrap().is_empty());

		send_key!(&mut app, kc.keys.exit_popup);
		assert!(!app.confirm_popup.is_visible());
		assert!(app.tags_popup.is_visible());
		assert!(!app.push_tags_popup.is_visible());
		assert!(bare_repo.tag_names(None).unwrap().is_empty());
		drain_app_work(&mut app, &rx_git);
	}

	#[test]
	fn t8f_push_tags_without_remote() {
		// 1. No remotes at all.
		{
			let (td, _repo) = git2_testing::repo_init();
			let (mut app, kc, rx_git) =
				make_test_app_with_rx(td.path());
			app.set_tab(1).unwrap();

			send_key!(&mut app, kc.keys.push);
			assert!(
				!app.confirm_popup.is_visible(),
				"no remote must not open confirm popup"
			);
			assert!(
				!app.push_tags_popup.is_visible(),
				"no remote must not open push tags popup"
			);
			assert!(
				app.msg_popup.is_visible(),
				"no remote must show error"
			);
			drain_app_work(&mut app, &rx_git);
		}

		// 2. Multiple remotes without a default (no origin).
		{
			let (td, repo) = git2_testing::repo_init();
			repo.remote("upstream", "https://example.com/up.git")
				.unwrap();
			repo.remote("fork", "https://example.com/fork.git")
				.unwrap();
			let (mut app, kc, rx_git) =
				make_test_app_with_rx(td.path());
			app.set_tab(1).unwrap();

			send_key!(&mut app, kc.keys.push);
			assert!(
				!app.confirm_popup.is_visible(),
				"ambiguous remotes must not open confirm popup"
			);
			assert!(
				!app.push_tags_popup.is_visible(),
				"ambiguous remotes must not open push tags popup"
			);
			assert!(
				app.msg_popup.is_visible(),
				"ambiguous remotes must show error"
			);
			drain_app_work(&mut app, &rx_git);
		}
	}

	fn bare_has_tag(path: &Path, tag: &str) -> bool {
		sync::get_tags(&RepoPath::Path(path.to_path_buf()))
			.unwrap()
			.values()
			.flatten()
			.any(|t| t.name == tag)
	}

	fn init_three_remote_repo() -> (
		tempfile::TempDir,
		tempfile::TempDir,
		tempfile::TempDir,
		tempfile::TempDir,
	) {
		let (origin_td, _origin_bare) =
			git2_testing::repo_init_bare();
		let (pushdef_td, _pushdef_bare) =
			git2_testing::repo_init_bare();
		let (up_td, _up_bare) = git2_testing::repo_init_bare();

		let (td, repo) = git2_testing::repo_init();
		let orig_branch =
			repo.head().unwrap().shorthand().unwrap().to_string();
		let head_oid = repo.head().unwrap().target().unwrap();
		let head_obj = repo.find_object(head_oid, None).unwrap();
		repo.tag_lightweight("v2.0", &head_obj, false).unwrap();

		repo.remote("origin", origin_td.path().to_str().unwrap())
			.unwrap();
		repo.remote("pushdef", pushdef_td.path().to_str().unwrap())
			.unwrap();
		repo.remote("up", up_td.path().to_str().unwrap()).unwrap();

		for name in ["origin", "pushdef"] {
			let mut rem = repo.find_remote(name).unwrap();
			rem.push(&["refs/tags/v2.0:refs/tags/v2.0"], None)
				.unwrap();
		}

		{
			let mut cfg = repo.config().unwrap();
			cfg.set_str("remote.pushDefault", "pushdef").unwrap();
			cfg.set_str(
				&format!("branch.{orig_branch}.remote"),
				"up",
			)
			.unwrap();
			cfg.set_str(
				&format!("branch.{orig_branch}.merge"),
				&format!("refs/heads/{orig_branch}"),
			)
			.unwrap();
		}
		let up_ref = format!("refs/remotes/up/{orig_branch}");
		repo.reference(&up_ref, head_oid, true, "test").unwrap();
		commit_file(td.path(), "a.txt", "ahead", "ahead");

		(origin_td, pushdef_td, up_td, td)
	}

	#[test]
	fn t8f_prompt_names_push_remote() {
		let (origin_td, pushdef_td, _up_td, td) =
			init_three_remote_repo();
		assert!(bare_has_tag(origin_td.path(), "v2.0"));
		assert!(bare_has_tag(pushdef_td.path(), "v2.0"));

		let (mut app, kc, rx_git) = make_test_app_with_rx(td.path());

		// 1. Force-push on branch with upstream -> names upstream remote ("up").
		app.set_tab(0).unwrap();
		send_key!(&mut app, kc.keys.force_push);
		assert!(app.confirm_popup.is_visible());
		let (_, msg) = app.confirm_popup.get_text();
		assert!(
			msg.contains("up") && !msg.contains("pushdef"),
			"force-push with upstream must name 'up', got: {msg}"
		);
		send_key!(&mut app, kc.keys.exit_popup);

		// 2. Force-push on branch without upstream -> names remote.pushDefault ("pushdef").
		let rp = RepoPath::Path(td.path().to_path_buf());
		sync::create_branch(&rp, "topic").unwrap();
		app.update().unwrap();
		send_key!(&mut app, kc.keys.force_push);
		assert!(app.confirm_popup.is_visible());
		let (_, msg) = app.confirm_popup.get_text();
		assert!(
			msg.contains("pushdef") && !msg.contains("origin"),
			"force-push without upstream must name 'pushdef', got: {msg}"
		);
		send_key!(&mut app, kc.keys.exit_popup);

		// 3. Push tags -> names default remote ("origin").
		app.set_tab(1).unwrap();
		send_key!(&mut app, kc.keys.push);
		assert!(app.confirm_popup.is_visible());
		let (_, msg) = app.confirm_popup.get_text();
		assert!(
			msg.contains("origin") && !msg.contains("pushdef"),
			"push tags must name default remote 'origin', got: {msg}"
		);
		send_key!(&mut app, kc.keys.exit_popup);

		// 4. Remote-tag delete -> names 'origin' AND actually deletes from 'origin', not 'pushdef'.
		app.tags_popup.open().unwrap();
		send_key!(&mut app, kc.keys.delete_tag);
		assert!(app.confirm_popup.is_visible());
		send_key!(&mut app, kc.keys.enter);
		assert!(app.confirm_popup.is_visible());
		let (_, msg) = app.confirm_popup.get_text();
		assert!(
			msg.contains("origin") && !msg.contains("pushdef"),
			"remote tag delete must name 'origin', got: {msg}"
		);
		send_key!(&mut app, kc.keys.enter);
		drain_app_work(&mut app, &rx_git);
		assert!(
			!bare_has_tag(origin_td.path(), "v2.0"),
			"tag v2.0 must be deleted on origin"
		);
		assert!(
			bare_has_tag(pushdef_td.path(), "v2.0"),
			"tag v2.0 must remain on pushdef"
		);
	}

	#[test]
	fn t8f_changed_remote_refuses() {
		let (origin_td, origin_bare) = git2_testing::repo_init_bare();
		let (other_td, _other_bare) = git2_testing::repo_init_bare();
		let (td, repo) = git2_testing::repo_init();
		let head_oid = repo.head().unwrap().target().unwrap();
		let head_obj = repo.find_object(head_oid, None).unwrap();
		repo.tag_lightweight("v1", &head_obj, false).unwrap();
		repo.remote("origin", origin_td.path().to_str().unwrap())
			.unwrap();
		repo.remote("other", other_td.path().to_str().unwrap())
			.unwrap();
		repo.config()
			.unwrap()
			.set_str("remote.pushDefault", "origin")
			.unwrap();

		let (mut app, kc, rx_git) = make_test_app_with_rx(td.path());

		// 1. Force-push: pushDefault changes while prompt is open -> refuse with repo-changed error.
		app.set_tab(0).unwrap();
		send_key!(&mut app, kc.keys.force_push);
		assert!(app.confirm_popup.is_visible());
		repo.config()
			.unwrap()
			.set_str("remote.pushDefault", "other")
			.unwrap();
		send_key!(&mut app, kc.keys.enter);
		assert!(!app.confirm_popup.is_visible());
		assert!(!app.push_popup.is_visible());
		assert!(
			app.msg_popup.is_visible(),
			"changed pushDefault must show repo-changed error"
		);
		send_key!(&mut app, kc.keys.exit_popup);

		// 2. Force-push: remote deleted while pushDefault still names it -> refuse.
		app.update().unwrap();
		send_key!(&mut app, kc.keys.force_push);
		assert!(app.confirm_popup.is_visible());
		repo.remote_delete("other").unwrap();
		send_key!(&mut app, kc.keys.enter);
		assert!(!app.confirm_popup.is_visible());
		assert!(!app.push_popup.is_visible());
		assert!(
			app.msg_popup.is_visible(),
			"deleted remote must show repo-changed error"
		);
		send_key!(&mut app, kc.keys.exit_popup);

		// 3. Push tags: default remote removed while prompt is open -> refuse, nothing pushed.
		app.set_tab(1).unwrap();
		send_key!(&mut app, kc.keys.push);
		assert!(app.confirm_popup.is_visible());
		repo.remote_delete("origin").unwrap();
		send_key!(&mut app, kc.keys.enter);
		assert!(!app.confirm_popup.is_visible());
		assert!(!app.push_tags_popup.is_visible());
		assert!(
			app.msg_popup.is_visible(),
			"deleted default remote must refuse push tags"
		);
		assert!(origin_bare.tag_names(None).unwrap().is_empty());
		drain_app_work(&mut app, &rx_git);
	}

	#[test]
	fn t8f_stash_pop_names_stash() {
		let (td, mut repo) = git2_testing::repo_init();
		commit_file(td.path(), "f.txt", "v1", "init f");
		std::fs::write(td.path().join("f.txt"), "v2").unwrap();
		let sig = repo.signature().unwrap();
		let stash_oid =
			repo.stash_save(&sig, "wip login fix", None).unwrap();
		let short_id = CommitId::new(stash_oid).get_short_string();

		let (mut app, kc) = make_test_app(td.path());
		app.set_tab(4).unwrap();
		send_key!(&mut app, kc.keys.enter);
		assert!(app.confirm_popup.is_visible());
		let (_, msg) = app.confirm_popup.get_text();
		assert!(
			msg.contains(&short_id),
			"stash pop prompt must include short id {short_id}, got: {msg}"
		);
		assert!(
			msg.contains("wip login fix"),
			"stash pop prompt must include stash message, got: {msg}"
		);
	}

	#[test]
	fn t8f_tag_delete_names_tag() {
		let (bare_td, _bare_repo) = git2_testing::repo_init_bare();
		let (td, repo) = git2_testing::repo_init();
		let head_oid = repo.head().unwrap().target().unwrap();
		let head_obj = repo.find_object(head_oid, None).unwrap();
		repo.tag_lightweight("v1.2.3", &head_obj, false).unwrap();
		repo.remote("origin", bare_td.path().to_str().unwrap())
			.unwrap();

		let (mut app, kc) = make_test_app(td.path());
		app.tags_popup.open().unwrap();
		send_key!(&mut app, kc.keys.delete_tag);
		assert!(app.confirm_popup.is_visible());
		// Confirm local delete -> opens remote tag delete prompt.
		send_key!(&mut app, kc.keys.enter);
		assert!(app.confirm_popup.is_visible());
		let (_, msg) = app.confirm_popup.get_text();
		assert!(
			msg.contains("v1.2.3"),
			"remote tag delete prompt must name tag, got: {msg}"
		);
		assert!(
			msg.contains("origin"),
			"remote tag delete prompt must name remote, got: {msg}"
		);
	}

	#[test]
	fn t8f_discard_names_path() {
		let (td, _repo) = git2_testing::repo_init();
		std::fs::create_dir_all(td.path().join("src")).unwrap();
		commit_file(td.path(), "src/notes.txt", "a\n", "add notes");
		std::fs::write(td.path().join("src/notes.txt"), "a\nb\nc\n")
			.unwrap();

		let (mut app, kc) = make_test_app(td.path());
		let diff = sync::diff::get_diff(
			&RepoPath::Path(td.path().to_path_buf()),
			"src/notes.txt",
			false,
			None,
		)
		.unwrap();
		// Drive DiffComponent through App's queue & confirm_popup.
		let mut diff_comp = crate::components::DiffComponent::new(
			&Environment {
				queue: app.queue.clone(),
				theme: app.theme.clone(),
				key_config: app.key_config.clone(),
				repo: app.repo.clone(),
				options: app.options.clone(),
				sender_git: crossbeam_channel::unbounded().0,
				sender_app: crossbeam_channel::unbounded().0,
			},
			false,
		);
		diff_comp.focus(true);
		diff_comp.update("src/notes.txt".to_string(), false, diff);

		// 1. Hunk discard: names the file path.
		let reset_hunk_ev = Event::Key(KeyEvent::new(
			kc.keys.status_reset_item.code,
			kc.keys.status_reset_item.modifiers,
		));
		diff_comp.event(&reset_hunk_ev).unwrap();
		app.process_queue(NeedsUpdate::empty()).unwrap();
		assert!(app.confirm_popup.is_visible());
		let (_, hunk_msg) = app.confirm_popup.get_text();
		assert!(
			hunk_msg.contains("src/notes.txt"),
			"hunk discard prompt must name file path, got: {hunk_msg}"
		);
		send_key!(&mut app, kc.keys.exit_popup);

		// 2. Line discard (2 added lines): names the file path and line count.
		let down_ev = Event::Key(KeyEvent::new(
			kc.keys.move_down.code,
			kc.keys.move_down.modifiers,
		));
		let shift_down_ev = Event::Key(KeyEvent::new(
			kc.keys.shift_down.code,
			kc.keys.shift_down.modifiers,
		));
		// Move from header (0) -> context 'a' (1) -> added 'b' (2), then extend to 'c' (3).
		diff_comp.event(&down_ev).unwrap();
		diff_comp.event(&down_ev).unwrap();
		diff_comp.event(&shift_down_ev).unwrap();
		let reset_lines_ev = Event::Key(KeyEvent::new(
			kc.keys.diff_reset_lines.code,
			kc.keys.diff_reset_lines.modifiers,
		));
		diff_comp.event(&reset_lines_ev).unwrap();
		app.process_queue(NeedsUpdate::empty()).unwrap();
		assert!(app.confirm_popup.is_visible());
		let (_, lines_msg) = app.confirm_popup.get_text();
		assert!(
			lines_msg.contains("src/notes.txt"),
			"line discard prompt must name file path, got: {lines_msg}"
		);
		assert!(
			lines_msg.contains('2'),
			"line discard prompt must name line count, got: {lines_msg}"
		);
	}

	#[test]
	fn t8f_force_push_names_remote() {
		let (bare_td, _bare_repo) = git2_testing::repo_init_bare();
		let (td, repo) = git2_testing::repo_init();
		let rp = RepoPath::Path(td.path().to_path_buf());
		sync::create_branch(&rp, "feat/login").unwrap();
		repo.remote("upstream", bare_td.path().to_str().unwrap())
			.unwrap();

		let (mut app, kc) = make_test_app(td.path());
		app.set_tab(0).unwrap();
		send_key!(&mut app, kc.keys.force_push);
		assert!(app.confirm_popup.is_visible());
		let (_, msg) = app.confirm_popup.get_text();
		assert!(
			msg.contains("feat/login"),
			"force-push prompt must name full branch, got: {msg}"
		);
		assert!(
			msg.contains("upstream"),
			"force-push prompt must name remote, got: {msg}"
		);
	}

	fn make_files_test_app(
		repo_path: &Path,
		select_file: Option<PathBuf>,
	) -> (
		App,
		KeyConfig,
		crossbeam_channel::Receiver<AsyncGitNotification>,
		crossbeam_channel::Receiver<AsyncAppNotification>,
	) {
		type RxPair = (
			crossbeam_channel::Receiver<AsyncGitNotification>,
			crossbeam_channel::Receiver<AsyncAppNotification>,
		);
		static LIVE_RX: std::sync::OnceLock<
			std::sync::Mutex<Vec<RxPair>>,
		> = std::sync::OnceLock::new();

		let (tx_git, rx_git) = crossbeam_channel::unbounded();
		let (tx_app, rx_app) = crossbeam_channel::unbounded();
		LIVE_RX
			.get_or_init(|| std::sync::Mutex::new(Vec::new()))
			.lock()
			.unwrap()
			.push((rx_git.clone(), rx_app.clone()));
		let key_config = KeyConfig::default();
		let app = App::new(
			CliArgs {
				theme: PathBuf::default(),
				select_file,
				repo_path: RepoPath::Path(repo_path.to_path_buf()),
				notify_watcher: false,
				key_bindings_path: None,
				key_symbols_path: None,
			},
			tx_git,
			tx_app,
			Input::new(),
			Theme::default(),
			key_config.clone(),
		)
		.unwrap();
		(app, key_config, rx_git, rx_app)
	}

	fn wait_git(
		rx: &crossbeam_channel::Receiver<AsyncGitNotification>,
		expected: AsyncGitNotification,
	) {
		let deadline = std::time::Instant::now()
			+ std::time::Duration::from_secs(5);
		loop {
			assert!(
				std::time::Instant::now() < deadline,
				"timed out waiting for {expected:?}"
			);
			if let Ok(ev) =
				rx.recv_timeout(std::time::Duration::from_millis(50))
			{
				if ev == expected {
					return;
				}
			}
		}
	}

	fn wait_syntax_done(
		rx: &crossbeam_channel::Receiver<AsyncAppNotification>,
	) {
		let deadline = std::time::Instant::now()
			+ std::time::Duration::from_secs(5);
		loop {
			assert!(
				std::time::Instant::now() < deadline,
				"timed out waiting for SyntaxHighlighting::Done"
			);
			if rx.recv_timeout(std::time::Duration::from_millis(50))
				== Ok(AsyncAppNotification::SyntaxHighlighting(
					crate::SyntaxHighlightProgress::Done,
				)) {
				return;
			}
		}
	}

	fn render_app(app: &App, width: u16, height: u16) -> String {
		use ratatui::{backend::TestBackend, Terminal};
		let mut term =
			Terminal::new(TestBackend::new(width, height)).unwrap();
		term.draw(|f| app.draw(f).unwrap()).unwrap();
		let buf = term.backend().buffer();
		let mut out = String::new();
		for y in 0..buf.area.height {
			for x in 0..buf.area.width {
				out.push_str(buf[(x, y)].symbol());
			}
			out.push('\n');
		}
		out
	}

	fn is_viewer_focused(
		comp: &impl Component,
		kc: &KeyConfig,
	) -> bool {
		let mut cmds = Vec::new();
		comp.commands(&mut cmds, false);
		let scroll_name =
			strings::commands::scroll(&Rc::new(kc.clone())).name;
		cmds.iter().any(|c| c.text.name == scroll_name)
	}

	fn commit_file_nested(
		repo_path: &Path,
		file: &str,
		content: &[u8],
		msg: &str,
	) -> CommitId {
		let full = repo_path.join(file);
		if let Some(parent) = full.parent() {
			std::fs::create_dir_all(parent).unwrap();
		}
		std::fs::write(full, content).unwrap();
		let rp = RepoPath::Path(repo_path.to_path_buf());
		sync::stage_add_file(&rp, Path::new(file)).unwrap();
		sync::commit(&rp, msg).unwrap()
	}

	fn remove_and_commit(
		repo_path: &Path,
		file: &str,
		msg: &str,
	) -> CommitId {
		std::fs::remove_file(repo_path.join(file)).unwrap();
		let rp = RepoPath::Path(repo_path.to_path_buf());
		sync::stage_addremoved(&rp, Path::new(file)).unwrap();
		sync::commit(&rp, msg).unwrap()
	}

	#[test]
	fn t8p_unborn_not_loading() {
		let (td, _repo) = git2_testing::repo_init_empty();
		let (mut app, _kc, _rx_git, _rx_app) =
			make_files_test_app(td.path(), None);

		app.set_tab(2).unwrap();
		app.update().unwrap();
		app.process_queue(NeedsUpdate::empty()).unwrap();

		let screen = render_app(&app, 80, 24);
		assert!(
			screen.contains("Files at"),
			"unborn Files tab must render empty tree, got:\n{screen}"
		);
		assert!(
			!screen.contains("Loading"),
			"unborn Files tab must not show Loading, got:\n{screen}"
		);
		assert!(
			!app.msg_popup.is_visible(),
			"unborn Files tab must not show an error popup"
		);
	}

	#[test]
	fn t8p_first_commit_loads() {
		let (td, _repo) = git2_testing::repo_init_empty();
		let (mut app, _kc, rx_git, _rx_app) =
			make_files_test_app(td.path(), None);

		app.set_tab(2).unwrap();
		app.update().unwrap();

		let _c1 =
			commit_file(td.path(), "first.txt", "hello", "init");
		app.update().unwrap();
		wait_git(&rx_git, AsyncGitNotification::TreeFiles);
		app.update_async(AsyncNotification::Git(
			AsyncGitNotification::TreeFiles,
		))
		.unwrap();

		let screen = render_app(&app, 80, 24);
		assert!(
			screen.contains("first.txt"),
			"first commit must load files, got:\n{screen}"
		);
	}

	#[test]
	fn t8p_becomes_unborn_clears() {
		let (td, repo) = git2_testing::repo_init();
		let _c1 =
			commit_file(td.path(), "born.txt", "born-body", "c1");
		let (mut app, _kc, rx_git, _rx_app) =
			make_files_test_app(td.path(), None);

		app.set_tab(2).unwrap();
		wait_git(&rx_git, AsyncGitNotification::TreeFiles);
		app.update_async(AsyncNotification::Git(
			AsyncGitNotification::TreeFiles,
		))
		.unwrap();
		app.files_tab.file_finder_update(Path::new("./born.txt"));

		let screen = render_app(&app, 80, 24);
		assert!(screen.contains("born-body"));

		// Trigger a new commit load, then make HEAD unborn while it's in flight.
		let _c2 =
			commit_file(td.path(), "born.txt", "second-body", "c2");
		app.update().unwrap();
		repo.set_head("refs/heads/orphan-unborn").unwrap();
		app.update().unwrap();

		// Now deliver the late TreeFiles result from c2.
		wait_git(&rx_git, AsyncGitNotification::TreeFiles);
		app.update_async(AsyncNotification::Git(
			AsyncGitNotification::TreeFiles,
		))
		.unwrap();

		let screen = render_app(&app, 80, 24);
		assert!(
			!screen.contains("born.txt"),
			"tree must clear when HEAD becomes unborn, got:\n{screen}"
		);
		assert!(
			!screen.contains("born-body")
				&& !screen.contains("second-body"),
			"viewer must clear when HEAD becomes unborn, got:\n{screen}"
		);
		assert!(
			!screen.contains("Loading"),
			"unborn must not show Loading, got:\n{screen}"
		);
	}

	#[test]
	fn t8p_head_move_follows_path() {
		let (td, _repo) = git2_testing::repo_init();
		let v1 = (0..25)
			.map(|i| format!("v1-line-{i:02}"))
			.collect::<Vec<_>>()
			.join("\n");
		let v2 = (0..25)
			.map(|i| format!("v2-line-{i:02}"))
			.collect::<Vec<_>>()
			.join("\n");
		let _c1 = commit_file_nested(
			td.path(),
			"sub/a.txt",
			v1.as_bytes(),
			"c1",
		);
		let _c1b =
			commit_file(td.path(), "other.txt", "other", "c1b");

		let (mut app, kc, rx_git, _rx_app) =
			make_files_test_app(td.path(), None);

		app.set_tab(2).unwrap();
		wait_git(&rx_git, AsyncGitNotification::TreeFiles);
		app.update_async(AsyncNotification::Git(
			AsyncGitNotification::TreeFiles,
		))
		.unwrap();

		app.files_tab.file_finder_update(Path::new("./sub/a.txt"));
		let _ = render_app(&app, 80, 12);
		// Focus viewer and scroll to bottom.
		send_key!(&mut app, kc.keys.move_right);
		send_key!(&mut app, kc.keys.end);
		let scrolled = render_app(&app, 80, 12);
		assert!(scrolled.contains("v1-line-24"));

		// Move HEAD with new content for sub/a.txt.
		let _c2 = commit_file_nested(
			td.path(),
			"sub/a.txt",
			v2.as_bytes(),
			"c2",
		);
		app.update().unwrap();
		wait_git(&rx_git, AsyncGitNotification::TreeFiles);
		app.update_async(AsyncNotification::Git(
			AsyncGitNotification::TreeFiles,
		))
		.unwrap();

		let screen = render_app(&app, 80, 12);
		assert!(
			screen.contains("a.txt"),
			"ancestor dir must stay open and file visible, got:\n{screen}"
		);
		assert!(
			screen.contains("v2-line-00"),
			"viewer must show new content scrolled to top, got:\n{screen}"
		);
		assert!(!screen.contains("v1-line-"));
	}

	#[test]
	fn t8p_head_move_path_gone() {
		let (td, _repo) = git2_testing::repo_init();
		let _c1 = commit_file_nested(
			td.path(),
			"sub/gone.txt",
			b"gone-body",
			"c1",
		);
		let _c1b =
			commit_file(td.path(), "keep.txt", "keep-body", "c1b");

		let (mut app, kc, rx_git, _rx_app) =
			make_files_test_app(td.path(), None);

		app.set_tab(2).unwrap();
		wait_git(&rx_git, AsyncGitNotification::TreeFiles);
		app.update_async(AsyncNotification::Git(
			AsyncGitNotification::TreeFiles,
		))
		.unwrap();
		app.files_tab
			.file_finder_update(Path::new("./sub/gone.txt"));
		send_key!(&mut app, kc.keys.move_right);
		assert!(is_viewer_focused(&app.files_tab, &kc));

		let _c2 = remove_and_commit(td.path(), "sub/gone.txt", "c2");
		app.update().unwrap();
		wait_git(&rx_git, AsyncGitNotification::TreeFiles);
		app.update_async(AsyncNotification::Git(
			AsyncGitNotification::TreeFiles,
		))
		.unwrap();

		let screen = render_app(&app, 80, 24);
		assert!(
			!screen.contains("gone-body"),
			"viewer must clear when path is gone, got:\n{screen}"
		);
		assert!(
			!is_viewer_focused(&app.files_tab, &kc),
			"focus must return to tree when path is gone"
		);
	}

	#[test]
	fn t8p_user_choice_during_load_wins() {
		let (td, _repo) = git2_testing::repo_init();
		let _c1 = commit_file(td.path(), "a.txt", "a-v1", "c1");
		let _c1b = commit_file(td.path(), "b.txt", "b-v1", "c1b");

		let (mut app, _kc, rx_git, _rx_app) =
			make_files_test_app(td.path(), None);

		app.set_tab(2).unwrap();
		wait_git(&rx_git, AsyncGitNotification::TreeFiles);
		app.update_async(AsyncNotification::Git(
			AsyncGitNotification::TreeFiles,
		))
		.unwrap();
		app.files_tab.file_finder_update(Path::new("./a.txt"));

		// Move HEAD and start loading c2, without delivering TreeFiles yet.
		let _c2 = commit_file(td.path(), "b.txt", "b-v2", "c2");
		app.update().unwrap();

		// User picks b.txt during loading.
		app.files_tab.file_finder_update(Path::new("./b.txt"));

		wait_git(&rx_git, AsyncGitNotification::TreeFiles);
		app.update_async(AsyncNotification::Git(
			AsyncGitNotification::TreeFiles,
		))
		.unwrap();

		let screen = render_app(&app, 80, 24);
		assert!(
			screen.contains("b-v2"),
			"user choice during load must win, got:\n{screen}"
		);
		assert!(!screen.contains("a-v1"));
	}

	#[test]
	fn t8p_select_file_first_load() {
		let (td, _repo) = git2_testing::repo_init();
		let _c1 = commit_file_nested(
			td.path(),
			"dir/target.txt",
			b"target-body",
			"c1",
		);
		let _c1b =
			commit_file(td.path(), "other.txt", "other-body", "c1b");

		let (mut app, _kc, rx_git, _rx_app) = make_files_test_app(
			td.path(),
			Some(td.path().join("dir/target.txt")),
		);

		wait_git(&rx_git, AsyncGitNotification::TreeFiles);
		app.update_async(AsyncNotification::Git(
			AsyncGitNotification::TreeFiles,
		))
		.unwrap();

		let screen = render_app(&app, 80, 24);
		assert!(
			screen.contains("target-body"),
			"--select-file must select and show target on first load, got:\n{screen}"
		);
	}

	#[test]
	fn t8p_same_head_is_stable() {
		let (td, _repo) = git2_testing::repo_init();
		let lines = (0..25)
			.map(|i| format!("stable-{i:02}"))
			.collect::<Vec<_>>()
			.join("\n");
		let _c1 = commit_file(td.path(), "s.txt", &lines, "c1");

		let (mut app, kc, rx_git, rx_app) =
			make_files_test_app(td.path(), None);

		app.set_tab(2).unwrap();
		wait_git(&rx_git, AsyncGitNotification::TreeFiles);
		app.update_async(AsyncNotification::Git(
			AsyncGitNotification::TreeFiles,
		))
		.unwrap();

		app.files_tab.file_finder_update(Path::new("./s.txt"));
		wait_syntax_done(&rx_app);
		app.update_async(AsyncNotification::App(
			AsyncAppNotification::SyntaxHighlighting(
				crate::SyntaxHighlightProgress::Done,
			),
		))
		.unwrap();
		let _ = render_app(&app, 80, 12);
		send_key!(&mut app, kc.keys.move_right);
		send_key!(&mut app, kc.keys.end);
		let before = render_app(&app, 80, 12);
		assert!(before.contains("stable-24"));

		// Drain any prior git notifications before checking same-HEAD stability.
		while rx_git.try_recv().is_ok() {}

		// Same HEAD again: no reload, selection and scroll stay.
		app.update().unwrap();
		assert!(
			!app.files_tab.anything_pending(),
			"same HEAD must not trigger a reload"
		);
		while let Ok(ev) =
			rx_git.recv_timeout(std::time::Duration::from_millis(50))
		{
			assert_ne!(
				ev,
				AsyncGitNotification::TreeFiles,
				"same HEAD must not spawn a TreeFiles job"
			);
		}
		let after = render_app(&app, 80, 12);
		assert!(
			after.contains("stable-24"),
			"scroll must stay on same HEAD, got:\n{after}"
		);
	}

	#[test]
	fn t8p_popup_new_commit_resets() {
		let (td, _repo) = git2_testing::repo_init();
		let c1 = commit_file(td.path(), "a.txt", "pop-v1", "c1");
		let c2 = commit_file(td.path(), "a.txt", "pop-v2", "c2");

		let (mut app, kc, rx_git, _rx_app) =
			make_files_test_app(td.path(), None);

		app.revision_files_popup
			.open(crate::popups::FileTreeOpen::new(c1))
			.unwrap();
		wait_git(&rx_git, AsyncGitNotification::TreeFiles);
		app.update_async(AsyncNotification::Git(
			AsyncGitNotification::TreeFiles,
		))
		.unwrap();
		app.revision_files_popup
			.file_finder_update(Path::new("./a.txt"));
		send_key!(&mut app, kc.keys.move_right);
		assert!(is_viewer_focused(&app.revision_files_popup, &kc));

		// Close and reopen on SAME commit: state stays.
		send_key!(&mut app, kc.keys.exit_popup);
		app.revision_files_popup
			.open(crate::popups::FileTreeOpen::new(c1))
			.unwrap();
		let same_screen = render_app(&app, 80, 24);
		assert!(same_screen.contains("pop-v1"));
		assert!(is_viewer_focused(&app.revision_files_popup, &kc));

		// Close and reopen on DIFFERENT commit: top entry, clear viewer, tree focus.
		send_key!(&mut app, kc.keys.exit_popup);
		app.revision_files_popup
			.open(crate::popups::FileTreeOpen::new(c2))
			.unwrap();
		wait_git(&rx_git, AsyncGitNotification::TreeFiles);
		app.update_async(AsyncNotification::Git(
			AsyncGitNotification::TreeFiles,
		))
		.unwrap();

		let new_screen = render_app(&app, 80, 24);
		assert!(
			!new_screen.contains("pop-v1")
				&& !new_screen.contains("pop-v2"),
			"new commit in popup must clear viewer, got:\n{new_screen}"
		);
		assert!(
			!is_viewer_focused(&app.revision_files_popup, &kc),
			"new commit in popup must focus tree"
		);
	}

	#[test]
	fn t8p_empty_viewer_not_focusable() {
		let (td, _repo) = git2_testing::repo_init();
		let _c1 =
			commit_file_nested(td.path(), "dir/zero.txt", b"", "c1");

		let (mut app, kc, rx_git, _rx_app) =
			make_files_test_app(td.path(), None);

		// Before loading (nothing loaded): Right must not focus viewer.
		app.set_tab(2).unwrap();
		send_key!(&mut app, kc.keys.move_right);
		assert!(
			!is_viewer_focused(&app.files_tab, &kc),
			"nothing loaded must not be focusable"
		);

		wait_git(&rx_git, AsyncGitNotification::TreeFiles);
		app.update_async(AsyncNotification::Git(
			AsyncGitNotification::TreeFiles,
		))
		.unwrap();

		// Root directory selected: Right must not focus viewer.
		send_key!(&mut app, kc.keys.move_right);
		assert!(
			!is_viewer_focused(&app.files_tab, &kc),
			"directory selection must not focus viewer"
		);

		// Zero-line file selected: Right must not focus viewer.
		app.files_tab
			.file_finder_update(Path::new("./dir/zero.txt"));
		send_key!(&mut app, kc.keys.move_right);
		assert!(
			!is_viewer_focused(&app.files_tab, &kc),
			"zero-line file must not be focusable"
		);
	}

	#[test]
	fn t8p_content_focusable() {
		let (td, _repo) = git2_testing::repo_init();
		let _c1 =
			commit_file(td.path(), "text.txt", "hello\nworld", "c1");
		let _c2 = commit_file_nested(
			td.path(),
			"bin.dat",
			&[0, 1, 2, 0, 255],
			"c2",
		);

		let (mut app, kc, rx_git, rx_app) =
			make_files_test_app(td.path(), None);

		app.set_tab(2).unwrap();
		wait_git(&rx_git, AsyncGitNotification::TreeFiles);
		app.update_async(AsyncNotification::Git(
			AsyncGitNotification::TreeFiles,
		))
		.unwrap();

		// Content still being highlighted: focusable.
		app.files_tab.file_finder_update(Path::new("./text.txt"));
		send_key!(&mut app, kc.keys.move_right);
		assert!(
			is_viewer_focused(&app.files_tab, &kc),
			"content still being highlighted must be focusable"
		);
		send_key!(&mut app, kc.keys.move_left);

		// Highlighted content: focusable.
		wait_syntax_done(&rx_app);
		app.update_async(AsyncNotification::App(
			AsyncAppNotification::SyntaxHighlighting(
				crate::SyntaxHighlightProgress::Done,
			),
		))
		.unwrap();
		send_key!(&mut app, kc.keys.move_right);
		assert!(
			is_viewer_focused(&app.files_tab, &kc),
			"highlighted content must be focusable"
		);
		send_key!(&mut app, kc.keys.move_left);

		// Error text (binary file): focusable.
		app.files_tab.file_finder_update(Path::new("./bin.dat"));
		send_key!(&mut app, kc.keys.move_right);
		assert!(
			is_viewer_focused(&app.files_tab, &kc),
			"error text must be focusable"
		);
	}

	#[test]
	fn t8p_emptied_viewer_returns_focus() {
		let (td, _repo) = git2_testing::repo_init();
		let _c1 = commit_file(td.path(), "f.txt", "non-empty", "c1");

		let (mut app, kc, rx_git, _rx_app) =
			make_files_test_app(td.path(), None);

		app.set_tab(2).unwrap();
		wait_git(&rx_git, AsyncGitNotification::TreeFiles);
		app.update_async(AsyncNotification::Git(
			AsyncGitNotification::TreeFiles,
		))
		.unwrap();

		app.files_tab.file_finder_update(Path::new("./f.txt"));
		send_key!(&mut app, kc.keys.move_right);
		assert!(is_viewer_focused(&app.files_tab, &kc));

		// Replace f.txt with a zero-line file on next commit: viewer becomes empty.
		let _c2 = commit_file(td.path(), "f.txt", "", "c2");
		app.update().unwrap();
		wait_git(&rx_git, AsyncGitNotification::TreeFiles);
		app.update_async(AsyncNotification::Git(
			AsyncGitNotification::TreeFiles,
		))
		.unwrap();

		assert!(
			!is_viewer_focused(&app.files_tab, &kc),
			"focused viewer becoming empty must return focus to tree"
		);
	}

	#[test]
	fn t8p_edit_closes_stack() {
		let (td, _repo) = git2_testing::repo_init();
		let c1 = commit_file(td.path(), "edit.txt", "body", "c1");

		let (mut app, kc, rx_git, _rx_app) =
			make_files_test_app(td.path(), None);

		app.set_tab(1).unwrap();
		app.popup_stack.push(StackablePopupOpen::InspectCommit(
			crate::popups::InspectCommitOpen::new(c1),
		));
		app.revision_files_popup
			.open(crate::popups::FileTreeOpen::new(c1))
			.unwrap();
		wait_git(&rx_git, AsyncGitNotification::TreeFiles);
		app.update_async(AsyncNotification::Git(
			AsyncGitNotification::TreeFiles,
		))
		.unwrap();

		app.revision_files_popup
			.file_finder_update(Path::new("./edit.txt"));
		send_key!(&mut app, kc.keys.edit_file);

		assert!(
			!app.revision_files_popup.is_visible(),
			"revision files popup must close on edit"
		);
		assert!(
			app.popup_stack.is_empty(),
			"popup stack must be cleared on edit"
		);
		assert_eq!(app.current_tab(), 0, "must switch to Status tab");
		assert!(
			app.external_editor_popup.is_visible(),
			"editor popup must open"
		);
		assert_eq!(app.file_to_open.as_deref(), Some("./edit.txt"));
		assert!(!app.msg_popup.is_visible());
	}

	#[test]
	fn t8p_edit_missing_keeps_popup() {
		let (td, _repo) = git2_testing::repo_init();
		let c1 = commit_file(td.path(), "gone.txt", "body", "c1");
		std::fs::remove_file(td.path().join("gone.txt")).unwrap();

		let (mut app, kc, rx_git, _rx_app) =
			make_files_test_app(td.path(), None);

		app.set_tab(1).unwrap();
		app.popup_stack.push(StackablePopupOpen::InspectCommit(
			crate::popups::InspectCommitOpen::new(c1),
		));
		app.revision_files_popup
			.open(crate::popups::FileTreeOpen::new(c1))
			.unwrap();
		wait_git(&rx_git, AsyncGitNotification::TreeFiles);
		app.update_async(AsyncNotification::Git(
			AsyncGitNotification::TreeFiles,
		))
		.unwrap();

		app.revision_files_popup
			.file_finder_update(Path::new("./gone.txt"));
		send_key!(&mut app, kc.keys.edit_file);

		assert!(
			app.msg_popup.is_visible(),
			"missing worktree file must show error"
		);
		assert!(
			app.revision_files_popup.is_visible(),
			"popup must stay open when worktree file is missing"
		);
		assert!(
			!app.popup_stack.is_empty(),
			"popup stack must stay unchanged"
		);
		assert_eq!(app.current_tab(), 1, "tab must stay unchanged");
		assert!(!app.external_editor_popup.is_visible());
	}

	#[test]
	fn t8p_edit_from_tab() {
		let (td, _repo) = git2_testing::repo_init();
		let _c1 = commit_file(td.path(), "tab.txt", "body", "c1");

		let (mut app, kc, rx_git, _rx_app) =
			make_files_test_app(td.path(), None);

		app.set_tab(2).unwrap();
		wait_git(&rx_git, AsyncGitNotification::TreeFiles);
		app.update_async(AsyncNotification::Git(
			AsyncGitNotification::TreeFiles,
		))
		.unwrap();

		app.files_tab.file_finder_update(Path::new("./tab.txt"));
		send_key!(&mut app, kc.keys.edit_file);

		assert_eq!(app.current_tab(), 0);
		assert!(app.external_editor_popup.is_visible());
		assert_eq!(app.file_to_open.as_deref(), Some("./tab.txt"));
	}

	fn init_origin_and_fork_repo(
		upstream_fork_push_default_origin: bool,
	) -> (tempfile::TempDir, tempfile::TempDir, String) {
		let (bare_td, _bare_repo) = git2_testing::repo_init_bare();
		let (td, repo) = git2_testing::repo_init();
		let branch =
			repo.head().unwrap().shorthand().unwrap().to_string();
		repo.remote("origin", bare_td.path().to_str().unwrap())
			.unwrap();
		repo.remote(
			"fork",
			"https://forkuser@fork.example.org/r.git",
		)
		.unwrap();
		if upstream_fork_push_default_origin {
			let mut cfg = repo.config().unwrap();
			cfg.set_str(&format!("branch.{branch}.remote"), "fork")
				.unwrap();
			cfg.set_str("remote.pushDefault", "origin").unwrap();
		}
		(bare_td, td, branch)
	}

	#[test]
	fn t9c_tag_delete_fork_creds() {
		let (_bare_td, td, _branch) =
			init_origin_and_fork_repo(false);
		let (mut app, _kc) = make_test_app(td.path());

		app.push_popup
			.delete_remote_tag("fork".into(), "v1".into())
			.unwrap();

		assert!(
			app.push_popup.input_cred().is_visible(),
			"credential prompt must open for fork remote-tag delete"
		);
		assert_eq!(
			app.push_popup
				.input_cred()
				.get_cred()
				.username
				.as_deref(),
			Some("forkuser")
		);
	}

	#[test]
	fn t9c_branch_push_upstream_creds() {
		let (_bare_td, td, branch) = init_origin_and_fork_repo(true);
		let (mut app, _kc) = make_test_app(td.path());

		app.push_popup
			.push(branch, PushType::Branch, false, false)
			.unwrap();

		assert!(
			app.push_popup.input_cred().is_visible(),
			"credential prompt must open for upstream fork branch push"
		);
		assert_eq!(
			app.push_popup
				.input_cred()
				.get_cred()
				.username
				.as_deref(),
			Some("forkuser")
		);
	}

	#[test]
	fn t9c_push_tags_named_remote() {
		let (_bare_td, td, _branch) =
			init_origin_and_fork_repo(false);
		let (mut app, _kc) = make_test_app(td.path());

		app.push_tags_popup.push_tags("fork".into()).unwrap();

		assert!(
			app.push_tags_popup.input_cred().is_visible(),
			"credential prompt must open for fork push_tags"
		);
		assert_eq!(
			app.push_tags_popup
				.input_cred()
				.get_cred()
				.username
				.as_deref(),
			Some("forkuser")
		);
	}
}
