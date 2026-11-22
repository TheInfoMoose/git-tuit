use crate::components::{
	time_to_string, visibility_blocking, CommandBlocking,
	CommandInfo, Component, DrawableComponent, EventState,
};
use crate::{
	app::Environment,
	components::ScrollType,
	keys::{key_match, SharedKeyConfig},
	queue::{Action, InternalEvent, Queue},
	strings,
	ui::{self, Size},
	AsyncNotification,
};
use anyhow::Result;
use asyncgit::{
	asyncjob::AsyncSingleJob,
	remote_tags::AsyncRemoteTagsJob,
	sync::cred::{
		extract_username_password, need_username_password,
		BasicAuthCredential,
	},
	sync::{
		self, get_tags_with_metadata, RepoPathRef, TagWithMetadata,
	},
	AsyncGitNotification,
};

use crossterm::event::Event;
use ratatui::{
	layout::{Constraint, Margin, Rect},
	text::Span,
	widgets::{
		Block, BorderType, Borders, Cell, Clear, Row, Table,
		TableState,
	},
	Frame,
};
use ui::style::SharedTheme;

enum RemoteTagsStatus {
	Unknown,
	Loaded(Vec<String>),
	Failed,
}

///
pub struct TagListPopup {
	repo: RepoPathRef,
	theme: SharedTheme,
	queue: Queue,
	tags: Option<Vec<TagWithMetadata>>,
	visible: bool,
	table_state: std::cell::Cell<TableState>,
	current_height: std::cell::Cell<usize>,
	remote_tags: RemoteTagsStatus,
	has_remotes: bool,
	basic_credential: Option<BasicAuthCredential>,
	async_remote_tags: AsyncSingleJob<AsyncRemoteTagsJob>,
	key_config: SharedKeyConfig,
}

impl DrawableComponent for TagListPopup {
	fn draw(&self, f: &mut Frame, rect: Rect) -> Result<()> {
		if self.visible {
			const PERCENT_SIZE: Size = Size::new(80, 50);
			const MIN_SIZE: Size = Size::new(60, 20);

			let area = ui::centered_rect(
				PERCENT_SIZE.width,
				PERCENT_SIZE.height,
				f.area(),
			);
			let area =
				ui::rect_inside(MIN_SIZE, f.area().into(), area);
			let area = area.intersection(rect);

			let tag_name_width =
				self.tags.as_ref().map_or(0, |tags| {
					tags.iter()
						.fold(0, |acc, tag| acc.max(tag.name.len()))
				});

			let constraints = [
				// symbol if tag is not yet on remote and can be pushed
				Constraint::Length(1),
				// tag name
				Constraint::Length(tag_name_width.try_into()?),
				// commit date
				Constraint::Length(10),
				// author width
				Constraint::Length(19),
				// attachment
				Constraint::Length(1),
				// commit id
				Constraint::Percentage(100),
			];

			let rows = self.get_rows();
			let number_of_rows = rows.len();

			let table = Table::new(rows, constraints)
				.column_spacing(1)
				.row_highlight_style(self.theme.text(true, true))
				.block(
					Block::default()
						.borders(Borders::ALL)
						.title(Span::styled(
							self.title(),
							self.theme.title(true),
						))
						.border_style(self.theme.block(true))
						.border_type(BorderType::Thick),
				);

			let mut table_state = self.table_state.take();

			f.render_widget(Clear, area);
			f.render_stateful_widget(table, area, &mut table_state);

			let area = area.inner(Margin {
				vertical: 1,
				horizontal: 0,
			});

			ui::draw_scrollbar(
				f,
				area,
				&self.theme,
				number_of_rows,
				table_state.selected().unwrap_or(0),
				ui::Orientation::Vertical,
			);

			self.table_state.set(table_state);
			self.current_height.set(area.height.into());
		}

		Ok(())
	}
}

impl Component for TagListPopup {
	fn commands(
		&self,
		out: &mut Vec<CommandInfo>,
		force_all: bool,
	) -> CommandBlocking {
		if self.visible || force_all {
			if !force_all {
				out.clear();
			}

			out.push(CommandInfo::new(
				strings::commands::scroll(&self.key_config),
				true,
				true,
			));

			out.push(CommandInfo::new(
				strings::commands::close_popup(&self.key_config),
				true,
				true,
			));

			out.push(CommandInfo::new(
				strings::commands::delete_tag_popup(&self.key_config),
				self.valid_selection(),
				true,
			));
			out.push(CommandInfo::new(
				strings::commands::select_tag(&self.key_config),
				self.valid_selection(),
				true,
			));
			out.push(CommandInfo::new(
				strings::commands::push_tags(&self.key_config),
				self.has_remotes,
				true,
			));
			out.push(CommandInfo::new(
				strings::commands::show_tag_annotation(
					&self.key_config,
				),
				self.can_show_annotation(),
				true,
			));
		}
		visibility_blocking(self)
	}

