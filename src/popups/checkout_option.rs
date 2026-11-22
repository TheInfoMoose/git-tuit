use crate::components::{
	visibility_blocking, CommandBlocking, CommandInfo, Component,
	DrawableComponent, EventState,
};
use crate::queue::{InternalEvent, NeedsUpdate};
use crate::strings::CheckoutOptions;
use crate::try_or_popup;
use crate::{
	app::Environment,
	keys::{key_match, SharedKeyConfig},
	queue::Queue,
	strings,
	ui::{self, style::SharedTheme},
};
use anyhow::{Ok, Result};
use asyncgit::sync::branch::checkout_remote_branch;
use asyncgit::sync::{
	checkout_branch, checkout_branch_discard, BranchInfo, RepoPath,
};
use crossterm::event::Event;
use ratatui::{
	layout::{Alignment, Rect},
	text::{Line, Span},
	widgets::{Block, Borders, Clear, Paragraph},
	Frame,
};

pub struct CheckoutOptionPopup {
	queue: Queue,
	repo: RepoPath,
	branch: Option<BranchInfo>,
	option: CheckoutOptions,
	visible: bool,
	key_config: SharedKeyConfig,
	theme: SharedTheme,
}

impl CheckoutOptionPopup {
	///
	pub fn new(env: &Environment) -> Self {
		Self {
			queue: env.queue.clone(),
			repo: env.repo.borrow().clone(),
			branch: None,
			option: CheckoutOptions::KeepLocalChanges,
			visible: false,
			key_config: env.key_config.clone(),
			theme: env.theme.clone(),
		}
	}

	fn get_text(&self, _width: u16) -> Vec<Line<'_>> {
		let mut txt: Vec<Line> = Vec::with_capacity(10);

		txt.push(Line::from(vec![
			Span::styled(
				String::from("Switch to: "),
				self.theme.text(true, false),
			),
			Span::styled(
				self.branch.as_ref().expect("No branch").name.clone(),
				self.theme.commit_hash(false),
			),
		]));

		let (kind_name, kind_desc) = self.option.to_string_pair();

		txt.push(Line::from(vec![
			Span::styled(
				String::from("How: "),
				self.theme.text(true, false),
			),
			Span::styled(kind_name, self.theme.text(true, true)),
			Span::styled(kind_desc, self.theme.text(true, false)),
		]));

		txt
	}

	///
	pub fn open(&mut self, branch: BranchInfo) -> Result<()> {
		self.option = CheckoutOptions::KeepLocalChanges;
		self.show()?;

		self.branch = Some(branch);

		Ok(())
	}

	fn checkout(&self) -> Result<()> {
		if let Some(branch) = &self.branch {
			if branch.is_local() {
				checkout_branch(&self.repo, &branch.name)?;
			} else {
				checkout_remote_branch(&self.repo, branch)?;
			}
		}

		Ok(())
	}

	fn handle_event(&mut self) -> Result<()> {
		match self.option {
			CheckoutOptions::KeepLocalChanges => {
				self.checkout()?;
			}
			CheckoutOptions::DiscardAllLocalChagnes => {
				if let Some(branch) = &self.branch {
					checkout_branch_discard(&self.repo, branch)?;
				}
			}
		}

		self.queue.push(InternalEvent::Update(NeedsUpdate::ALL));
		self.queue.push(InternalEvent::SelectBranch);
		self.hide();

		Ok(())
	}

	const fn change_kind(&mut self, incr: bool) {
		self.option = if incr {
			self.option.next()
		} else {
			self.option.previous()
		};
	}
}

impl DrawableComponent for CheckoutOptionPopup {
	fn draw(&self, f: &mut Frame, area: Rect) -> Result<()> {
		if self.is_visible() {
			const SIZE: (u16, u16) = (55, 4);
			let area =
				ui::centered_rect_absolute(SIZE.0, SIZE.1, area);

			let width = area.width;

			f.render_widget(Clear, area);
			f.render_widget(
				Paragraph::new(self.get_text(width))
					.block(
						Block::default()
							.borders(Borders::ALL)
							.title(Span::styled(
								"Checkout options",
								self.theme.title(true),
							))
							.border_style(self.theme.block(true)),
					)
					.alignment(Alignment::Left),
				area,
			);
		}

		Ok(())
	}
}

