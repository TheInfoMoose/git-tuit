use super::{
	CommandBlocking, CommandInfo, Component, DrawableComponent,
	EventState,
};
use crate::{
	app::Environment,
	keys::SharedKeyConfig,
	string_utils::tabs_to_spaces,
	strings,
	ui::{
		self, common_nav, style::SharedTheme, AsyncSyntaxJob,
		ParagraphState, ScrollPos, StatefulParagraph,
	},
	AsyncAppNotification, AsyncNotification, SyntaxHighlightProgress,
};
use anyhow::Result;
use asyncgit::{
	asyncjob::AsyncSingleJob,
	sync::{self, RepoPathRef, TreeFile},
	ProgressPercent,
};
use crossterm::event::Event;
use filetreelist::MoveSelection;
use itertools::Either;
use ratatui::{
	layout::Rect,
	text::Text,
	widgets::{Block, Borders, Wrap},
	Frame,
};
use std::{cell::Cell, path::Path};

pub struct SyntaxTextComponent {
	repo: RepoPathRef,
	current_file: Option<(String, Either<ui::SyntaxText, String>)>,
	loaded_tree_file: Option<TreeFile>,
	generation: u64,
	async_highlighting: AsyncSingleJob<AsyncSyntaxJob>,
	syntax_progress: Option<ProgressPercent>,
	key_config: SharedKeyConfig,
	paragraph_state: Cell<ParagraphState>,
	focused: bool,
	theme: SharedTheme,
}

impl SyntaxTextComponent {
	///
	pub fn new(env: &Environment) -> Self {
		Self {
			async_highlighting: AsyncSingleJob::new(
				env.sender_app.clone(),
			),
			syntax_progress: None,
			current_file: None,
			loaded_tree_file: None,
			generation: 0,
			paragraph_state: Cell::new(ParagraphState::default()),
			focused: false,
			key_config: env.key_config.clone(),
			theme: env.theme.clone(),
			repo: env.repo.clone(),
		}
	}

	///
	pub fn update(&mut self, ev: AsyncNotification) {
		if let AsyncNotification::App(
			AsyncAppNotification::SyntaxHighlighting(progress),
		) = ev
		{
			match progress {
				SyntaxHighlightProgress::Progress => {
					if self.current_file.is_some() {
						self.syntax_progress =
							self.async_highlighting.progress();
					}
				}
				SyntaxHighlightProgress::Done => {
					if let Some(job) =
						self.async_highlighting.take_last()
					{
						if let Some(syntax) = job.result() {
							if syntax.generation() == self.generation
							{
								self.syntax_progress = None;
								if let Some((path, content)) =
									self.current_file.as_mut()
								{
									if syntax.path()
										== Path::new(path)
									{
										*content =
											Either::Left(syntax);
									}
								}
							}
						}
					}
				}
			}
		}
	}

	///
	pub fn any_work_pending(&self) -> bool {
		self.async_highlighting.is_pending()
	}

	///
	pub fn has_lines(&self) -> bool {
		self.current_file.as_ref().is_some_and(|(_, content)| {
			match content {
				Either::Left(syn) => !syn.is_empty(),
				Either::Right(s) => s.lines().next().is_some(),
			}
		})
	}

	///
	pub fn clear(&mut self) {
		self.current_file = None;
		self.loaded_tree_file = None;
		self.syntax_progress = None;
		self.generation = self.generation.wrapping_add(1);
		self.paragraph_state.set(ParagraphState::default());
	}

	///
	pub fn load_file(&mut self, path: String, item: &TreeFile) {
		let already_loaded = self
			.loaded_tree_file
			.as_ref()
			.is_some_and(|loaded| loaded == item)
			&& self.current_file.as_ref().is_some_and(
				|(current_file, _)| current_file == &path,
			);

		if !already_loaded {
			self.generation = self.generation.wrapping_add(1);
			self.syntax_progress = None;
			self.paragraph_state.set(ParagraphState::default());
			self.loaded_tree_file = Some(item.clone());
			//TODO: fetch file content async as well
			match sync::tree_file_content(&self.repo.borrow(), item) {
				Ok(content) => {
					let content = tabs_to_spaces(content);
					self.syntax_progress =
						Some(ProgressPercent::empty());
					self.async_highlighting.spawn(
						AsyncSyntaxJob::with_generation(
							content.clone(),
							path.clone(),
							self.theme.get_syntax(),
							self.generation,
						),
					);

					self.current_file =
						Some((path, Either::Right(content)));
				}
				Err(e) => {
					self.current_file = Some((
						path,
						Either::Right(format!(
							"error loading file: {e}"
						)),
					));
				}
			}
		}
	}

