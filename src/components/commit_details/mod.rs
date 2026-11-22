mod compare_details;
mod details;
mod style;

use super::{
	command_pump, event_pump, CommandBlocking, CommandInfo,
	Component, DrawableComponent, EventState, StatusTreeComponent,
};
use crate::{
	accessors,
	app::Environment,
	keys::{key_match, SharedKeyConfig},
	strings,
};
use anyhow::Result;
use asyncgit::{
	sync::{commit_files::OldNew, CommitTags},
	AsyncCommitFiles, CommitFilesParams,
};
use compare_details::CompareDetailsComponent;
use crossterm::event::Event;
use details::DetailsComponent;
use ratatui::{
	layout::{Constraint, Direction, Layout, Rect},
	Frame,
};

pub struct CommitDetailsComponent {
	commit: Option<CommitFilesParams>,
	single_details: DetailsComponent,
	compare_details: CompareDetailsComponent,
	file_tree: StatusTreeComponent,
	git_commit_files: AsyncCommitFiles,
	visible: bool,
	key_config: SharedKeyConfig,
}

impl CommitDetailsComponent {
	accessors!(self, [single_details, compare_details, file_tree]);

	///
	pub fn new(env: &Environment) -> Self {
		Self {
			single_details: DetailsComponent::new(env, false),
			compare_details: CompareDetailsComponent::new(env, false),
			git_commit_files: AsyncCommitFiles::new(
				env.repo.borrow().clone(),
				&env.sender_git,
			),
			file_tree: StatusTreeComponent::new(env, "", false),
			visible: false,
			commit: None,
			key_config: env.key_config.clone(),
		}
	}

	fn get_files_title(&self) -> String {
		let files_count = self.file_tree.file_count();

		format!(
			"{} {}",
			strings::commit::details_files_title(&self.key_config),
			files_count
		)
	}

	///
	pub fn set_commits(
		&mut self,
		params: Option<CommitFilesParams>,
		tags: Option<&CommitTags>,
	) -> Result<()> {
		if params.is_none() {
			self.single_details.set_commit(None, None);
			self.compare_details.set_commits(None);
		}

		self.commit = params;

		if let Some(id) = params {
			self.file_tree.set_commit(Some(id.id));

			if let Some(other) = id.other {
				self.compare_details.set_commits(Some(OldNew {
					new: id.id,
					old: other,
				}));
			} else {
				self.single_details
					.set_commit(Some(id.id), tags.cloned());
			}

			if let Some((fetched_id, res)) =
				self.git_commit_files.current()?
			{
				if fetched_id == id {
					self.file_tree.update(res.as_slice())?;
					self.file_tree.set_title(self.get_files_title());

					return Ok(());
				}
			}

			self.file_tree.clear()?;
			self.git_commit_files.fetch(id)?;
		}

		self.file_tree.set_title(self.get_files_title());

		Ok(())
	}

	///
	pub fn any_work_pending(&self) -> bool {
		self.git_commit_files.is_pending()
	}

	///
	pub const fn files(&self) -> &StatusTreeComponent {
		&self.file_tree
	}

	fn details_focused(&self) -> bool {
		self.single_details.focused()
			|| self.compare_details.focused()
	}

	fn set_details_focus(&mut self, focus: bool) {
		if self.is_compare() {
			self.compare_details.focus(focus);
		} else {
			self.single_details.focus(focus);
		}
	}

	fn is_compare(&self) -> bool {
		self.commit.is_some_and(|p| p.other.is_some())
	}
}

const BORDER_HEIGHT: u16 = 2;
const MIN_PANE_HEIGHT: u16 = 1 + BORDER_HEIGHT;

fn split_single_heights(
	total: u16,
	info_rows: u16,
	details_focused: bool,
) -> (u16, u16) {
	let min_both = MIN_PANE_HEIGHT.saturating_mul(2);
	if total < min_both {
		let files = total / 2;
		return (total.saturating_sub(files), files);
	}

	let info_ideal = info_rows.saturating_add(BORDER_HEIGHT);
	let details_min_full = info_ideal.saturating_add(MIN_PANE_HEIGHT);
	let max_details = total.saturating_sub(MIN_PANE_HEIGHT);

	if details_min_full >= max_details {
		return (max_details, MIN_PANE_HEIGHT);
	}

	let pct: u32 = if details_focused { 60 } else { 40 };
	let desired_details =
		u16::try_from((u32::from(total) * pct + 50) / 100)
			.unwrap_or(total);
	let details =
		desired_details.clamp(details_min_full, max_details);
	let files = total.saturating_sub(details);

	(details, files)
}

