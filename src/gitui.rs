use std::time::Instant;

use anyhow::Result;
use asyncgit::{sync::utils::repo_work_dir, AsyncGitNotification};
use crossbeam_channel::{never, tick, unbounded, Receiver};
use scopetime::scope_time;

#[cfg(test)]
use crossterm::event::{KeyCode, KeyModifiers};

use crate::{
	app::{App, QuitState},
	args::CliArgs,
	draw,
	input::{Input, InputEvent, InputState},
	keys::KeyConfig,
	select_event,
	spinner::Spinner,
	ui::style::Theme,
	watcher::RepoWatcher,
	AsyncAppNotification, AsyncNotification, QueueEvent, Updater,
	SPINNER_INTERVAL, TICK_INTERVAL,
};

pub struct Gitui {
	app: crate::app::App,
	rx_input: Receiver<InputEvent>,
	rx_git: Receiver<AsyncGitNotification>,
	rx_app: Receiver<AsyncAppNotification>,
	rx_ticker: Receiver<Instant>,
	rx_watcher: Receiver<()>,
}

impl Gitui {
	pub(crate) fn new(
		cliargs: CliArgs,
		theme: Theme,
		key_config: &KeyConfig,
		updater: Updater,
	) -> Result<Self, anyhow::Error> {
		let (tx_git, rx_git) = unbounded();
		let (tx_app, rx_app) = unbounded();

		let input = Input::new();

		let (rx_ticker, rx_watcher) = match updater {
			Updater::NotifyWatcher => {
				let repo_watcher = RepoWatcher::new(
					repo_work_dir(&cliargs.repo_path)?.as_str(),
				);

				(never(), repo_watcher.receiver())
			}
			Updater::Ticker => (tick(TICK_INTERVAL), never()),
		};

		let app = App::new(
			cliargs,
			tx_git,
			tx_app,
			input.clone(),
			theme,
			key_config.clone(),
		)?;

		Ok(Self {
			app,
			rx_input: input.receiver(),
			rx_git,
			rx_app,
			rx_ticker,
			rx_watcher,
		})
	}

	pub(crate) fn run_main_loop<B: ratatui::backend::Backend>(
		&mut self,
		terminal: &mut ratatui::Terminal<B>,
	) -> Result<QuitState, anyhow::Error>
	where
		<B as ratatui::backend::Backend>::Error:
			'static + Send + Sync,
	{
		let spinner_ticker = tick(SPINNER_INTERVAL);
		let mut spinner = Spinner::default();
		let mut first_update = true;

		self.app.update()?;

		loop {
			let event = if first_update {
				first_update = false;
				QueueEvent::Notify
			} else {
				select_event(
					&self.rx_input,
					&self.rx_git,
					&self.rx_app,
					&self.rx_ticker,
					&self.rx_watcher,
					&spinner_ticker,
				)?
			};

			{
				if matches!(event, QueueEvent::SpinnerUpdate) {
					spinner.update();
					spinner.draw(terminal)?;
					continue;
				}

				scope_time!("loop");

				match event {
					QueueEvent::InputEvent(ev) => {
						if matches!(
							ev,
							InputEvent::State(InputState::Polling)
						) {
							//Note: external ed closed, we need to re-hide cursor
							terminal.hide_cursor()?;
						}
						self.app.event(ev)?;
					}
					QueueEvent::Tick | QueueEvent::Notify => {
						self.app.update()?;
					}
					QueueEvent::AsyncEvent(ev) => {
						if !matches!(
							ev,
							AsyncNotification::Git(
								AsyncGitNotification::FinishUnchanged
							)
						) {
							self.app.update_async(ev)?;
						}
					}
					QueueEvent::SpinnerUpdate => unreachable!(),
				}

				self.draw(terminal)?;

				spinner.set_state(self.app.any_work_pending());
				spinner.draw(terminal)?;

				if self.app.is_quit() {
					break;
				}
			}
		}

		Ok(self.app.quit_state())
	}

	fn draw<B: ratatui::backend::Backend>(
		&self,
		terminal: &mut ratatui::Terminal<B>,
	) -> Result<(), B::Error> {
		draw(terminal, &self.app)
	}

	#[cfg(test)]
	fn update_async(&mut self, event: crate::AsyncNotification) {
		self.app.update_async(event).unwrap();
	}

