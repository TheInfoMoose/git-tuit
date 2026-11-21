fn get_git_hash() -> String {
	use std::process::Command;

	// Allow builds from `git archive` generated tarballs if output of `git get-tar-commit-id` is
	// set in an env var.
	if let Ok(commit) = std::env::var("BUILD_GIT_COMMIT_ID") {
		return commit.chars().take(7).collect();
	};
	let commit = Command::new("git")
		.arg("rev-parse")
		.arg("--short=7")
		.arg("--verify")
		.arg("HEAD")
		.output();
	if let Ok(commit_output) = commit {
		let commit_string =
			String::from_utf8_lossy(&commit_output.stdout);

		return commit_string.lines().next().unwrap_or("").into();
	}

	println!(
		"cargo:warning=can not get git commit: {}",
		commit.unwrap_err()
	);
	"unknown".into()
}

fn main() {
	let build_date = std::env::var("SOURCE_DATE_EPOCH")
		.ok()
		.and_then(|val| val.parse::<i64>().ok())
		.and_then(|secs| jiff::Timestamp::from_second(secs).ok())
		.map_or_else(
			|| jiff::Zoned::now().date(),
			|ts| ts.to_zoned(jiff::tz::TimeZone::system()).date(),
		);

	let build_name = if std::env::var("GITUI_RELEASE").is_ok() {
		env!("CARGO_PKG_VERSION").to_string()
	} else {
		format!(
			"{}-nightly {} ({})",
			env!("CARGO_PKG_VERSION"),
			build_date,
			get_git_hash()
		)
	};

	if std::env::var("GITUI_RELEASE").is_err() {
		println!("cargo:warning=buildname '{build_name}'");
	}
	println!("cargo:rustc-env=GITUI_BUILD_NAME={build_name}");
}