	fn scroll(&self, nav: MoveSelection) -> bool {
		let state = self.paragraph_state.get();

		let new_scroll_pos = match nav {
			MoveSelection::Down => state.scroll().y.saturating_add(1),
			MoveSelection::Up => state.scroll().y.saturating_sub(1),
			MoveSelection::Top => 0,
			MoveSelection::End => state
				.lines()
				.saturating_sub(state.height().saturating_sub(2)),
			MoveSelection::PageUp => state
				.scroll()
				.y
				.saturating_sub(state.height().saturating_sub(2)),
			MoveSelection::PageDown => state
				.scroll()
				.y
				.saturating_add(state.height().saturating_sub(2)),
			_ => state.scroll().y,
		};

		self.set_scroll(new_scroll_pos)
	}

	fn set_scroll(&self, pos: u16) -> bool {
		let mut state = self.paragraph_state.get();

		let new_scroll_pos = pos.min(
			state
				.lines()
				.saturating_sub(state.height().saturating_sub(2)),
		);

		if new_scroll_pos == state.scroll().y {
			return false;
		}

		state.set_scroll(ScrollPos {
			x: 0,
			y: new_scroll_pos,
		});
		self.paragraph_state.set(state);

		true
	}
}

impl DrawableComponent for SyntaxTextComponent {
	fn draw(&self, f: &mut Frame, area: Rect) -> Result<()> {
		let text = self.current_file.as_ref().map_or_else(
			|| Text::from(""),
			|(_, content)| match content {
				Either::Left(syn) => syn.into(),
				Either::Right(s) => Text::from(s.as_str()),
			},
		);

		let title = format!(
			"{}{}",
			self.current_file
				.as_ref()
				.map(|(name, _)| name.clone())
				.unwrap_or_default(),
			self.syntax_progress
				.map(|p| format!(" ({}%)", p.progress))
				.unwrap_or_default()
		);

		let content = StatefulParagraph::new(text)
			.wrap(Wrap { trim: false })
			.block(
				Block::default()
					.title(title)
					.borders(Borders::ALL)
					.border_style(self.theme.title(self.focused())),
			);

		let mut state = self.paragraph_state.get();

		f.render_stateful_widget(content, area, &mut state);

		self.paragraph_state.set(state);

		if self.focused() {
			ui::draw_scrollbar(
				f,
				area,
				&self.theme,
				usize::from(state.lines().saturating_sub(
					state.height().saturating_sub(2),
				)),
				usize::from(state.scroll().y),
				ui::Orientation::Vertical,
			);
		}

		Ok(())
	}
}

impl Component for SyntaxTextComponent {
	fn commands(
		&self,
		out: &mut Vec<CommandInfo>,
		force_all: bool,
	) -> CommandBlocking {
		if self.focused() || force_all {
			out.push(
				CommandInfo::new(
					strings::commands::scroll(&self.key_config),
					true,
					true,
				)
				.order(strings::order::NAV),
			);
		}
		CommandBlocking::PassingOn
	}

	fn event(
		&mut self,
		event: &crossterm::event::Event,
	) -> Result<EventState> {
		if let Event::Key(key) = event {
			if let Some(nav) = common_nav(key, &self.key_config) {
				return Ok(if self.scroll(nav) {
					EventState::Consumed
				} else {
					EventState::NotConsumed
				});
			}
		}

		Ok(EventState::NotConsumed)
	}

	///
	fn focused(&self) -> bool {
		self.focused
	}

