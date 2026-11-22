use crate::{
	app::Environment,
	components::{
		visibility_blocking, CommandBlocking, CommandInfo, Component,
		DrawableComponent, EventState,
	},
	keys::{key_match, SharedKeyConfig},
	queue::{Action, InternalEvent, Queue},
	strings, ui,
};
use anyhow::Result;
use asyncgit::sync::{self, RepoPathRef, RepoSnapshot};
use crossterm::event::Event;
use ratatui::{layout::Rect, text::Text, widgets::Clear, Frame};
use std::borrow::Cow;
use ui::style::SharedTheme;
use unicode_width::UnicodeWidthStr;

use super::popup_paragraph;

fn wrapped_line_count(msg: &str, inner_width: usize) -> usize {
	if inner_width == 0 {
		return 1;
	}
	let mut total = 0usize;
	for raw_line in msg.split('\n') {
		if raw_line.is_empty() {
			total += 1;
			continue;
		}
		let mut lines = 1usize;
		let mut col = 0usize;
		for word in raw_line.split_whitespace() {
			let w = word.width();
			if col == 0 {
				if w > inner_width {
					lines += w.div_ceil(inner_width) - 1;
					col = w % inner_width;
					if col == 0 {
						col = inner_width;
					}
				} else {
					col = w;
				}
			} else if col + 1 + w <= inner_width {
				col += 1 + w;
			} else {
				lines += 1;
				if w > inner_width {
					lines += w.div_ceil(inner_width) - 1;
					col = w % inner_width;
					if col == 0 {
						col = inner_width;
					}
				} else {
					col = w;
				}
			}
		}
		total += lines;
	}
	total.max(1)
}

///
pub struct ConfirmPopup {
	repo: RepoPathRef,
	target: Option<Action>,
	snapshot: Option<RepoSnapshot>,
	visible: bool,
	queue: Queue,
	theme: SharedTheme,
	key_config: SharedKeyConfig,
}

impl DrawableComponent for ConfirmPopup {
	fn draw(&self, f: &mut Frame, _rect: Rect) -> Result<()> {
		if self.visible {
			let frame_area = f.area();
			let width = frame_area
				.width
				.saturating_sub(4)
				.clamp(40, 64)
				.min(frame_area.width);
			let inner_width = usize::from(width.saturating_sub(2));
			let max_name_width =
				inner_width.saturating_sub(24).clamp(16, 40);

			let (title, msg) =
				self.get_text_with_limit(max_name_width);
			let needed = u16::try_from(
				wrapped_line_count(&msg, inner_width)
					.saturating_add(2),
			)
			.unwrap_or(frame_area.height);

			let txt = Text::styled(
				Cow::from(msg),
				self.theme.text_danger(),
			);

			let paragraph =
				popup_paragraph(&title, txt, &self.theme, true, true);
			let max_height =
				frame_area.height.saturating_sub(2).max(5);
			let height =
				needed.clamp(5, max_height).min(frame_area.height);

			let area =
				ui::centered_rect_absolute(width, height, frame_area);
			f.render_widget(Clear, area);
			f.render_widget(paragraph, area);
		}

		Ok(())
	}
}

impl Component for ConfirmPopup {
	fn commands(
		&self,
		out: &mut Vec<CommandInfo>,
		_force_all: bool,
	) -> CommandBlocking {
		out.push(CommandInfo::new(
			strings::commands::confirm_action(&self.key_config),
			true,
			self.visible,
		));
		out.push(CommandInfo::new(
			strings::commands::close_popup(&self.key_config),
			true,
			self.visible,
		));

		visibility_blocking(self)
	}