	fn event(&mut self, event: &Event) -> Result<EventState> {
		if self.visible {
			if let Event::Key(key) = event {
				if key_match(key, self.key_config.keys.exit_popup) {
					self.hide();
				} else if key_match(key, self.key_config.keys.move_up)
				{
					self.move_selection(ScrollType::Up);
				} else if key_match(
					key,
					self.key_config.keys.move_down,
				) {
					self.move_selection(ScrollType::Down);
				} else if key_match(
					key,
					self.key_config.keys.shift_up,
				) || key_match(
					key,
					self.key_config.keys.home,
				) {
					self.move_selection(ScrollType::Home);
				} else if key_match(
					key,
					self.key_config.keys.shift_down,
				) || key_match(
					key,
					self.key_config.keys.end,
				) {
					self.move_selection(ScrollType::End);
				} else if key_match(
					key,
					self.key_config.keys.page_down,
				) {
					self.move_selection(ScrollType::PageDown);
				} else if key_match(key, self.key_config.keys.page_up)
				{
					self.move_selection(ScrollType::PageUp);
				} else if key_match(
					key,
					self.key_config.keys.move_right,
				) && self.can_show_annotation()
				{
					self.show_annotation();
				} else if key_match(
					key,
					self.key_config.keys.delete_tag,
				) {
					return self.selected_tag().map_or(
						Ok(EventState::Consumed),
						|tag| {
							self.queue.push(
								InternalEvent::ConfirmAction(
									Action::DeleteTag(
										tag.name.clone(),
									),
								),
							);
							Ok(EventState::Consumed)
						},
					);
				} else if key_match(
					key,
					self.key_config.keys.select_tag,
				) {
					return self.selected_tag().map_or(
						Ok(EventState::Consumed),
						|tag| {
							self.queue.push(
								InternalEvent::SelectCommitInRevlog(
									tag.commit_id,
								),
							);
							Ok(EventState::Consumed)
						},
					);
				} else if key_match(key, self.key_config.keys.push)
					&& self.has_remotes
				{
					self.queue.push(InternalEvent::PushTags);
				} else if key_match(
					key,
					self.key_config.keys.cmd_bar_toggle,
				) {
					return Ok(EventState::NotConsumed);
				}
			}

			Ok(EventState::Consumed)
		} else {
			Ok(EventState::NotConsumed)
		}
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

impl TagListPopup {
	pub fn new(env: &Environment) -> Self {
		Self {
			theme: env.theme.clone(),
			queue: env.queue.clone(),
			tags: None,
			visible: false,
			has_remotes: false,
			table_state: std::cell::Cell::new(TableState::default()),
			current_height: std::cell::Cell::new(0),
			basic_credential: None,
			remote_tags: RemoteTagsStatus::Unknown,
			async_remote_tags: AsyncSingleJob::new(
				env.sender_git.clone(),
			),
			key_config: env.key_config.clone(),
			repo: env.repo.clone(),
		}
	}

	fn title(&self) -> String {
		if matches!(self.remote_tags, RemoteTagsStatus::Failed) {
			strings::title_tags_remote_unknown()
		} else {
			strings::title_tags()
		}
	}

	///
	pub fn open(&mut self) -> Result<()> {
		self.table_state.get_mut().select(Some(0));
		self.show()?;

		self.has_remotes =
			sync::get_branches_info(&self.repo.borrow(), false)
				.is_ok_and(|branches| !branches.is_empty());

		let basic_credential = if self.has_remotes {
			match need_username_password(&self.repo.borrow()) {
				Ok(true) => {
					match extract_username_password(
						&self.repo.borrow(),
					) {
						Ok(credential)
							if credential.is_complete() =>
						{
							Some(credential)
						}
						Ok(_) => None,
						Err(e) => {
							log::error!(
								"remote tags credential error: {e}"
							);
							None
						}
					}
				}
				Ok(false) => None,
				Err(e) => {
					log::error!("remote tags error: {e}");
					None
				}
			}
		} else {
			self.remote_tags = RemoteTagsStatus::Unknown;
			None
		};

		self.basic_credential = basic_credential;

		self.update_tags()?;
		self.update_missing_remote_tags();

		Ok(())
	}

	///
	pub fn update(&mut self, ev: AsyncNotification) {
		if matches!(
			ev,
			AsyncNotification::Git(AsyncGitNotification::RemoteTags)
		) {
			if let Some(job) = self.async_remote_tags.take_last() {
				match job.result() {
					Some(Ok(missing_remote_tags)) => {
						self.remote_tags = RemoteTagsStatus::Loaded(
							missing_remote_tags,
						);
					}
					Some(Err(e)) => {
						log::error!("remote tags error: {e}");
						self.remote_tags = RemoteTagsStatus::Failed;
					}
					None => {}
				}
			}
		} else if matches!(
			ev,
			AsyncNotification::Git(AsyncGitNotification::PushTags)
		) {
			self.update_missing_remote_tags();
		}
	}

	///
	pub fn any_work_pending(&self) -> bool {
		self.async_remote_tags.is_pending()
	}

	/// fetch list of tags
	pub fn update_tags(&mut self) -> Result<()> {
		let tags = get_tags_with_metadata(&self.repo.borrow())?;

		self.tags = Some(tags);

		Ok(())
	}

	pub fn update_missing_remote_tags(&self) {
		if self.has_remotes {
			self.async_remote_tags.spawn(AsyncRemoteTagsJob::new(
				self.repo.borrow().clone(),
				self.basic_credential.clone(),
			));
		}
	}

	///
	fn move_selection(&self, scroll_type: ScrollType) -> bool {
		let mut table_state = self.table_state.take();

		let old_selection = table_state.selected().unwrap_or(0);
		let max_selection = self
			.tags
			.as_ref()
			.map_or(0, |tags| tags.len().saturating_sub(1));

		let new_selection = match scroll_type {
			ScrollType::Up => old_selection.saturating_sub(1),
			ScrollType::Down => {
				old_selection.saturating_add(1).min(max_selection)
			}
			ScrollType::Home => 0,
			ScrollType::End => max_selection,
			ScrollType::PageUp => old_selection.saturating_sub(
				self.current_height.get().saturating_sub(1),
			),
			ScrollType::PageDown => old_selection
				.saturating_add(
					self.current_height.get().saturating_sub(1),
				)
				.min(max_selection),
		};

		let needs_update = new_selection != old_selection;

		table_state.select(Some(new_selection));
		self.table_state.set(table_state);

		needs_update
	}

	fn show_annotation(&self) {
		if let Some(tag) = self.selected_tag() {
			if let Some(annotation) = &tag.annotation {
				self.queue.push(InternalEvent::ShowInfoMsg(
					annotation.clone(),
				));
			}
		}
	}

	fn can_show_annotation(&self) -> bool {
		self.selected_tag()
			.and_then(|t| t.annotation.as_ref())
			.is_some()
	}

	///
	fn get_rows(&self) -> Vec<Row<'_>> {
		self.tags.as_ref().map_or_else(Vec::new, |tags| {
			tags.iter().map(|tag| self.get_row(tag)).collect()
		})
	}

	///
	fn get_row(&self, tag: &TagWithMetadata) -> Row<'_> {
		const UPSTREAM_SYMBOL: &str = "\u{2191}";
		const ATTACHMENT_SYMBOL: &str = "@";
		const EMPTY_SYMBOL: &str = " ";

		let is_tag_missing_on_remote = match &self.remote_tags {
			RemoteTagsStatus::Loaded(missing_remote_tags) => {
				let remote_tag = format!("refs/tags/{}", tag.name);

				missing_remote_tags.contains(&remote_tag)
			}
			RemoteTagsStatus::Unknown | RemoteTagsStatus::Failed => {
				false
			}
		};

		let has_remote_str = if is_tag_missing_on_remote {
			UPSTREAM_SYMBOL
		} else {
			EMPTY_SYMBOL
		};

		let has_attachment_str = if tag.annotation.is_some() {
			ATTACHMENT_SYMBOL
		} else {
			EMPTY_SYMBOL
		};

		let cells: Vec<Cell> = vec![
			Cell::from(has_remote_str)
				.style(self.theme.commit_author(false)),
			Cell::from(tag.name.clone())
				.style(self.theme.text(true, false)),
			Cell::from(time_to_string(tag.time, true))
				.style(self.theme.commit_time(false)),
			Cell::from(tag.author.clone())
				.style(self.theme.commit_author(false)),
			Cell::from(has_attachment_str)
				.style(self.theme.text_danger()),
			Cell::from(tag.message.clone())
				.style(self.theme.text(true, false)),
		];

		Row::new(cells)
	}