impl Component for CheckoutOptionPopup {
	fn commands(
		&self,
		out: &mut Vec<CommandInfo>,
		force_all: bool,
	) -> CommandBlocking {
		if self.is_visible() || force_all {
			out.push(
				CommandInfo::new(
					strings::commands::close_popup(&self.key_config),
					true,
					true,
				)
				.order(1),
			);

			out.push(
				CommandInfo::new(
					strings::commands::reset_commit(&self.key_config),
					true,
					true,
				)
				.order(1),
			);

			out.push(
				CommandInfo::new(
					strings::commands::reset_type(&self.key_config),
					true,
					true,
				)
				.order(1),
			);
		}

		visibility_blocking(self)
	}

	fn event(
		&mut self,
		event: &crossterm::event::Event,
	) -> Result<EventState> {
		if self.is_visible() {
			if let Event::Key(key) = &event {
				if key_match(key, self.key_config.keys.exit_popup) {
					self.hide();
				} else if key_match(
					key,
					self.key_config.keys.move_down,
				) {
					self.change_kind(true);
				} else if key_match(key, self.key_config.keys.move_up)
				{
					self.change_kind(false);
				} else if key_match(key, self.key_config.keys.enter) {
					try_or_popup!(
						self,
						"checkout error:",
						self.handle_event()
					);
				}
			}

			return Ok(EventState::Consumed);
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

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use asyncgit::sync::{
		commit, create_branch, delete_branch, get_branches_info,
		get_head, stage_add_file,
		status::{get_status, StatusType},
	};
	use std::{fs, path::Path, process::Command};
	use tempfile::TempDir;

	fn popup_for(
		repo: &RepoPath,
		branch: BranchInfo,
	) -> CheckoutOptionPopup {
		let mut popup = CheckoutOptionPopup {
			queue: Queue::new(),
			repo: repo.clone(),
			branch: None,
			option: CheckoutOptions::KeepLocalChanges,
			visible: false,
			key_config: SharedKeyConfig::default(),
			theme: SharedTheme::default(),
		};
		popup.open(branch).unwrap();
		popup.option = CheckoutOptions::DiscardAllLocalChagnes;
		popup
	}

	fn write_and_commit(
		repo_path: &RepoPath,
		root: &Path,
		rel: &str,
		content: &str,
		msg: &str,
	) {
		let full = root.join(rel);
		if let Some(parent) = full.parent() {
			fs::create_dir_all(parent).unwrap();
		}
		fs::write(&full, content).unwrap();
		stage_add_file(repo_path, Path::new(rel)).unwrap();
		commit(repo_path, msg).unwrap();
	}

	fn branch_info(
		repo_path: &RepoPath,
		local: bool,
		name: &str,
	) -> BranchInfo {
		get_branches_info(repo_path, local)
			.unwrap()
			.into_iter()
			.find(|b| b.name == name)
			.unwrap()
	}

	fn local_branch_names(repo_path: &RepoPath) -> Vec<String> {
		get_branches_info(repo_path, true)
			.unwrap()
			.into_iter()
			.map(|b| b.name)
			.collect()
	}

	fn head_branch_name(repo_path: &RepoPath) -> String {
		let head = get_head(repo_path).unwrap();
		get_branches_info(repo_path, true)
			.unwrap()
			.into_iter()
			.find(|b| {
				b.local_details().is_some_and(|d| d.is_head)
					&& b.top_commit == head
			})
			.unwrap()
			.name
	}

	fn setup_remote_clone() -> (TempDir, TempDir, RepoPath) {
		let (bare_dir, _bare) = git2_testing::repo_init_bare();
		let bare_str = bare_dir.path().to_str().unwrap();

		let clone1_dir = TempDir::new().unwrap();
		let status = Command::new("git")
			.args([
				"clone",
				bare_str,
				clone1_dir.path().to_str().unwrap(),
			])
			.status()
			.unwrap();
		assert!(status.success());

		let c1_root = clone1_dir.path();
		let c1_path: RepoPath = c1_root.to_str().unwrap().into();
		for (k, v) in [
			("user.name", "testuser"),
			("user.email", "testuser@example.com"),
		] {
			let s = Command::new("git")
				.current_dir(c1_root)
				.args(["config", k, v])
				.status()
				.unwrap();
			assert!(s.success());
		}

		write_and_commit(&c1_path, c1_root, "test.txt", "v1", "c1");
		let s = Command::new("git")
			.current_dir(c1_root)
			.args(["push", "origin", "HEAD:refs/heads/master"])
			.status()
			.unwrap();
		assert!(s.success());

		create_branch(&c1_path, "feat").unwrap();
		write_and_commit(&c1_path, c1_root, "test.txt", "v2", "c2");
		let s = Command::new("git")
			.current_dir(c1_root)
			.args(["push", "origin", "HEAD:refs/heads/feat"])
			.status()
			.unwrap();
		assert!(s.success());

		let clone2_dir = TempDir::new().unwrap();
		let status = Command::new("git")
			.args([
				"clone",
				bare_str,
				clone2_dir.path().to_str().unwrap(),
			])
			.status()
			.unwrap();
		assert!(status.success());

		let c2_root = clone2_dir.path();
		for (k, v) in [
			("user.name", "testuser"),
			("user.email", "testuser@example.com"),
		] {
			let s = Command::new("git")
				.current_dir(c2_root)
				.args(["config", k, v])
				.status()
				.unwrap();
			assert!(s.success());
		}

		let c2_path: RepoPath = c2_root.to_str().unwrap().into();
		(bare_dir, clone2_dir, c2_path)
	}

	#[test]
	fn t8b_discard_switch_succeeds() {
		let (td, _repo) = git2_testing::repo_init();
		let root = td.path();
		let repo_path: RepoPath = root.to_str().unwrap().into();

		write_and_commit(&repo_path, root, "a.txt", "base", "c1");
		create_branch(&repo_path, "other").unwrap();
		write_and_commit(&repo_path, root, "a.txt", "other", "c2");
		write_and_commit(&repo_path, root, "b.txt", "b_other", "c3");
		checkout_branch(&repo_path, "master").unwrap();

		// Unstaged edit + staged new file
		fs::write(root.join("a.txt"), "unstaged").unwrap();
		fs::write(root.join("staged.txt"), "staged").unwrap();
		stage_add_file(&repo_path, Path::new("staged.txt")).unwrap();

		let target = branch_info(&repo_path, true, "other");
		let mut popup = popup_for(&repo_path, target);

		popup.handle_event().unwrap();

		assert!(!popup.is_visible());
		assert_eq!(head_branch_name(&repo_path), "other");
		assert_eq!(
			fs::read_to_string(root.join("a.txt")).unwrap(),
			"other"
		);
		assert_eq!(
			fs::read_to_string(root.join("b.txt")).unwrap(),
			"b_other"
		);
		assert!(!root.join("staged.txt").exists());
		assert_eq!(
			get_status(&repo_path, StatusType::Both, None).unwrap(),
			vec![]
		);
	}

	#[test]
	fn t8b_untracked_collision_changes_nothing() {
		let (td, _repo) = git2_testing::repo_init();
		let root = td.path();
		let repo_path: RepoPath = root.to_str().unwrap().into();

		write_and_commit(&repo_path, root, "a.txt", "base", "c1");
		create_branch(&repo_path, "other").unwrap();
		write_and_commit(
			&repo_path,
			root,
			"collide.txt",
			"from_other",
			"c2",
		);
		checkout_branch(&repo_path, "master").unwrap();

		// Unstaged tracked edit, staged new file, and untracked colliding file
		fs::write(root.join("a.txt"), "local_edit").unwrap();
		fs::write(root.join("staged.txt"), "staged_val").unwrap();
		stage_add_file(&repo_path, Path::new("staged.txt")).unwrap();
		fs::write(root.join("collide.txt"), "precious_untracked")
			.unwrap();

		let branches_before = local_branch_names(&repo_path);
		let target = branch_info(&repo_path, true, "other");
		let mut popup = popup_for(&repo_path, target);

		let err = popup.handle_event().unwrap_err();
		assert!(
			err.to_string().contains("collide.txt"),
			"error should name colliding file: {err}"
		);
		assert!(popup.is_visible());
		assert_eq!(head_branch_name(&repo_path), "master");
		assert_eq!(
			fs::read_to_string(root.join("a.txt")).unwrap(),
			"local_edit"
		);
		assert_eq!(
			fs::read_to_string(root.join("staged.txt")).unwrap(),
			"staged_val"
		);
		assert_eq!(
			fs::read_to_string(root.join("collide.txt")).unwrap(),
			"precious_untracked"
		);
		let staged =
			get_status(&repo_path, StatusType::Stage, None).unwrap();
		assert!(
			staged.iter().any(|s| s.path == "staged.txt"),
			"staged.txt should remain staged"
		);
		assert_eq!(local_branch_names(&repo_path), branches_before);
	}

	#[test]
	fn t8b_untracked_bystander_survives() {
		// 1. Local branch switch with untracked bystander
		{
			let (td, _repo) = git2_testing::repo_init();
			let root = td.path();
			let repo_path: RepoPath = root.to_str().unwrap().into();

			write_and_commit(&repo_path, root, "a.txt", "base", "c1");
			create_branch(&repo_path, "other").unwrap();
			write_and_commit(
				&repo_path, root, "a.txt", "other", "c2",
			);
			checkout_branch(&repo_path, "master").unwrap();

			fs::write(root.join("a.txt"), "dirty").unwrap();
			fs::write(root.join("bystander.txt"), "keep_local")
				.unwrap();

			let target = branch_info(&repo_path, true, "other");
			let mut popup = popup_for(&repo_path, target);
			popup.handle_event().unwrap();

			assert!(!popup.is_visible());
			assert_eq!(head_branch_name(&repo_path), "other");
			assert_eq!(
				fs::read_to_string(root.join("a.txt")).unwrap(),
				"other"
			);
			assert_eq!(
				fs::read_to_string(root.join("bystander.txt"))
					.unwrap(),
				"keep_local"
			);
		}

		// 2. Remote branch switch with untracked bystander
		{
			let (_bare, clone2_dir, c2_path) = setup_remote_clone();
			let c2_root = clone2_dir.path();

			fs::write(c2_root.join("test.txt"), "dirty").unwrap();
			fs::write(c2_root.join("bystander.txt"), "keep_remote")
				.unwrap();

			let target = branch_info(&c2_path, false, "origin/feat");
			let mut popup = popup_for(&c2_path, target);
			popup.handle_event().unwrap();

			assert!(!popup.is_visible());
			assert_eq!(head_branch_name(&c2_path), "feat");
			assert_eq!(
				fs::read_to_string(c2_root.join("test.txt")).unwrap(),
				"v2"
			);
			assert_eq!(
				fs::read_to_string(c2_root.join("bystander.txt"))
					.unwrap(),
				"keep_remote"
			);
		}
	}

	#[test]
	fn t8b_existing_local_name_changes_nothing() {
		let (_bare, clone2_dir, c2_path) = setup_remote_clone();
		let c2_root = clone2_dir.path();

		// Create local branch "feat" so origin/feat collides with local name
		create_branch(&c2_path, "feat").unwrap();
		checkout_branch(&c2_path, "master").unwrap();

		fs::write(c2_root.join("test.txt"), "local_edit").unwrap();
		fs::write(c2_root.join("staged.txt"), "staged_val").unwrap();
		stage_add_file(&c2_path, Path::new("staged.txt")).unwrap();

		let branches_before = local_branch_names(&c2_path);
		let target = branch_info(&c2_path, false, "origin/feat");
		let mut popup = popup_for(&c2_path, target);

		assert!(popup.handle_event().is_err());
		assert!(popup.is_visible());
		assert_eq!(head_branch_name(&c2_path), "master");
		assert_eq!(
			fs::read_to_string(c2_root.join("test.txt")).unwrap(),
			"local_edit"
		);
		assert_eq!(
			fs::read_to_string(c2_root.join("staged.txt")).unwrap(),
			"staged_val"
		);
		let staged =
			get_status(&c2_path, StatusType::Stage, None).unwrap();
		assert!(
			staged.iter().any(|s| s.path == "staged.txt"),
			"staged.txt should remain staged"
		);
		assert_eq!(local_branch_names(&c2_path), branches_before);
	}

	#[test]
	fn t8b_missing_target_changes_nothing() {
		let (td, _repo) = git2_testing::repo_init();
		let root = td.path();
		let repo_path: RepoPath = root.to_str().unwrap().into();

		write_and_commit(&repo_path, root, "a.txt", "base", "c1");
		create_branch(&repo_path, "vanished").unwrap();
		write_and_commit(&repo_path, root, "a.txt", "vanished", "c2");
		checkout_branch(&repo_path, "master").unwrap();

		let target = branch_info(&repo_path, true, "vanished");
		delete_branch(&repo_path, &target.reference).unwrap();

		fs::write(root.join("a.txt"), "local_edit").unwrap();
		fs::write(root.join("staged.txt"), "staged_val").unwrap();
		stage_add_file(&repo_path, Path::new("staged.txt")).unwrap();

		let branches_before = local_branch_names(&repo_path);
		let mut popup = popup_for(&repo_path, target);

		assert!(popup.handle_event().is_err());
		assert!(popup.is_visible());
		assert_eq!(head_branch_name(&repo_path), "master");
		assert_eq!(
			fs::read_to_string(root.join("a.txt")).unwrap(),
			"local_edit"
		);
		assert_eq!(
			fs::read_to_string(root.join("staged.txt")).unwrap(),
			"staged_val"
		);
		let staged =
			get_status(&repo_path, StatusType::Stage, None).unwrap();
		assert!(
			staged.iter().any(|s| s.path == "staged.txt"),
			"staged.txt should remain staged"
		);
		assert_eq!(local_branch_names(&repo_path), branches_before);
	}
}