	#[cfg(test)]
	fn input_event(
		&mut self,
		code: KeyCode,
		modifiers: KeyModifiers,
	) {
		let event = crossterm::event::KeyEvent::new(code, modifiers);
		self.app
			.event(crate::input::InputEvent::Input(
				crossterm::event::Event::Key(event),
			))
			.unwrap();
	}

	#[cfg(test)]
	fn wait_for_async_git_notification(
		&self,
		expected: AsyncGitNotification,
	) {
		loop {
			let actual = self
				.rx_git
				.recv_timeout(std::time::Duration::from_millis(100))
				.unwrap();

			if actual == expected {
				break;
			}
		}
	}

	#[cfg(test)]
	fn update(&mut self) {
		self.app.update().unwrap();
	}
}

#[cfg(test)]
mod tests {
	use std::path::PathBuf;

	use asyncgit::{sync::RepoPath, AsyncGitNotification};
	use crossterm::event::{KeyCode, KeyModifiers};
	use git2_testing::repo_init_suffix;
	use insta::assert_snapshot;
	use ratatui::{backend::TestBackend, Terminal};

	use crate::{
		args::CliArgs, gitui::Gitui, keys::KeyConfig,
		ui::style::Theme, AsyncNotification, Updater,
	};

	// Macro adapted from: https://insta.rs/docs/cmd/
	macro_rules! apply_common_filters {
		{} => {
			let mut settings = insta::Settings::clone_current();
			// Windows and MacOS
			// We don't match on the full path, but on the suffix we pass to `repo_init_suffix` below.
			settings.add_filter(r" *\[…\]\S+-insta/?", "[TEMP_FILE]");
			// Linux Temp Folder
			settings.add_filter(r" */tmp/\.tmp\S+-insta/", "[TEMP_FILE]");
			// Commit ids that follow a vertical bar
			settings.add_filter(r"│[a-z0-9]{7} ", "│[AAAAA] ");
			let _bound = settings.bind_to_scope();
		}
	}

	#[test]
	fn gitui_starts() {
		apply_common_filters!();

		let (temp_dir, _repo) = repo_init_suffix(Some("-insta"));
		let path: RepoPath = temp_dir.path().to_str().unwrap().into();
		let cliargs = CliArgs {
			theme: PathBuf::from("theme.ron"),
			select_file: None,
			repo_path: path,
			notify_watcher: false,
			key_bindings_path: None,
			key_symbols_path: None,
		};

		let theme = Theme::init(&PathBuf::new());
		let key_config = KeyConfig::default();

		let mut gitui =
			Gitui::new(cliargs, theme, &key_config, Updater::Ticker)
				.unwrap();

		let mut terminal =
			Terminal::new(TestBackend::new(90, 12)).unwrap();

		gitui.draw(&mut terminal).unwrap();

		assert_snapshot!("app_loading", terminal.backend());

		let event =
			AsyncNotification::Git(AsyncGitNotification::Status);
		gitui.update_async(event);

		gitui.draw(&mut terminal).unwrap();

		assert_snapshot!("app_loading_finished", terminal.backend());

		gitui.input_event(KeyCode::Char('2'), KeyModifiers::empty());
		gitui.input_event(
			key_config.keys.tab_log.code,
			key_config.keys.tab_log.modifiers,
		);

		gitui.wait_for_async_git_notification(
			AsyncGitNotification::Log,
		);

		gitui.update();

		gitui.draw(&mut terminal).unwrap();

		assert_snapshot!(
			"app_log_tab_showing_one_commit",
			terminal.backend()
		);
	}

