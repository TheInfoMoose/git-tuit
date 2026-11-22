use std::path::{Path, PathBuf};

use crate::{
	app::Environment,
	components::{
		visibility_blocking, CommandBlocking, CommandInfo, Component,
		DrawableComponent, EventState, RevisionFilesComponent,
	},
	queue::{InternalEvent, Queue},
	AsyncNotification,
};
use anyhow::Result;
use asyncgit::sync::{self, RepoPathRef};

fn is_unborn_head_error(err: &asyncgit::Error) -> bool {
	match err {
		asyncgit::Error::NoHead => true,
		asyncgit::Error::Git(e) => {
			format!("{:?}", e.code()) == "UnbornBranch"
		}
		_ => false,
	}
}

pub struct FilesTab {
	repo: RepoPathRef,
	queue: Queue,
	visible: bool,
	head_error_shown: bool,
	files: RevisionFilesComponent,
}

impl FilesTab {
	///
	pub fn new(
		env: &Environment,
		select_file: Option<PathBuf>,
	) -> Self {
		Self {
			visible: false,
			head_error_shown: false,
			files: RevisionFilesComponent::new(env, select_file),
			repo: env.repo.clone(),
			queue: env.queue.clone(),
		}
	}

	///
	pub fn update(&mut self) {
		if self.is_visible() {
			let head = sync::get_head(&self.repo.borrow());
			match head {
				Ok(head) => {
					if let Err(err) = self.files.set_commit(head) {
						self.files.clear_revision();
						self.report_head_error(&err);
					} else {
						self.head_error_shown = false;
					}
				}
				Err(err) if is_unborn_head_error(&err) => {
					self.head_error_shown = false;
					self.files.clear_revision();
				}
				Err(err) => {
					self.files.clear_revision();
					self.report_head_error(&err);
				}
			}
		}
	}

	fn report_head_error(&mut self, err: &impl std::fmt::Display) {
		if !self.head_error_shown {
			self.head_error_shown = true;
			self.queue.push(InternalEvent::ShowErrorMsg(format!(
				"failed to read HEAD:\n{err}"
			)));
		}
	}

	///
	pub fn anything_pending(&self) -> bool {
		self.files.any_work_pending()
	}

	///
	pub fn update_async(
		&mut self,
		ev: AsyncNotification,
	) -> Result<()> {
		// async results must land even while the tab is hidden
		self.files.update(ev)?;

		Ok(())
	}

	pub fn file_finder_update(&mut self, file: &Path) {
		self.files.find_file(file);
	}
}

impl DrawableComponent for FilesTab {
	fn draw(
		&self,
		f: &mut ratatui::Frame,
		rect: ratatui::layout::Rect,
	) -> Result<()> {
		if self.is_visible() {
			self.files.draw(f, rect)?;
		}
		Ok(())
	}
}

impl Component for FilesTab {
	fn commands(
		&self,
		out: &mut Vec<CommandInfo>,
		force_all: bool,
	) -> CommandBlocking {
		if self.visible || force_all {
			return self.files.commands(out, force_all);
		}

		visibility_blocking(self)
	}

	fn event(
		&mut self,
		ev: &crossterm::event::Event,
	) -> Result<EventState> {
		if self.visible {
			let res = self.files.event(ev)?;
			// the tree hides itself to open blame/history popups; the tab keeps it shown
			if !self.files.is_visible() {
				self.files.show()?;
			}
			return Ok(res);
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
		self.update();
		Ok(())
	}
}