const fn split_compare_heights(total: u16) -> (u16, u16) {
	if total < MIN_PANE_HEIGHT {
		return (0, total);
	}

	let max_compare = total.saturating_sub(MIN_PANE_HEIGHT);
	let compare =
		if CompareDetailsComponent::IDEAL_HEIGHT < max_compare {
			CompareDetailsComponent::IDEAL_HEIGHT
		} else {
			max_compare
		};
	let files = total.saturating_sub(compare);

	(compare, files)
}

impl DrawableComponent for CommitDetailsComponent {
	fn draw(&self, f: &mut Frame, rect: Rect) -> Result<()> {
		if !self.visible {
			return Ok(());
		}

		let (details_height, files_height) = if self.is_compare() {
			split_compare_heights(rect.height)
		} else {
			let details_focused =
				!self.file_tree.focused() && self.details_focused();
			split_single_heights(
				rect.height,
				self.single_details.info_line_count(),
				details_focused,
			)
		};

		let constraints = [
			Constraint::Length(details_height),
			Constraint::Length(files_height),
		];

		let chunks = Layout::default()
			.direction(Direction::Vertical)
			.constraints(constraints.as_ref())
			.split(rect);

		if self.is_compare() {
			self.compare_details.draw(f, chunks[0])?;
		} else {
			self.single_details.draw(f, chunks[0])?;
		}
		self.file_tree.draw(f, chunks[1])?;

		Ok(())
	}
}

impl Component for CommitDetailsComponent {
	fn commands(
		&self,
		out: &mut Vec<CommandInfo>,
		force_all: bool,
	) -> CommandBlocking {
		if self.visible || force_all {
			command_pump(
				out,
				force_all,
				self.components().as_slice(),
			);
		}

		CommandBlocking::PassingOn
	}