	fn screen_text(terminal: &Terminal<TestBackend>) -> String {
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

	fn commit_file(
		repo_path: &std::path::Path,
		file: &str,
		content: &str,
		msg: &str,
	) -> asyncgit::sync::CommitId {
		std::fs::write(repo_path.join(file), content).unwrap();
		let rp = RepoPath::Path(repo_path.to_path_buf());
		asyncgit::sync::stage_add_file(
			&rp,
			std::path::Path::new(file),
		)
		.unwrap();
		asyncgit::sync::commit(&rp, msg).unwrap()
	}

	fn current_repo_state(
		repo_path: &std::path::Path,
	) -> asyncgit::sync::RepoState {
		asyncgit::sync::repo_state(&RepoPath::Path(
			repo_path.to_path_buf(),
		))
		.unwrap()
	}

	fn shared_test_input() -> crate::input::Input {
		static INPUT: std::sync::OnceLock<crate::input::Input> =
			std::sync::OnceLock::new();
		INPUT
			.get_or_init(|| {
				let input = crate::input::Input::new();
				input.set_polling(false);
				input
			})
			.clone()
	}

	struct TestGitui(Gitui);

	impl std::ops::Deref for TestGitui {
		type Target = Gitui;

		fn deref(&self) -> &Self::Target {
			&self.0
		}
	}

	impl std::ops::DerefMut for TestGitui {
		fn deref_mut(&mut self) -> &mut Self::Target {
			&mut self.0
		}
	}

	impl Drop for TestGitui {
		fn drop(&mut self) {
			let deadline = std::time::Instant::now()
				+ std::time::Duration::from_secs(2);
			while self.0.app.any_work_pending()
				&& std::time::Instant::now() < deadline
			{
				let _ = self.0.rx_git.recv_timeout(
					std::time::Duration::from_millis(20),
				);
			}
		}
	}

	fn open_gitui_on_log(
		path: &std::path::Path,
	) -> (TestGitui, KeyConfig, Terminal<TestBackend>) {
		// Keep clones of rx_git and rx_app alive for the test binary so
		// background rayon jobs still in flight when a test drops Gitui
		// cannot hit SendError and abort the process.
		type RxPair = (
			crossbeam_channel::Receiver<AsyncGitNotification>,
			crossbeam_channel::Receiver<crate::AsyncAppNotification>,
		);
		static LIVE_RX: std::sync::OnceLock<
			std::sync::Mutex<Vec<RxPair>>,
		> = std::sync::OnceLock::new();

		let cliargs = CliArgs {
			theme: PathBuf::from("theme.ron"),
			select_file: None,
			repo_path: RepoPath::Path(path.to_path_buf()),
			notify_watcher: false,
			key_bindings_path: None,
			key_symbols_path: None,
		};
		let theme = Theme::init(&PathBuf::new());
		let key_config = KeyConfig::default();
		let (tx_git, rx_git) = crossbeam_channel::unbounded();
		let (tx_app, rx_app) = crossbeam_channel::unbounded();
		LIVE_RX
			.get_or_init(|| std::sync::Mutex::new(Vec::new()))
			.lock()
			.unwrap()
			.push((rx_git.clone(), rx_app.clone()));
		let input = shared_test_input();
		let app = crate::app::App::new(
			cliargs,
			tx_git,
			tx_app,
			input.clone(),
			theme,
			key_config.clone(),
		)
		.unwrap();
		let mut gitui = Gitui {
			app,
			rx_input: input.receiver(),
			rx_git,
			rx_app,
			rx_ticker: crossbeam_channel::never(),
			rx_watcher: crossbeam_channel::never(),
		};
		let mut terminal =
			Terminal::new(TestBackend::new(80, 24)).unwrap();

		gitui.input_event(
			key_config.keys.tab_log.code,
			key_config.keys.tab_log.modifiers,
		);
		gitui.wait_for_async_git_notification(
			AsyncGitNotification::Log,
		);
		gitui.update();
		gitui.draw(&mut terminal).unwrap();
		(TestGitui(gitui), key_config, terminal)
	}

	#[test]
	fn t8c_revert_asks_first() {
		let (td, repo) = git2_testing::repo_init();
		let c1 =
			commit_file(td.path(), "a.txt", "hello", "add greeting");
		let branch =
			repo.head().unwrap().shorthand().unwrap().to_string();

		let (mut gitui, kc, mut terminal) =
			open_gitui_on_log(td.path());

		gitui.input_event(
			kc.keys.status_reset_item.code,
			kc.keys.status_reset_item.modifiers,
		);
		gitui.draw(&mut terminal).unwrap();

		assert!(
			gitui.app.is_confirm_popup_visible(),
			"revert must open confirm popup first"
		);
		assert_eq!(
			current_repo_state(td.path()),
			asyncgit::sync::RepoState::Clean,
			"repo must not be reverted before confirming"
		);
		assert_eq!(gitui.app.current_tab(), 1);

		let screen = screen_text(&terminal);
		assert!(screen.contains("Revert"), "screen:\n{screen}");
		assert!(
			screen.contains(&c1.get_short_string()),
			"screen:\n{screen}"
		);
		assert!(screen.contains("add greeting"), "screen:\n{screen}");
		assert!(screen.contains(&branch), "screen:\n{screen}");
	}

	#[test]
	fn t8c_checkout_asks_first() {
		let (td, repo) = git2_testing::repo_init();
		let c0 = asyncgit::sync::CommitId::new(
			repo.head().unwrap().peel_to_commit().unwrap().id(),
		);
		let _c1 =
			commit_file(td.path(), "a.txt", "hello", "second commit");
		let branch =
			repo.head().unwrap().shorthand().unwrap().to_string();

		let (mut gitui, kc, mut terminal) =
			open_gitui_on_log(td.path());

		// Select initial commit.
		gitui.input_event(
			kc.keys.move_down.code,
			kc.keys.move_down.modifiers,
		);
		gitui.input_event(
			kc.keys.log_checkout_commit.code,
			kc.keys.log_checkout_commit.modifiers,
		);
		gitui.draw(&mut terminal).unwrap();

		assert!(
			gitui.app.is_confirm_popup_visible(),
			"checkout must open confirm popup first"
		);
		assert!(
			!repo.head_detached().unwrap(),
			"HEAD must not detach before confirming"
		);

		let screen = screen_text(&terminal);
		assert!(screen.contains("Checkout"), "screen:\n{screen}");
		assert!(
			screen.contains(&c0.get_short_string()),
			"screen:\n{screen}"
		);
		assert!(screen.contains("initial"), "screen:\n{screen}");
		assert!(screen.contains(&branch), "screen:\n{screen}");
		assert!(
			screen.to_lowercase().contains("detached"),
			"screen:\n{screen}"
		);
	}

	#[test]
	fn t8c_cancel_runs_nothing() {
		let (td, repo) = git2_testing::repo_init();
		let _c1 =
			commit_file(td.path(), "a.txt", "hello", "add greeting");
		let (mut gitui, kc, _terminal) = open_gitui_on_log(td.path());

		// Open revert prompt, press Esc, then press Enter.
		gitui.input_event(
			kc.keys.status_reset_item.code,
			kc.keys.status_reset_item.modifiers,
		);
		assert!(gitui.app.is_confirm_popup_visible());
		gitui.input_event(
			kc.keys.exit_popup.code,
			kc.keys.exit_popup.modifiers,
		);
		assert!(!gitui.app.is_confirm_popup_visible());
		gitui
			.input_event(kc.keys.enter.code, kc.keys.enter.modifiers);
		assert_eq!(
			current_repo_state(td.path()),
			asyncgit::sync::RepoState::Clean
		);
		assert_eq!(
			std::fs::read_to_string(td.path().join("a.txt")).unwrap(),
			"hello"
		);

		// Open checkout prompt, press Esc, then press Enter.
		gitui.input_event(
			kc.keys.log_checkout_commit.code,
			kc.keys.log_checkout_commit.modifiers,
		);
		assert!(gitui.app.is_confirm_popup_visible());
		gitui.input_event(
			kc.keys.exit_popup.code,
			kc.keys.exit_popup.modifiers,
		);
		assert!(!gitui.app.is_confirm_popup_visible());
		gitui
			.input_event(kc.keys.enter.code, kc.keys.enter.modifiers);
		assert!(!repo.head_detached().unwrap());
	}

	#[test]
	fn t8c_confirm_runs_once() {
		// 1. Revert succeeds -> switches to Status tab, runs once.
		{
			let (td, _repo) = git2_testing::repo_init();
			let _c1 = commit_file(
				td.path(),
				"a.txt",
				"hello",
				"add greeting",
			);
			let (mut gitui, kc, _terminal) =
				open_gitui_on_log(td.path());

			gitui.input_event(
				kc.keys.status_reset_item.code,
				kc.keys.status_reset_item.modifiers,
			);
			assert!(gitui.app.is_confirm_popup_visible());
			gitui.input_event(
				kc.keys.enter.code,
				kc.keys.enter.modifiers,
			);
			assert!(!gitui.app.is_confirm_popup_visible());
			assert_eq!(
				current_repo_state(td.path()),
				asyncgit::sync::RepoState::Revert
			);
			assert_eq!(
				gitui.app.current_tab(),
				0,
				"revert success must switch to Status tab"
			);
			assert!(!gitui.app.is_msg_popup_visible());
		}

		// 2. Revert fails -> stays on Log tab and shows error popup.
		{
			let (td, _repo) = git2_testing::repo_init();
			let _c1 = commit_file(
				td.path(),
				"a.txt",
				"hello",
				"add greeting",
			);
			// Dirty worktree on the same file makes revert fail.
			std::fs::write(td.path().join("a.txt"), "dirty").unwrap();
			let (mut gitui, kc, _terminal) =
				open_gitui_on_log(td.path());

			gitui.input_event(
				kc.keys.status_reset_item.code,
				kc.keys.status_reset_item.modifiers,
			);
			assert!(gitui.app.is_confirm_popup_visible());
			gitui.input_event(
				kc.keys.enter.code,
				kc.keys.enter.modifiers,
			);
			assert!(!gitui.app.is_confirm_popup_visible());
			assert_eq!(
				gitui.app.current_tab(),
				1,
				"failed revert must not switch to Status"
			);
			assert!(
				gitui.app.is_msg_popup_visible(),
				"failed revert must show error popup"
			);
		}

		// 3. Checkout succeeds -> detaches HEAD to selected commit.
		{
			let (td, repo) = git2_testing::repo_init();
			let c0 =
				repo.head().unwrap().peel_to_commit().unwrap().id();
			let _c1 = commit_file(
				td.path(),
				"a.txt",
				"hello",
				"add greeting",
			);
			let (mut gitui, kc, _terminal) =
				open_gitui_on_log(td.path());

			gitui.input_event(
				kc.keys.move_down.code,
				kc.keys.move_down.modifiers,
			);
			gitui.input_event(
				kc.keys.log_checkout_commit.code,
				kc.keys.log_checkout_commit.modifiers,
			);
			assert!(gitui.app.is_confirm_popup_visible());
			gitui.input_event(
				kc.keys.enter.code,
				kc.keys.enter.modifiers,
			);
			assert!(!gitui.app.is_confirm_popup_visible());
			assert!(repo.head_detached().unwrap());
			assert_eq!(
				repo.head().unwrap().peel_to_commit().unwrap().id(),
				c0
			);
		}
	}

	#[test]
	fn t8c_prompt_pins_commit() {
		let (td, repo) = git2_testing::repo_init();
		let c0 = asyncgit::sync::CommitId::new(
			repo.head().unwrap().peel_to_commit().unwrap().id(),
		);
		let c1 = commit_file(td.path(), "a.txt", "v1", "first");
		let _c2 = commit_file(td.path(), "a.txt", "v2", "second");
		let (mut gitui, kc, _terminal) = open_gitui_on_log(td.path());

		// Select c1 and open checkout confirmation.
		gitui.app.revlog_mut().select_commit(c1).unwrap();
		gitui.update();
		gitui.input_event(
			kc.keys.log_checkout_commit.code,
			kc.keys.log_checkout_commit.modifiers,
		);
		assert!(gitui.app.is_confirm_popup_visible());

		// Move selection and refresh list while prompt is open.
		gitui.app.revlog_mut().select_commit(c0).unwrap();
		gitui.update();

		// Confirm: c1 (the commit named in the prompt) must win.
		gitui
			.input_event(kc.keys.enter.code, kc.keys.enter.modifiers);
		assert!(repo.head_detached().unwrap());
		assert_eq!(
			asyncgit::sync::CommitId::new(
				repo.head().unwrap().peel_to_commit().unwrap().id(),
			),
			c1
		);
	}

	#[test]
	fn t8c_stale_confirm_refuses() {
		let (td, repo) = git2_testing::repo_init();
		let _c1 = commit_file(td.path(), "a.txt", "v1", "first");
		let (mut gitui, kc, mut terminal) =
			open_gitui_on_log(td.path());

		gitui.input_event(
			kc.keys.log_checkout_commit.code,
			kc.keys.log_checkout_commit.modifiers,
		);
		assert!(gitui.app.is_confirm_popup_visible());

		// External commit advances HEAD while prompt is open.
		let c2 = commit_file(td.path(), "b.txt", "v2", "external");

		gitui
			.input_event(kc.keys.enter.code, kc.keys.enter.modifiers);
		gitui.draw(&mut terminal).unwrap();

		assert!(!gitui.app.is_confirm_popup_visible());
		assert!(
			gitui.app.is_msg_popup_visible(),
			"stale confirm must show error"
		);
		assert!(!repo.head_detached().unwrap());
		assert_eq!(
			asyncgit::sync::CommitId::new(
				repo.head().unwrap().peel_to_commit().unwrap().id(),
			),
			c2
		);
	}

	#[test]
	fn t8c_stale_branch_refuses() {
		let (td, repo) = git2_testing::repo_init();
		let _c1 = commit_file(td.path(), "a.txt", "v1", "first");
		let head_commit =
			repo.head().unwrap().peel_to_commit().unwrap();
		repo.branch("other", &head_commit, false).unwrap();

		let (mut gitui, kc, _terminal) = open_gitui_on_log(td.path());

		// 1. Switch to another branch pointing to the SAME commit.
		gitui.input_event(
			kc.keys.log_checkout_commit.code,
			kc.keys.log_checkout_commit.modifiers,
		);
		assert!(gitui.app.is_confirm_popup_visible());
		repo.set_head("refs/heads/other").unwrap();
		gitui
			.input_event(kc.keys.enter.code, kc.keys.enter.modifiers);
		assert!(gitui.app.is_msg_popup_visible());
		assert!(!repo.head_detached().unwrap());
		// Dismiss error popup.
		gitui.input_event(
			kc.keys.exit_popup.code,
			kc.keys.exit_popup.modifiers,
		);

		// 2. Detach HEAD at the SAME commit while prompt is open.
		gitui.input_event(
			kc.keys.status_reset_item.code,
			kc.keys.status_reset_item.modifiers,
		);
		assert!(gitui.app.is_confirm_popup_visible());
		repo.set_head_detached(head_commit.id()).unwrap();
		gitui
			.input_event(kc.keys.enter.code, kc.keys.enter.modifiers);
		assert!(gitui.app.is_msg_popup_visible());
		assert_eq!(
			current_repo_state(td.path()),
			asyncgit::sync::RepoState::Clean
		);
		gitui.input_event(
			kc.keys.exit_popup.code,
			kc.keys.exit_popup.modifiers,
		);

		// 3. Attach HEAD to a branch at the SAME commit while prompt is open.
		gitui.input_event(
			kc.keys.status_reset_item.code,
			kc.keys.status_reset_item.modifiers,
		);
		assert!(gitui.app.is_confirm_popup_visible());
		repo.set_head("refs/heads/other").unwrap();
		gitui
			.input_event(kc.keys.enter.code, kc.keys.enter.modifiers);
		assert!(gitui.app.is_msg_popup_visible());
		assert_eq!(
			current_repo_state(td.path()),
			asyncgit::sync::RepoState::Clean
		);
	}

	#[test]
	fn t8c_stale_state_refuses() {
		let (td, repo) = git2_testing::repo_init();
		let c1 = commit_file(td.path(), "a.txt", "v1", "first");
		let (mut gitui, kc, _terminal) = open_gitui_on_log(td.path());

		gitui.input_event(
			kc.keys.log_checkout_commit.code,
			kc.keys.log_checkout_commit.modifiers,
		);
		assert!(gitui.app.is_confirm_popup_visible());

		// Externally enter a pending merge state.
		std::fs::write(
			repo.path().join("MERGE_HEAD"),
			format!("{c1}\n"),
		)
		.unwrap();

		gitui
			.input_event(kc.keys.enter.code, kc.keys.enter.modifiers);
		assert!(gitui.app.is_msg_popup_visible());
		assert!(!repo.head_detached().unwrap());
	}

	#[test]
	fn t8c_confirm_fits_80x24() {
		let (td, repo) = git2_testing::repo_init();
		let long_subject = "feat: extremely-long-commit-subject-alpha-beta-gamma-delta-epsilon-zeta-eta-theta-iota-kappa-tail-end";
		let c1 = commit_file(td.path(), "a.txt", "v1", long_subject);
		let long_branch = "feature/very-long-branch-name-alpha-beta-gamma-delta-epsilon-zeta-eta-theta-final-part";
		let head_commit =
			repo.head().unwrap().peel_to_commit().unwrap();
		repo.branch(long_branch, &head_commit, false).unwrap();
		repo.set_head(&format!("refs/heads/{long_branch}")).unwrap();

		let (mut gitui, kc, mut terminal) =
			open_gitui_on_log(td.path());

		gitui.input_event(
			kc.keys.log_checkout_commit.code,
			kc.keys.log_checkout_commit.modifiers,
		);
		gitui.draw(&mut terminal).unwrap();

		let screen = screen_text(&terminal);
		assert!(screen.contains("Checkout"), "screen:\n{screen}");
		assert!(
			screen.contains(&c1.get_short_string()),
			"screen:\n{screen}"
		);
		assert!(
			screen.contains('\u{2026}'),
			"long names must be shortened in the middle:\n{screen}"
		);
		assert!(
			screen.contains("tail-end")
				|| screen.contains("final-part"),
			"middle shortening must keep the end of long names:\n{screen}"
		);
		assert!(
			screen.to_lowercase().contains("detached"),
			"whole message including detached warning must fit at 80x24:\n{screen}"
		);
	}
}