	/// focus/unfocus this component depending on param
	fn focus(&mut self, focus: bool) {
		self.focused = focus;
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use ratatui::{backend::TestBackend, Terminal};

	fn buffer_text(terminal: &Terminal<TestBackend>) -> String {
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
	fn t8n_file_view_clamps_first_frame() {
		let env = Environment::test_env();
		let mut comp = SyntaxTextComponent::new(&env);
		let twenty_lines = (0..20)
			.map(|i| format!("line-{i:02}"))
			.collect::<Vec<_>>()
			.join("\n");
		comp.current_file = Some((
			String::from("sample.txt"),
			Either::Right(twenty_lines),
		));

		// Draw at height 7 (inner height 5) and scroll to bottom (y = 15)
		let mut term =
			Terminal::new(TestBackend::new(20, 7)).unwrap();
		term.draw(|f| {
			comp.draw(f, f.area()).unwrap();
		})
		.unwrap();
		assert!(comp.scroll(MoveSelection::End));
		assert_eq!(comp.paragraph_state.get().scroll().y, 15);

		// Resize taller to height 12 (inner height 10): max_scroll = 10.
		// Must clamp in the SAME frame with no blank frame.
		let mut term =
			Terminal::new(TestBackend::new(20, 12)).unwrap();
		term.draw(|f| {
			comp.draw(f, f.area()).unwrap();
		})
		.unwrap();
		assert_eq!(comp.paragraph_state.get().scroll().y, 10);
		let rendered = buffer_text(&term);
		assert!(
			rendered.contains("line-10")
				&& rendered.contains("line-19"),
			"expected clamped lines 10..=19 on first frame, got:\n{rendered}"
		);

		// Shrink content to 5 lines while scroll.y = 10: must clamp to 0
		// in the SAME frame with no blank frame.
		let five_lines = (0..5)
			.map(|i| format!("short-{i:02}"))
			.collect::<Vec<_>>()
			.join("\n");
		comp.current_file = Some((
			String::from("sample.txt"),
			Either::Right(five_lines),
		));
		let mut term =
			Terminal::new(TestBackend::new(20, 7)).unwrap();
		term.draw(|f| {
			comp.draw(f, f.area()).unwrap();
		})
		.unwrap();
		assert_eq!(comp.paragraph_state.get().scroll().y, 0);
		let rendered = buffer_text(&term);
		assert!(
			rendered.contains("short-00")
				&& rendered.contains("short-04"),
			"expected short-00..=short-04 on first frame, got:\n{rendered}"
		);

		// u16::MAX scroll offset must not overflow u16 and must render content.
		let mut state = comp.paragraph_state.get();
		state.set_scroll(ScrollPos { x: 0, y: u16::MAX });
		comp.paragraph_state.set(state);
		let mut term =
			Terminal::new(TestBackend::new(20, 7)).unwrap();
		term.draw(|f| {
			comp.draw(f, f.area()).unwrap();
		})
		.unwrap();
		assert_eq!(comp.paragraph_state.get().scroll().y, 0);
		assert!(buffer_text(&term).contains("short-00"));
	}

	fn test_env_for_repo(
		repo_path: &std::path::Path,
	) -> (
		Environment,
		crossbeam_channel::Receiver<AsyncAppNotification>,
	) {
		use crate::{options::Options, queue::Queue};
		use asyncgit::sync::RepoPath;
		use std::{cell::RefCell, rc::Rc};

		let (tx_git, _rx_git) = crossbeam_channel::unbounded();
		let (tx_app, rx_app) = crossbeam_channel::unbounded();
		let repo =
			RefCell::new(RepoPath::Path(repo_path.to_path_buf()));
		let env = Environment {
			queue: Queue::new(),
			theme: Rc::default(),
			key_config: Rc::default(),
			options: Rc::new(RefCell::new(Options::test_env())),
			repo,
			sender_git: tx_git,
			sender_app: tx_app,
		};
		(env, rx_app)
	}

	fn commit_file(
		repo_path: &std::path::Path,
		file: &str,
		content: &str,
		msg: &str,
	) -> asyncgit::sync::CommitId {
		use asyncgit::sync::RepoPath;
		let full = repo_path.join(file);
		if let Some(parent) = full.parent() {
			std::fs::create_dir_all(parent).unwrap();
		}
		std::fs::write(full, content).unwrap();
		let rp = RepoPath::Path(repo_path.to_path_buf());
		sync::stage_add_file(&rp, Path::new(file)).unwrap();
		sync::commit(&rp, msg).unwrap()
	}

	#[test]
	fn t8p_same_path_new_revision() {
		let (td, _repo) = git2_testing::repo_init();
		let rp =
			asyncgit::sync::RepoPath::Path(td.path().to_path_buf());
		let c1 = commit_file(td.path(), "a.txt", "rev-one", "c1");
		let c2 = commit_file(td.path(), "a.txt", "rev-two", "c2");
		let f1 = sync::tree_files(&rp, c1)
			.unwrap()
			.into_iter()
			.find(|f| f.path == Path::new("./a.txt"))
			.unwrap();
		let f2 = sync::tree_files(&rp, c2)
			.unwrap()
			.into_iter()
			.find(|f| f.path == Path::new("./a.txt"))
			.unwrap();

		let (env, _rx_app) = test_env_for_repo(td.path());
		let mut comp = SyntaxTextComponent::new(&env);
		let mut term =
			Terminal::new(TestBackend::new(40, 10)).unwrap();

		comp.load_file(String::from("a.txt"), &f1);
		term.draw(|f| comp.draw(f, f.area()).unwrap()).unwrap();
		assert!(buffer_text(&term).contains("rev-one"));

		comp.load_file(String::from("a.txt"), &f2);
		term.draw(|f| comp.draw(f, f.area()).unwrap()).unwrap();
		let rendered = buffer_text(&term);
		assert!(
			rendered.contains("rev-two"),
			"expected rev-two after loading new revision of same path, got:\n{rendered}"
		);
		assert!(!rendered.contains("rev-one"));
	}

	#[test]
	fn t8p_new_file_scroll_top() {
		let (td, _repo) = git2_testing::repo_init();
		let rp =
			asyncgit::sync::RepoPath::Path(td.path().to_path_buf());
		let long_a = (0..25)
			.map(|i| format!("file-a-{i:02}"))
			.collect::<Vec<_>>()
			.join("\n");
		let long_b = (0..25)
			.map(|i| format!("file-b-{i:02}"))
			.collect::<Vec<_>>()
			.join("\n");
		let _c1 = commit_file(td.path(), "a.txt", &long_a, "c1");
		let c2 = commit_file(td.path(), "b.txt", &long_b, "c2");
		let files = sync::tree_files(&rp, c2).unwrap();
		let fa = files
			.iter()
			.find(|f| f.path == Path::new("./a.txt"))
			.unwrap();
		let fb = files
			.iter()
			.find(|f| f.path == Path::new("./b.txt"))
			.unwrap();

		let (env, _rx_app) = test_env_for_repo(td.path());
		let mut comp = SyntaxTextComponent::new(&env);
		let mut term =
			Terminal::new(TestBackend::new(30, 7)).unwrap();

		comp.load_file(String::from("a.txt"), fa);
		term.draw(|f| comp.draw(f, f.area()).unwrap()).unwrap();
		assert!(comp.scroll(MoveSelection::End));
		assert!(comp.paragraph_state.get().scroll().y > 0);

		comp.load_file(String::from("b.txt"), fb);
		assert_eq!(
			comp.paragraph_state.get().scroll().y,
			0,
			"loading another file must reset scroll to top"
		);
		term.draw(|f| comp.draw(f, f.area()).unwrap()).unwrap();
		assert!(buffer_text(&term).contains("file-b-00"));
	}

	#[test]
	fn t8p_stale_highlight_loses() {
		let (td, _repo) = git2_testing::repo_init();
		let rp =
			asyncgit::sync::RepoPath::Path(td.path().to_path_buf());
		let c1 = commit_file(td.path(), "a.txt", "old-content", "c1");
		let c2 = commit_file(td.path(), "a.txt", "new-content", "c2");
		let f1 = sync::tree_files(&rp, c1)
			.unwrap()
			.into_iter()
			.find(|f| f.path == Path::new("./a.txt"))
			.unwrap();
		let f2 = sync::tree_files(&rp, c2)
			.unwrap()
			.into_iter()
			.find(|f| f.path == Path::new("./a.txt"))
			.unwrap();

		let (env, rx_app) = test_env_for_repo(td.path());
		let mut comp = SyntaxTextComponent::new(&env);
		let mut term =
			Terminal::new(TestBackend::new(40, 10)).unwrap();

		// Load rev 1 and wait for its highlight job to finish in background,
		// WITHOUT delivering Done to comp yet.
		comp.load_file(String::from("a.txt"), &f1);
		let deadline = std::time::Instant::now()
			+ std::time::Duration::from_secs(5);
		loop {
			assert!(
				std::time::Instant::now() < deadline,
				"timed out waiting for highlight job"
			);
			if rx_app
				.recv_timeout(std::time::Duration::from_millis(50))
				== Ok(AsyncAppNotification::SyntaxHighlighting(
					SyntaxHighlightProgress::Done,
				)) {
				break;
			}
		}

		// Now load rev 2 (same path!), simulating rev 1's Done arriving late:
		// keep the finished rev 1 job in place and deliver Done while rev 2 is current.
		let stale_job = comp.async_highlighting.take_last().unwrap();
		comp.load_file(String::from("a.txt"), &f2);
		// Put stale_job into a fresh AsyncSingleJob? Instead, test via update
		// after replacing only the completed job isn't possible if private,
		// so deliver stale_job through a dedicated channel/slot:
		// Wait: AsyncSingleJob::spawn runs async; let's check by spawning a
		// second AsyncSyntaxJob with old content and delivering its Done!
		let _ = stale_job;
		comp.async_highlighting.spawn(AsyncSyntaxJob::new(
			String::from("old-content"),
			String::from("a.txt"),
			comp.theme.get_syntax(),
		));
		let deadline = std::time::Instant::now()
			+ std::time::Duration::from_secs(5);
		loop {
			assert!(
				std::time::Instant::now() < deadline,
				"timed out waiting for stale highlight job"
			);
			if rx_app
				.recv_timeout(std::time::Duration::from_millis(50))
				== Ok(AsyncAppNotification::SyntaxHighlighting(
					SyntaxHighlightProgress::Done,
				)) {
				break;
			}
		}

		comp.update(AsyncNotification::App(
			AsyncAppNotification::SyntaxHighlighting(
				SyntaxHighlightProgress::Done,
			),
		));

		term.draw(|f| comp.draw(f, f.area()).unwrap()).unwrap();
		let rendered = buffer_text(&term);
		assert!(
			rendered.contains("new-content"),
			"stale highlight for same path must not overwrite current content, got:\n{rendered}"
		);
		assert!(!rendered.contains("old-content"));
	}
}