	fn event(&mut self, ev: &Event) -> Result<EventState> {
		if event_pump(ev, self.components_mut().as_mut_slice())?
			.is_consumed()
		{
			if !self.file_tree.is_visible() {
				self.hide();
			}

			return Ok(EventState::Consumed);
		}

		if self.focused() {
			if let Event::Key(e) = ev {
				return if key_match(e, self.key_config.keys.move_down)
					&& self.details_focused()
				{
					self.set_details_focus(false);
					self.file_tree.focus(true);
					Ok(EventState::Consumed)
				} else if key_match(e, self.key_config.keys.move_up)
					&& self.file_tree.focused()
					&& !self.is_compare()
				{
					self.file_tree.focus(false);
					self.set_details_focus(true);
					Ok(EventState::Consumed)
				} else {
					Ok(EventState::NotConsumed)
				};
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
		self.file_tree.show()?;
		Ok(())
	}

	fn focused(&self) -> bool {
		self.details_focused() || self.file_tree.focused()
	}

	fn focus(&mut self, focus: bool) {
		self.single_details.focus(false);
		self.compare_details.focus(false);
		self.file_tree.focus(focus);
		self.file_tree.show_selection(true);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use asyncgit::{
		sync::{
			CommitDetails, CommitId, CommitMessage, CommitSignature,
			Tag,
		},
		StatusItem, StatusItemType,
	};
	use ratatui::{backend::TestBackend, Terminal};

	fn sample_commit(
		with_committer: bool,
		subject: &str,
		hash: &str,
	) -> CommitDetails {
		CommitDetails {
			author: CommitSignature {
				name: String::from("testuser"),
				email: String::from("testuser@example.com"),
				time: 1_700_000_000,
			},
			committer: with_committer.then(|| CommitSignature {
				name: String::from("committer"),
				email: String::from("committer@example.com"),
				time: 1_700_000_100,
			}),
			message: Some(CommitMessage {
				subject: String::from(subject),
				body: Some(String::from(
					"body line one\nbody line two",
				)),
			}),
			hash: String::from(hash),
		}
	}

	fn make_single_component(
		with_committer: bool,
		with_tags: bool,
	) -> Result<CommitDetailsComponent> {
		let env = Environment::test_env();
		let mut comp = CommitDetailsComponent::new(&env);
		comp.show()?;
		comp.commit =
			Some(CommitFilesParams::from(CommitId::default()));
		let tags = if with_tags {
			vec![Tag::new("v1.0.0")]
		} else {
			vec![]
		};
		comp.single_details.set_test_data(
			Some(sample_commit(
				with_committer,
				"subject_marker",
				"0123456789abcdef0123456789abcdef01234567",
			)),
			tags,
		);
		comp.file_tree.update(&[StatusItem {
			path: String::from("changed_file.rs"),
			status: StatusItemType::Modified,
		}])?;
		comp.file_tree.set_title(comp.get_files_title());
		Ok(comp)
	}

	fn make_compare_component() -> Result<CommitDetailsComponent> {
		let env = Environment::test_env();
		let mut comp = CommitDetailsComponent::new(&env);
		comp.show()?;
		let id = CommitId::default();
		comp.commit = Some(CommitFilesParams {
			id,
			other: Some(id),
		});
		comp.compare_details.set_test_data(Some(OldNew {
			old: sample_commit(
				false,
				"old_subject",
				"1111111000000000000000000000000000000000",
			),
			new: sample_commit(
				false,
				"new_subject",
				"2222222000000000000000000000000000000000",
			),
		}));
		comp.file_tree.update(&[StatusItem {
			path: String::from("changed_file.rs"),
			status: StatusItemType::Modified,
		}])?;
		comp.file_tree.set_title(comp.get_files_title());
		Ok(comp)
	}

	fn render_lines(
		comp: &CommitDetailsComponent,
		width: u16,
		height: u16,
	) -> Result<Vec<String>> {
		let backend = TestBackend::new(width, height);
		let mut terminal = Terminal::new(backend)?;
		terminal.draw(|f| {
			let _ = comp.draw(f, Rect::new(0, 0, width, height));
		})?;
		let buf = terminal.backend().buffer();
		let mut lines = Vec::with_capacity(usize::from(height));
		for y in 0..height {
			let mut row = String::with_capacity(usize::from(width));
			for x in 0..width {
				row.push_str(buf[(x, y)].symbol());
			}
			lines.push(row);
		}
		Ok(lines)
	}

	fn find_row(lines: &[String], needle: &str) -> Option<usize> {
		lines.iter().position(|l| l.contains(needle))
	}

	#[test]
	fn t8m_details_fit_80x24() -> Result<()> {
		let comp = make_single_component(true, true)?;
		// 80x24 terminal:
		// - Log without search bar: top(2) + cmdbar(1) -> height 21, width 40% = 32
		// - Log with search bar: 21 - search(3) -> height 18, width 32
		// - Inspect popup: 24 - cmdbar(1) -> height 23, width 50% = 40
		for (label, width, height) in [
			("log_no_search", 32, 21),
			("log_with_search", 32, 18),
			("inspect", 40, 23),
		] {
			let lines = render_lines(&comp, width, height)?;
			let screen = lines.join("\n");
			assert!(
				screen.contains("testuser"),
				"{label}: missing author\n{screen}"
			);
			assert!(
				screen.contains("committer"),
				"{label}: missing committer\n{screen}"
			);
			assert!(
				screen.contains("0123456"),
				"{label}: missing sha\n{screen}"
			);
			assert!(
				screen.contains("v1.0.0"),
				"{label}: missing tags\n{screen}"
			);
			assert!(
				screen.contains("subject_marker"),
				"{label}: missing message row\n{screen}"
			);
			assert!(
				screen.contains("changed_file.rs"),
				"{label}: missing file row\n{screen}"
			);
		}
		Ok(())
	}

	#[test]
	fn t8m_large_split_kept() -> Result<()> {
		let mut comp = make_single_component(true, true)?;

		// Room to spare (height 50): 40/60 when files focused
		comp.focus(true);
		let lines = render_lines(&comp, 40, 50)?;
		let files_y = find_row(&lines, "Files")
			.ok_or_else(|| anyhow::anyhow!("missing Files title"))?;
		assert_eq!(files_y, 20, "expected 40/60 split at height 50");

		// Room to spare (height 50): 60/40 when details focused
		comp.file_tree.focus(false);
		comp.set_details_focus(true);
		let lines = render_lines(&comp, 40, 50)?;
		let files_y = find_row(&lines, "Files")
			.ok_or_else(|| anyhow::anyhow!("missing Files title"))?;
		assert_eq!(files_y, 30, "expected 60/40 split at height 50");

		// Minimums win over 40/60 when 40% is too small (height 20: 40% = 8 < 12)
		comp.focus(true);
		let lines = render_lines(&comp, 40, 20)?;
		let files_y = find_row(&lines, "Files")
			.ok_or_else(|| anyhow::anyhow!("missing Files title"))?;
		assert_eq!(
			files_y, 12,
			"expected minimums (9 info + 3 msg = 12) to win over 40%"
		);
		Ok(())
	}

	#[test]
	fn t8m_info_gives_way() -> Result<()> {
		let mut comp = make_single_component(true, true)?;
		for details_focused in [false, true] {
			if details_focused {
				comp.file_tree.focus(false);
				comp.set_details_focus(true);
			} else {
				comp.focus(true);
			}

			// Height 10 (< 15 needed for full info + 1 msg + 1 file):
			// info clips from bottom first, message and files keep 1 row each.
			// Focusing details scrolls to the bottom message line.
			let expected_msg = if details_focused {
				"body line two"
			} else {
				"subject_marker"
			};
			let lines = render_lines(&comp, 40, 10)?;
			let screen = lines.join("\n");
			assert!(
				screen.contains("testuser"),
				"focused={details_focused} h=10: top info line should remain\n{screen}"
			);
			assert!(
				!screen.contains("v1.0.0"),
				"focused={details_focused} h=10: bottom info line should clip first\n{screen}"
			);
			assert!(
				screen.contains(expected_msg),
				"focused={details_focused} h=10: message must keep 1 row\n{screen}"
			);
			assert!(
				screen.contains("changed_file.rs"),
				"focused={details_focused} h=10: files must keep 1 row\n{screen}"
			);

			// Height 6 (exact room for 1 message row + 1 file row, 0 info rows):
			let lines = render_lines(&comp, 40, 6)?;
			let screen = lines.join("\n");
			assert!(
				screen.contains(expected_msg),
				"focused={details_focused} h=6: message must keep 1 row\n{screen}"
			);
			assert!(
				screen.contains("changed_file.rs"),
				"focused={details_focused} h=6: files must keep 1 row\n{screen}"
			);
		}
		Ok(())
	}

	#[test]
	fn t8m_info_height_follows_content() -> Result<()> {
		for (with_committer, with_tags, expected_info_height) in [
			(false, false, 5), // 3 rows + 2 borders
			(true, false, 7),  // 5 rows + 2 borders
			(false, true, 7),  // 5 rows + 2 borders
			(true, true, 9),   // 7 rows + 2 borders
		] {
			let comp =
				make_single_component(with_committer, with_tags)?;
			let lines = render_lines(&comp, 40, 40)?;
			let msg_y =
				find_row(&lines, "Message").ok_or_else(|| {
					anyhow::anyhow!("missing Message title")
				})?;
			assert_eq!(
				msg_y, expected_info_height,
				"committer={with_committer}, tags={with_tags}: unexpected info height"
			);
		}
		Ok(())
	}

	#[test]
	fn t8m_compare_fits() -> Result<()> {
		let comp = make_compare_component()?;
		for height in [13, 24] {
			let lines = render_lines(&comp, 40, height)?;
			let screen = lines.join("\n");
			assert!(
				screen.contains("old_subject"),
				"h={height}: missing old commit subject\n{screen}"
			);
			assert!(
				screen.contains("new_subject"),
				"h={height}: missing new commit subject\n{screen}"
			);
			assert!(
				screen.contains("changed_file.rs"),
				"h={height}: missing file row\n{screen}"
			);
		}
		Ok(())
	}

	#[test]
	fn t8m_compare_info_gives_way() -> Result<()> {
		let comp = make_compare_component()?;
		for (height, expected_old_h, expected_new_h) in
			[(12, 4, 5), (11, 4, 4), (9, 3, 3)]
		{
			let lines = render_lines(&comp, 40, height)?;
			let screen = lines.join("\n");
			assert!(
				screen.contains("changed_file.rs"),
				"h={height}: files must keep 1 row\n{screen}"
			);
			let old_y =
				find_row(&lines, "1111111").ok_or_else(|| {
					anyhow::anyhow!("h={height}: missing old title")
				})?;
			let new_y =
				find_row(&lines, "2222222").ok_or_else(|| {
					anyhow::anyhow!("h={height}: missing new title")
				})?;
			let files_y =
				find_row(&lines, "Files").ok_or_else(|| {
					anyhow::anyhow!("h={height}: missing Files title")
				})?;
			let old_h = new_y - old_y;
			let new_h = files_y - new_y;
			assert!(
				old_h.abs_diff(new_h) <= 1,
				"h={height}: compare infos should shrink evenly (old={old_h}, new={new_h})"
			);
			assert_eq!(
				old_h + new_h,
				expected_old_h + expected_new_h,
				"h={height}: compare total height"
			);
			assert_eq!(
				usize::from(height) - files_y,
				3,
				"h={height}: files should keep 3 rows (1 content + 2 borders)"
			);
		}
		Ok(())
	}

	#[test]
	fn t8m_tiny_no_panic() -> Result<()> {
		let mut single = make_single_component(true, true)?;
		let compare = make_compare_component()?;
		let mut empty =
			CommitDetailsComponent::new(&Environment::test_env());
		empty.show()?;

		for height in 0..=6 {
			for width in [0, 1, 2, 4, 40] {
				single.focus(false);
				let _ = render_lines(&single, width, height)?;
				single.focus(true);
				let _ = render_lines(&single, width, height)?;
				single.file_tree.focus(false);
				single.set_details_focus(true);
				let _ = render_lines(&single, width, height)?;

				let _ = render_lines(&compare, width, height)?;
				let _ = render_lines(&empty, width, height)?;
			}
		}
		Ok(())
	}
}