	fn event(&mut self, ev: &Event) -> Result<EventState> {
		if self.visible {
			if let Event::Key(e) = ev {
				if key_match(e, self.key_config.keys.exit_popup) {
					self.hide();
				} else if key_match(e, self.key_config.keys.enter) {
					self.confirm();
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
		self.target = None;
		self.snapshot = None;
	}

	fn show(&mut self) -> Result<()> {
		self.visible = true;

		Ok(())
	}
}

impl ConfirmPopup {
	///
	pub fn new(env: &Environment) -> Self {
		Self {
			repo: env.repo.clone(),
			target: None,
			snapshot: None,
			visible: false,
			queue: env.queue.clone(),
			theme: env.theme.clone(),
			key_config: env.key_config.clone(),
		}
	}
	///
	pub fn open(&mut self, a: Action) -> Result<()> {
		let snapshot = if a.guards_repo() {
			Some(sync::repo_snapshot(&self.repo.borrow())?)
		} else {
			None
		};
		self.snapshot = snapshot;
		self.target = Some(a);
		self.show()?;

		Ok(())
	}
	///
	pub fn confirm(&mut self) {
		let action = self.target.take();
		let snapshot = self.snapshot.take();
		self.hide();

		if let Some(a) = action {
			if let Some(expected) = snapshot {
				match sync::repo_snapshot(&self.repo.borrow()) {
					Ok(current) if current == expected => {}
					Ok(_) => {
						self.queue.push(InternalEvent::ShowErrorMsg(
							strings::confirm_err_repo_changed(),
						));
						return;
					}
					Err(e) => {
						self.queue.push(InternalEvent::ShowErrorMsg(
							format!(
								"failed to verify repository state:\n{e}"
							),
						));
						return;
					}
				}
			}
			self.queue.push(InternalEvent::ConfirmedAction(a));
		}
	}

	#[cfg(test)]
	fn get_text(&self) -> (String, String) {
		self.get_text_with_limit(38)
	}

	fn get_text_with_limit(
		&self,
		max_name_width: usize,
	) -> (String, String) {
		let kc = &self.key_config;
		let trim = |s: &str| {
			strings::ellipsis_trim_middle(s, max_name_width)
				.into_owned()
		};
		let Some(ref a) = self.target else {
			return (String::new(), String::new());
		};
		match a {
			Action::Reset(item) => (
				strings::confirm_title_reset(),
				strings::confirm_msg_reset(&trim(&item.path)),
			),
			Action::StashDrop(ids) => (
				strings::confirm_title_stashdrop(kc, ids.len() > 1),
				strings::confirm_msg_stashdrop(kc, ids),
			),
			Action::StashPop(_) => (
				strings::confirm_title_stashpop(kc),
				strings::confirm_msg_stashpop(kc),
			),
			Action::ResetHunk(_, _) => (
				strings::confirm_title_reset(),
				strings::confirm_msg_resethunk(kc),
			),
			Action::ResetLines(_, lines) => (
				strings::confirm_title_reset(),
				strings::confirm_msg_reset_lines(lines.len()),
			),
			Action::DeleteLocalBranch(_)
			| Action::DeleteRemoteBranch(_)
			| Action::DeleteRemote(_)
			| Action::DeleteTag(_)
			| Action::DeleteRemoteTag(_, _) => {
				self.delete_confirm_text(a, max_name_width)
			}
			Action::ForcePush(branch, _force) => (
				strings::confirm_title_force_push(kc),
				strings::confirm_msg_force_push(
					kc,
					&trim(
						branch
							.strip_prefix("refs/heads/")
							.unwrap_or(branch),
					),
				),
			),
			Action::PullMerge { incoming, rebase } => (
				strings::confirm_title_merge(kc, *rebase),
				strings::confirm_msg_merge(kc, *incoming, *rebase),
			),
			Action::AbortMerge => (
				strings::confirm_title_abortmerge(),
				strings::confirm_msg_revertchanges(),
			),
			Action::AbortRebase => (
				strings::confirm_title_abortrebase(),
				strings::confirm_msg_abortrebase(),
			),
			Action::AbortRevert => (
				strings::confirm_title_abortrevert(),
				strings::confirm_msg_revertchanges(),
			),
			Action::UndoCommit => (
				strings::confirm_title_undo_commit(),
				strings::confirm_msg_undo_commit(),
			),
			Action::RevertCommit { id, subject } => self
				.commit_confirm_text(
					*id,
					subject,
					false,
					max_name_width,
				),
			Action::CheckoutCommit { id, subject } => self
				.commit_confirm_text(
					*id,
					subject,
					true,
					max_name_width,
				),
		}
	}

	fn delete_confirm_text(
		&self,
		action: &Action,
		max_name_width: usize,
	) -> (String, String) {
		let kc = &self.key_config;
		let trim = |s: &str| {
			strings::ellipsis_trim_middle(s, max_name_width)
				.into_owned()
		};
		match action {
			Action::DeleteLocalBranch(branch_ref) => (
				strings::confirm_title_delete_branch(kc),
				strings::confirm_msg_delete_branch(
					kc,
					&trim(branch_ref),
				),
			),
			Action::DeleteRemoteBranch(branch_ref) => (
				strings::confirm_title_delete_remote_branch(kc),
				strings::confirm_msg_delete_remote_branch(
					kc,
					&trim(branch_ref),
				),
			),
			Action::DeleteRemote(remote_name) => (
				strings::confirm_title_delete_remote(kc),
				strings::confirm_msg_delete_remote(
					kc,
					&trim(remote_name),
				),
			),
			Action::DeleteTag(tag_name) => (
				strings::confirm_title_delete_tag(kc),
				strings::confirm_msg_delete_tag(kc, &trim(tag_name)),
			),
			Action::DeleteRemoteTag(_tag_name, remote) => (
				strings::confirm_title_delete_tag_remote(),
				strings::confirm_msg_delete_tag_remote(&trim(remote)),
			),
			_ => (String::new(), String::new()),
		}
	}

	fn commit_confirm_text(
		&self,
		id: asyncgit::sync::CommitId,
		subject: &str,
		is_checkout: bool,
		max_name_width: usize,
	) -> (String, String) {
		let (head_label, detached) =
			self.snapshot.as_ref().map_or_else(
				|| (String::from("HEAD"), true),
				|s| (s.head_label(), s.detached),
			);
		let short_id = id.get_short_string();
		if is_checkout {
			(
				strings::confirm_title_checkout_commit(),
				strings::confirm_msg_checkout_commit(
					&short_id,
					subject,
					&head_label,
					detached,
					max_name_width,
				),
			)
		} else {
			(
				strings::confirm_title_revert_commit(),
				strings::confirm_msg_revert_commit(
					&short_id,
					subject,
					&head_label,
					detached,
					max_name_width,
				),
			)
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn popup_with(action: Action) -> ConfirmPopup {
		ConfirmPopup {
			repo: std::cell::RefCell::new(sync::RepoPath::Path(
				std::path::PathBuf::default(),
			)),
			target: Some(action),
			snapshot: None,
			visible: true,
			queue: Queue::new(),
			theme: SharedTheme::default(),
			key_config: SharedKeyConfig::default(),
		}
	}

	#[test]
	fn t7l_force_push_full_branch() {
		let key_config = SharedKeyConfig::default();

		let (_, msg_with_prefix) = popup_with(Action::ForcePush(
			"refs/heads/feat/login".to_string(),
			true,
		))
		.get_text();
		assert_eq!(
			msg_with_prefix,
			strings::confirm_msg_force_push(
				&key_config,
				"feat/login",
			)
		);
		assert!(!msg_with_prefix.contains("refs/heads/"));

		let (_, msg_without_prefix) = popup_with(Action::ForcePush(
			"feat/login".to_string(),
			true,
		))
		.get_text();
		assert_eq!(
			msg_without_prefix,
			strings::confirm_msg_force_push(
				&key_config,
				"feat/login",
			)
		);

		let (_, msg_simple) = popup_with(Action::ForcePush(
			"refs/heads/main".to_string(),
			true,
		))
		.get_text();
		assert_eq!(
			msg_simple,
			strings::confirm_msg_force_push(&key_config, "main")
		);
	}
}
