use crate::bug_report;
use anyhow::{anyhow, bail, Context, Result};
use asyncgit::sync::RepoPath;
use clap::{
	builder::ArgPredicate, crate_authors, crate_description,
	crate_name, Arg, Command as ClapApp,
};
use simplelog::{Config, LevelFilter, WriteLogger};
use std::{
	env,
	fs::{self, File},
	path::{Path, PathBuf},
};

const BUG_REPORT_FLAG_ID: &str = "bugreport";
const LOG_FILE_FLAG_ID: &str = "logfile";
const LOGGING_FLAG_ID: &str = "logging";
const THEME_FLAG_ID: &str = "theme";
const WORKDIR_FLAG_ID: &str = "workdir";
const FILE_FLAG_ID: &str = "file";
const GIT_DIR_FLAG_ID: &str = "directory";
const WATCHER_FLAG_ID: &str = "watcher";
const KEY_BINDINGS_FLAG_ID: &str = "key_bindings";
const KEY_SYMBOLS_FLAG_ID: &str = "key_symbols";
const DEFAULT_THEME: &str = "theme.ron";
const DEFAULT_GIT_DIR: &str = ".";

#[derive(Clone)]
pub struct CliArgs {
	pub theme: PathBuf,
	pub select_file: Option<PathBuf>,
	pub repo_path: RepoPath,
	pub notify_watcher: bool,
	pub key_bindings_path: Option<PathBuf>,
	pub key_symbols_path: Option<PathBuf>,
}

pub fn process_cmdline() -> Result<CliArgs> {
	let app = app();

	let arg_matches = app.get_matches();

	if arg_matches.get_flag(BUG_REPORT_FLAG_ID) {
		bug_report::generate_bugreport();
		std::process::exit(0);
	}
	if arg_matches.get_flag(LOGGING_FLAG_ID) {
		let logfile = arg_matches.get_one::<String>(LOG_FILE_FLAG_ID);
		setup_logging(logfile.map(PathBuf::from))?;
	}

	let workdir = arg_matches
		.get_one::<String>(WORKDIR_FLAG_ID)
		.map(PathBuf::from);
	let gitdir =
		arg_matches.get_one::<String>(GIT_DIR_FLAG_ID).map_or_else(
			|| PathBuf::from(DEFAULT_GIT_DIR),
			PathBuf::from,
		);

	let select_file = arg_matches
		.get_one::<String>(FILE_FLAG_ID)
		.map(PathBuf::from);

	let repo_path = if let Some(w) = workdir {
		RepoPath::Workdir { gitdir, workdir: w }
	} else {
		RepoPath::Path(gitdir)
	};

	let arg_theme = arg_matches
		.get_one::<String>(THEME_FLAG_ID)
		.map_or_else(|| PathBuf::from(DEFAULT_THEME), PathBuf::from);

	let confpath = get_app_config_path()?;
	fs::create_dir_all(&confpath).with_context(|| {
		format!(
			"failed to create config directory: {}",
			confpath.display()
		)
	})?;
	let theme = confpath.join(arg_theme);

	let notify_watcher: bool =
		*arg_matches.get_one(WATCHER_FLAG_ID).unwrap_or(&false);

	let key_bindings_path = arg_matches
		.get_one::<String>(KEY_BINDINGS_FLAG_ID)
		.map(PathBuf::from);

	let key_symbols_path = arg_matches
		.get_one::<String>(KEY_SYMBOLS_FLAG_ID)
		.map(PathBuf::from);

	Ok(CliArgs {
		theme,
		select_file,
		repo_path,
		notify_watcher,
		key_bindings_path,
		key_symbols_path,
	})
}