	fn valid_selection(&self) -> bool {
		self.selected_tag().is_some()
	}

	fn selected_tag(&self) -> Option<&TagWithMetadata> {
		self.tags.as_ref().and_then(|tags| {
			let table_state = self.table_state.take();

			let tag = table_state
				.selected()
				.and_then(|selected| tags.get(selected));

			self.table_state.set(table_state);

			tag
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crossbeam_channel::{unbounded, Receiver};
	use ratatui::{backend::TestBackend, Terminal};
	use std::{cell::RefCell, time::Duration};

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

	fn wait_remote_tags(
		popup: &mut TagListPopup,
		rx: &Receiver<AsyncGitNotification>,
	) {
		loop {
			match rx.recv_timeout(Duration::from_secs(5)) {
				Ok(AsyncGitNotification::RemoteTags) | Err(_) => {
					break
				}
				Ok(_) => {}
			}
		}
		popup.update(AsyncNotification::Git(
			AsyncGitNotification::RemoteTags,
		));
	}

	fn render_popup(popup: &TagListPopup) -> String {
		let backend = TestBackend::new(80, 24);
		let mut terminal = Terminal::new(backend).unwrap();
		terminal
			.draw(|f| {
				let area = f.area();
				popup.draw(f, area).unwrap();
			})
			.unwrap();
		let buf = terminal.backend().buffer();
		let mut out = String::new();
		for y in 0..buf.area.height {
			for x in 0..buf.area.width {
				out.push_str(buf[(x, y)].symbol());
			}
			out.push('\n');
		}
		out
	}

	#[test]
	fn t7n_remote_tags_failure_is_inline() {
		// Case 1: unreachable remote (offline).
		let (td, repo) = git2_testing::repo_init();
		let head_oid = repo.head().unwrap().target().unwrap();
		repo.remote(
			"origin",
			td.path()
				.join("nonexistent-remote.git")
				.to_str()
				.unwrap(),
		)
		.unwrap();
		repo.reference(
			"refs/remotes/origin/main",
			head_oid,
			true,
			"test",
		)
		.unwrap();

		let (env, rx) = env_for_repo(td.path());
		let mut popup = TagListPopup::new(&env);
		popup.open().unwrap();
		wait_remote_tags(&mut popup, &rx);

		assert!(env.queue.pop().is_none());
		let rendered = render_popup(&popup);
		assert!(
			rendered.contains("remote status unknown"),
			"expected inline unknown status, got:\n{rendered}"
		);

		// Case 2: several remotes without origin.
		let (td2, repo2) = git2_testing::repo_init();
		let head_oid2 = repo2.head().unwrap().target().unwrap();
		repo2
			.remote("upstream", "https://example.com/upstream.git")
			.unwrap();
		repo2
			.remote("fork", "https://example.com/fork.git")
			.unwrap();
		repo2
			.reference(
				"refs/remotes/upstream/main",
				head_oid2,
				true,
				"test",
			)
			.unwrap();

		let (env2, rx2) = env_for_repo(td2.path());
		let mut popup2 = TagListPopup::new(&env2);
		popup2.open().unwrap();
		wait_remote_tags(&mut popup2, &rx2);

		assert!(env2.queue.pop().is_none());
		let rendered2 = render_popup(&popup2);
		assert!(
			rendered2.contains("remote status unknown"),
			"expected inline unknown status, got:\n{rendered2}"
		);
	}

	#[test]
	fn t7n_remote_tags_state_resets() {
		let (bare_td, _bare_repo) = git2_testing::repo_init_bare();
		let (td, repo) = git2_testing::repo_init();
		let head_oid = repo.head().unwrap().target().unwrap();
		let head_obj = repo.find_object(head_oid, None).unwrap();
		repo.tag_lightweight("v1", &head_obj, false).unwrap();

		let bare_url = bare_td.path().to_str().unwrap().to_owned();
		repo.remote("origin", &bare_url).unwrap();
		repo.reference(
			"refs/remotes/origin/main",
			head_oid,
			true,
			"test",
		)
		.unwrap();

		let (env, rx) = env_for_repo(td.path());
		let mut popup = TagListPopup::new(&env);

		// First lookup succeeds: v1 is missing on remote -> shows ↑, no unknown notice.
		popup.open().unwrap();
		wait_remote_tags(&mut popup, &rx);
		assert!(env.queue.pop().is_none());
		let rendered_ok = render_popup(&popup);
		assert!(rendered_ok.contains('\u{2191}'));
		assert!(!rendered_ok.contains("remote status unknown"));

		// Break remote so next lookup fails: old ↑ markers cleared, unknown notice shown, no popup.
		repo.remote_set_url(
			"origin",
			td.path()
				.join("nonexistent-remote.git")
				.to_str()
				.unwrap(),
		)
		.unwrap();
		popup.update_missing_remote_tags();
		wait_remote_tags(&mut popup, &rx);
		assert!(env.queue.pop().is_none());
		let rendered_err = render_popup(&popup);
		assert!(
			!rendered_err.contains('\u{2191}'),
			"expected old remote markers to be cleared, got:\n{rendered_err}"
		);
		assert!(rendered_err.contains("remote status unknown"));

		// Restore remote so later success clears unknown notice and restores ↑.
		repo.remote_set_url("origin", &bare_url).unwrap();
		popup.update_missing_remote_tags();
		wait_remote_tags(&mut popup, &rx);
		assert!(env.queue.pop().is_none());
		let rendered_ok2 = render_popup(&popup);
		assert!(rendered_ok2.contains('\u{2191}'));
		assert!(!rendered_ok2.contains("remote status unknown"));
	}
}