fn app() -> ClapApp {
	ClapApp::new(crate_name!())
		.author(crate_authors!())
		.version(env!("GITUI_BUILD_NAME"))
		.about(crate_description!())
		.help_template(
			"\
{before-help}gitui {version}
{author}
{about}

{usage-heading} {usage}

{all-args}{after-help}
		",
		)
			.arg(
			Arg::new(KEY_BINDINGS_FLAG_ID)
				.help("Use a custom keybindings file")
				.short('k')
				.long("key-bindings")
				.value_name("KEY_LIST_FILENAME")
				.num_args(1),
		)
			.arg(
			Arg::new(KEY_SYMBOLS_FLAG_ID)
				.help("Use a custom symbols file")
				.short('s')
				.long("key-symbols")
				.value_name("KEY_SYMBOLS_FILENAME")
				.num_args(1),
		)
		.arg(
			Arg::new(THEME_FLAG_ID)
				.help("Set color theme filename loaded from config directory")
				.short('t')
				.long("theme")
				.value_name("THEME_FILE")
				.default_value(DEFAULT_THEME)
				.num_args(1),
		)
		.arg(
			Arg::new(LOGGING_FLAG_ID)
				.help("Store logging output into a file (in the cache directory by default)")
				.short('l')
				.long("logging")
                .default_value_if("logfile", ArgPredicate::IsPresent, "true")
				.action(clap::ArgAction::SetTrue),
		)
        .arg(Arg::new(LOG_FILE_FLAG_ID)
            .help("Store logging output into the specified file (implies --logging)")
            .long("logfile")
            .value_name("LOG_FILE"))
		.arg(
			Arg::new(WATCHER_FLAG_ID)
				.help("Use notify-based file system watcher instead of tick-based update. This is more performant, but can cause issues on some platforms. See https://github.com/gitui-org/gitui/blob/master/FAQ.md#watcher for details.")
				.long("watcher")
				.action(clap::ArgAction::SetTrue),
		)
		.arg(
			Arg::new(BUG_REPORT_FLAG_ID)
				.help("Generate a bug report")
				.long("bugreport")
				.action(clap::ArgAction::SetTrue),
		)
		.arg(
			Arg::new(FILE_FLAG_ID)
				.help("Select the file in the file tab")
				.short('f')
				.long("file")
				.num_args(1),
		)
		.arg(
			Arg::new(GIT_DIR_FLAG_ID)
				.help("Set the git directory")
				.short('d')
				.long("directory")
				.env("GIT_DIR")
				.num_args(1),
		)
		.arg(
			Arg::new(WORKDIR_FLAG_ID)
				.help("Set the working directory")
				.short('w')
				.long("workdir")
				.env("GIT_WORK_TREE")
				.num_args(1),
		)
}

fn setup_logging(path_override: Option<PathBuf>) -> Result<()> {
	let path = if let Some(path) = path_override {
		path
	} else {
		let mut path = get_app_cache_path()?;
		path.push("gitui.log");
		path
	};

	println!("Logging enabled. Log written to: {}", path.display());

	WriteLogger::init(
		LevelFilter::Trace,
		Config::default(),
		open_log_file(&path)?,
	)?;

	Ok(())
}

fn open_log_file(path: &Path) -> Result<File> {
	if fs::symlink_metadata(path)
		.is_ok_and(|m| m.file_type().is_symlink())
	{
		bail!(
			"refusing to write log through symlink: {}",
			path.display()
		);
	}
	let mut options = fs::OpenOptions::new();
	options.write(true).create(true).truncate(true);
	#[cfg(unix)]
	{
		use std::os::unix::fs::OpenOptionsExt;
		options.mode(0o600);
	}
	let file = options.open(path)?;
	#[cfg(unix)]
	{
		use std::os::unix::fs::PermissionsExt;
		file.set_permissions(fs::Permissions::from_mode(0o600))?;
	}
	Ok(file)
}

fn get_app_cache_path() -> Result<PathBuf> {
	let mut path = dirs::cache_dir()
		.ok_or_else(|| anyhow!("failed to find os cache dir."))?;

	path.push("gitui");
	fs::create_dir_all(&path).with_context(|| {
		format!(
			"failed to create cache directory: {}",
			path.display()
		)
	})?;
	Ok(path)
}

pub fn get_app_config_path() -> Result<PathBuf> {
	let mut path = if cfg!(target_os = "macos") {
		dirs::home_dir().map(|h| h.join(".config"))
	} else {
		dirs::config_dir()
	}
	.ok_or_else(|| anyhow!("failed to find os config dir."))?;

	path.push("gitui");
	Ok(path)
}

#[test]
fn verify_app() {
	app().debug_assert();
}

#[cfg(test)]
mod tests {
	use super::*;

	#[cfg(unix)]
	#[test]
	fn test_open_log_file_mode_0600() {
		use std::os::unix::fs::PermissionsExt;
		let dir = tempfile::tempdir().unwrap();
		let path = dir.path().join("gitui.log");
		drop(open_log_file(&path).unwrap());
		assert_eq!(
			fs::metadata(&path).unwrap().permissions().mode() & 0o777,
			0o600
		);
	}

	#[cfg(unix)]
	#[test]
	fn test_open_log_file_refuses_symlink() {
		let dir = tempfile::tempdir().unwrap();
		let target = dir.path().join("target.txt");
		fs::write(&target, "keep").unwrap();
		let link = dir.path().join("gitui.log");
		std::os::unix::fs::symlink(&target, &link).unwrap();
		assert!(open_log_file(&link).is_err());
		assert_eq!(fs::read_to_string(&target).unwrap(), "keep");
	}

	#[cfg(unix)]
	#[test]
	fn test_open_log_file_tightens_existing_mode() {
		use std::os::unix::fs::PermissionsExt;
		let dir = tempfile::tempdir().unwrap();
		let path = dir.path().join("gitui.log");
		fs::write(&path, "old").unwrap();
		fs::set_permissions(&path, fs::Permissions::from_mode(0o644))
			.unwrap();
		drop(open_log_file(&path).unwrap());
		assert_eq!(
			fs::metadata(&path).unwrap().permissions().mode() & 0o777,
			0o600
		);
	}
}
