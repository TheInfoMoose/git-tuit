//! branch functions

pub mod merge_commit;
pub mod merge_ff;
pub mod merge_rebase;
pub mod rename;

use super::{utils::bytes2string, RepoPath};
use crate::{
	error::{Error, Result},
	sync::{
		remotes::get_default_remote_for_push_in_repo,
		repository::repo, utils::get_head_repo, CommitId,
	},
};
use git2::{Branch, BranchType, Repository};
use scopetime::scope_time;
use std::collections::HashSet;

/// returns the branch-name head is currently pointing to
/// this might be expensive, see `cached::BranchName`
pub(crate) fn get_branch_name(
	repo_path: &RepoPath,
) -> Result<String> {
	let repo = repo(repo_path)?;

	get_branch_name_repo(&repo)
}

/// ditto
pub(crate) fn get_branch_name_repo(
	repo: &Repository,
) -> Result<String> {
	scope_time!("get_branch_name_repo");

	let head_ref = repo.head().map_err(|e| {
		if e.code() == git2::ErrorCode::UnbornBranch {
			Error::NoHead
		} else {
			e.into()
		}
	})?;

	bytes2string(head_ref.shorthand_bytes())
}

///
#[derive(Clone, Debug)]
pub struct LocalBranch {
	///
	pub is_head: bool,
	///
	pub has_upstream: bool,
	///
	pub upstream: Option<UpstreamBranch>,
	///
	pub remote: Option<String>,
}

///
#[derive(Clone, Debug)]
pub struct UpstreamBranch {
	///
	pub reference: String,
}

///
#[derive(Clone, Debug)]
pub struct RemoteBranch {
	///
	pub has_tracking: bool,
}

///
#[derive(Clone, Debug)]
pub enum BranchDetails {
	///
	Local(LocalBranch),
	///
	Remote(RemoteBranch),
}

///
#[derive(Clone, Debug)]
pub struct BranchInfo {
	///
	pub name: String,
	///
	pub reference: String,
	///
	pub top_commit_message: String,
	///
	pub top_commit: CommitId,
	///
	pub details: BranchDetails,
}

impl BranchInfo {
	/// returns details about local branch or None
	pub const fn local_details(&self) -> Option<&LocalBranch> {
		if let BranchDetails::Local(details) = &self.details {
			return Some(details);
		}

		None
	}

	/// returns whether branch is local
	pub const fn is_local(&self) -> bool {
		matches!(self.details, BranchDetails::Local(_))
	}
}

///
pub fn validate_branch_name(name: &str) -> Result<bool> {
	scope_time!("validate_branch_name");

	let valid = Branch::name_is_valid(name)?;

	Ok(valid)
}

/// returns a list of `BranchInfo` with a simple summary on each branch
/// `local` filters for local branches otherwise remote branches will be returned
pub fn get_branches_info(
	repo_path: &RepoPath,
	local: bool,
) -> Result<Vec<BranchInfo>> {
	scope_time!("get_branches_info");

	let repo = repo(repo_path)?;

	let (filter, remotes_with_tracking) = if local {
		(BranchType::Local, HashSet::default())
	} else {
		let remotes: HashSet<_> = repo
			.branches(Some(BranchType::Local))?
			.filter_map(|b| {
				let branch = b.ok()?.0;
				let upstream = branch.upstream();
				upstream
					.ok()?
					.name_bytes()
					.ok()
					.map(ToOwned::to_owned)
			})
			.collect();
		(BranchType::Remote, remotes)
	};

	let mut branches_for_display: Vec<BranchInfo> = repo
		.branches(Some(filter))?
		.map(|b| {
			let branch = b?.0;
			let top_commit = branch.get().peel_to_commit()?;
			let reference = bytes2string(branch.get().name_bytes())?;
			let upstream = branch.upstream();

			let remote = repo
				.branch_upstream_remote(&reference)
				.ok()
				.as_ref()
				.and_then(|upstream_remote| {
					git2::Buf::as_str(upstream_remote).ok()
				})
				.map(String::from);

			let name_bytes = branch.name_bytes()?;

			let upstream_branch =
				upstream.ok().and_then(|upstream| {
					bytes2string(upstream.get().name_bytes())
						.ok()
						.map(|reference| UpstreamBranch { reference })
				});

			let details = if local {
				BranchDetails::Local(LocalBranch {
					is_head: branch.is_head(),
					has_upstream: upstream_branch.is_some(),
					upstream: upstream_branch,
					remote,
				})
			} else {
				BranchDetails::Remote(RemoteBranch {
					has_tracking: remotes_with_tracking
						.contains(name_bytes),
				})
			};

			Ok(BranchInfo {
				name: bytes2string(name_bytes)?,
				reference,
				top_commit_message: bytes2string(
					top_commit.summary_bytes().unwrap_or_default(),
				)?,
				top_commit: top_commit.id().into(),
				details,
			})
		})
		.filter_map(Result::ok)
		.collect();

	branches_for_display.sort_by(|a, b| a.name.cmp(&b.name));

	Ok(branches_for_display)
}

///
#[derive(Debug, Default)]
pub struct BranchCompare {
	///
	pub ahead: usize,
	///
	pub behind: usize,
}

///
pub(crate) fn branch_set_upstream_after_push(
	repo: &Repository,
	branch_name: &str,
) -> Result<()> {
	scope_time!("branch_set_upstream");

	let mut branch =
		repo.find_branch(branch_name, BranchType::Local)?;

	if branch.upstream().is_err() {
		let remote = get_default_remote_for_push_in_repo(repo)?;
		let upstream_name = format!("{remote}/{branch_name}");
		branch.set_upstream(Some(upstream_name.as_str()))?;
	}

	Ok(())
}

/// returns remote of the upstream tracking branch for `branch`
pub fn get_branch_remote(
	repo_path: &RepoPath,
	branch: &str,
) -> Result<Option<String>> {
	let repo = repo(repo_path)?;
	let branch = repo.find_branch(branch, BranchType::Local)?;
	let reference = bytes2string(branch.get().name_bytes())?;
	let remote_name = repo.branch_upstream_remote(&reference).ok();
	if let Some(remote_name) = remote_name {
		Ok(Some(bytes2string(remote_name.as_ref())?))
	} else {
		Ok(None)
	}
}

/// Retrieve the upstream merge of a local `branch`,
/// configured in "branch.*.merge"
///
/// For details check git2 `branch_upstream_merge`
pub fn get_branch_upstream_merge(
	repo_path: &RepoPath,
	branch: &str,
) -> Result<Option<String>> {
	let repo = repo(repo_path)?;
	let branch = repo.find_branch(branch, BranchType::Local)?;
	let reference = bytes2string(branch.get().name_bytes())?;
	let remote_name = repo.branch_upstream_merge(&reference).ok();
	if let Some(remote_name) = remote_name {
		Ok(Some(bytes2string(remote_name.as_ref())?))
	} else {
		Ok(None)
	}
}

/// returns whether the pull merge strategy is set to rebase
pub fn config_is_pull_rebase(repo_path: &RepoPath) -> Result<bool> {
	let repo = repo(repo_path)?;
	let config = repo.config()?;

	if let Ok(rebase) = config.get_entry("pull.rebase") {
		let value =
			rebase.value().map(String::from).unwrap_or_default();
		return Ok(value == "true");
	}

	Ok(false)
}

///
pub fn branch_compare_upstream(
	repo_path: &RepoPath,
	branch: &str,
) -> Result<BranchCompare> {
	scope_time!("branch_compare_upstream");

	let repo = repo(repo_path)?;

	let branch = repo.find_branch(branch, BranchType::Local)?;

	let upstream = branch.upstream()?;

	let branch_commit =
		branch.into_reference().peel_to_commit()?.id();

	let upstream_commit =
		upstream.into_reference().peel_to_commit()?.id();

	let (ahead, behind) =
		repo.graph_ahead_behind(branch_commit, upstream_commit)?;

	Ok(BranchCompare { ahead, behind })
}

/// Switch branch to given `branch_name`.
///
/// Method will fail if there are conflicting changes between current and target branch. However,
/// if files are not conflicting, they will remain in tree (e.g. tracked new file is not
/// conflicting and therefore is kept in tree even after checkout).
pub fn checkout_branch(
	repo_path: &RepoPath,
	branch_name: &str,
) -> Result<()> {
	scope_time!("checkout_branch");

	let repo = repo(repo_path)?;

	let branch = repo.find_branch(branch_name, BranchType::Local)?;

	let branch_ref = branch.into_reference();

	let target_treeish = branch_ref.peel_to_tree()?;
	let target_treeish_object = target_treeish.as_object();

	// modify state to match branch's state
	repo.checkout_tree(
		target_treeish_object,
		Some(&mut git2::build::CheckoutBuilder::new()),
	)?;

	let branch_ref = branch_ref.name().map_err(|_| {
		Error::Generic(String::from("branch ref not found"))
	});

	// modify HEAD to point to given branch
	repo.set_head(branch_ref?)?;

	Ok(())
}

/// Detach HEAD to point to a commit then checkout HEAD, does not work if there are uncommitted changes
pub fn checkout_commit(
	repo_path: &RepoPath,
	commit_hash: CommitId,
) -> Result<()> {
	scope_time!("checkout_commit");

	let repo = repo(repo_path)?;
	let cur_ref = repo.head()?;
	let statuses = repo.statuses(Some(
		git2::StatusOptions::new().include_ignored(false),
	))?;

	if statuses.is_empty() {
		repo.set_head_detached(commit_hash.into())?;

		if let Err(e) = repo.checkout_head(Some(
			git2::build::CheckoutBuilder::new().force(),
		)) {
			repo.set_head(
				bytes2string(cur_ref.name_bytes())?.as_str(),
			)?;
			return Err(Error::Git(e));
		}
		Ok(())
	} else {
		Err(Error::UncommittedChanges)
	}
}

///
pub fn checkout_remote_branch(
	repo_path: &RepoPath,
	branch: &BranchInfo,
) -> Result<()> {
	scope_time!("checkout_remote_branch");

	let repo = repo(repo_path)?;
	let cur_ref = repo.head()?;

	if !repo
		.statuses(Some(
			git2::StatusOptions::new().include_ignored(false),
		))?
		.is_empty()
	{
		return Err(Error::UncommittedChanges);
	}

	let name = branch.name.find('/').map_or_else(
		|| branch.name.clone(),
		|pos| branch.name[pos..].to_string(),
	);

	let commit = repo.find_commit(branch.top_commit.into())?;
	let mut new_branch = repo.branch(&name, &commit, false)?;
	new_branch.set_upstream(Some(&branch.name))?;

	repo.set_head(
		bytes2string(new_branch.into_reference().name_bytes())?
			.as_str(),
	)?;

	if let Err(e) = repo.checkout_head(Some(
		git2::build::CheckoutBuilder::new().force(),
	)) {
		// This is safe because cur_ref was just found
		repo.set_head(bytes2string(cur_ref.name_bytes())?.as_str())?;
		return Err(Error::Git(e));
	}
	Ok(())
}

fn local_name_for_remote(
	repo: &Repository,
	branch: &BranchInfo,
) -> String {
	if let Ok(remote_buf) = repo.branch_remote_name(&branch.reference)
	{
		if let Ok(remote_name) = bytes2string(remote_buf.as_ref()) {
			let prefix = format!("{remote_name}/");
			if let Some(rest) = branch.name.strip_prefix(&prefix) {
				return rest.to_string();
			}
		}
	}

	branch.name.split_once('/').map_or_else(
		|| branch.name.clone(),
		|(_, rest)| rest.to_string(),
	)
}

fn untracked_path_collides_with_tree(
	target_tree: &git2::Tree<'_>,
	rel_path: &std::path::Path,
) -> bool {
	if target_tree.get_path(rel_path).is_ok() {
		return true;
	}

	let mut cur = rel_path.parent();
	while let Some(parent) = cur {
		if parent.as_os_str().is_empty() {
			break;
		}
		if let Ok(entry) = target_tree.get_path(parent) {
			if entry.kind() != Some(git2::ObjectType::Tree) {
				return true;
			}
		}
		cur = parent.parent();
	}

	false
}

fn worktree_matches_tree_entry(
	full: &std::path::Path,
	entry: &git2::TreeEntry<'_>,
) -> Result<bool> {
	match entry.filemode() {
		0o100_644 | 0o100_755 => {
			let meta = std::fs::symlink_metadata(full)?;
			if meta.file_type().is_dir()
				|| meta.file_type().is_symlink()
			{
				return Ok(false);
			}
			// Compare raw on-disk bytes against the HEAD blob without
			// applying content filters (e.g. autocrlf): if raw bytes
			// differ only by a filter, treat as a collision (safe side).
			let bytes = std::fs::read(full)?;
			let oid = git2::Oid::hash_object(
				git2::ObjectType::Blob,
				&bytes,
			)?;
			Ok(oid == entry.id())
		}
		0o120_000 => {
			let meta = std::fs::symlink_metadata(full)?;
			if !meta.file_type().is_symlink() {
				return Ok(false);
			}
			let target = std::fs::read_link(full)?;
			let oid = git2::Oid::hash_object(
				git2::ObjectType::Blob,
				target.as_os_str().as_encoded_bytes(),
			)?;
			Ok(oid == entry.id())
		}
		_ => Ok(false),
	}
}

fn find_untracked_collisions(
	repo: &Repository,
	head_tree: &git2::Tree<'_>,
	target_tree: &git2::Tree<'_>,
) -> Result<Vec<String>> {
	let mut opts = git2::StatusOptions::new();
	opts.include_untracked(true)
		.recurse_untracked_dirs(true)
		.include_ignored(false);

	let statuses = repo.statuses(Some(&mut opts))?;
	let mut collisions = Vec::new();

	for entry in statuses.iter() {
		let status = entry.status();
		if !status.is_wt_new() {
			continue;
		}

		let path = bytes2string(entry.path_bytes())?;
		let rel_path = std::path::Path::new(&path);

		if let Ok(head_entry) = head_tree.get_path(rel_path) {
			if let Some(workdir) = repo.workdir() {
				if worktree_matches_tree_entry(
					&workdir.join(rel_path),
					&head_entry,
				)? {
					continue;
				}
			}
			collisions.push(path);
			continue;
		}

		if untracked_path_collides_with_tree(target_tree, rel_path) {
			collisions.push(path);
		}
	}

	collisions.sort();
	Ok(collisions)
}

fn should_cleanup_state_after_discard(
	repo: &Repository,
) -> Result<bool> {
	match repo.state() {
		git2::RepositoryState::Rebase
		| git2::RepositoryState::RebaseInteractive
		| git2::RepositoryState::RebaseMerge
		| git2::RepositoryState::ApplyMailbox
		| git2::RepositoryState::ApplyMailboxOrRebase => {
			Err(Error::Generic(String::from(
				"a rebase is in progress: abort it first",
			)))
		}
		git2::RepositoryState::Merge
		| git2::RepositoryState::Revert
		| git2::RepositoryState::RevertSequence
		| git2::RepositoryState::CherryPick
		| git2::RepositoryState::CherryPickSequence => Ok(true),
		_ => Ok(false),
	}
}

fn head_points_to(other_repo: &Repository, ref_name: &str) -> bool {
	other_repo.find_reference("HEAD").is_ok_and(|head| {
		head.symbolic_target_bytes() == Some(ref_name.as_bytes())
	})
}

fn check_branch_not_checked_out_elsewhere(
	repo: &Repository,
	branch_name: &str,
	ref_name: &str,
) -> Result<()> {
	let self_gitdir = std::fs::canonicalize(repo.path())
		.unwrap_or_else(|_| repo.path().to_path_buf());

	if repo.is_worktree() {
		if let Ok(main_repo) = Repository::open(repo.commondir()) {
			let main_gitdir = std::fs::canonicalize(main_repo.path())
				.unwrap_or_else(|_| main_repo.path().to_path_buf());
			if !main_repo.is_bare()
				&& main_gitdir != self_gitdir
				&& head_points_to(&main_repo, ref_name)
			{
				let wt_path = main_repo
					.workdir()
					.unwrap_or_else(|| main_repo.path());
				return Err(Error::Generic(format!(
					"branch '{branch_name}' is checked out at {}",
					wt_path.display()
				)));
			}
		}
	}

	let worktrees = repo.worktrees()?;
	for wt_name in &worktrees {
		let Ok(Some(wt_name)) = wt_name else {
			continue;
		};
		let Ok(wt) = repo.find_worktree(wt_name) else {
			continue;
		};
		let Ok(wt_repo) = Repository::open_from_worktree(&wt) else {
			continue;
		};
		let wt_gitdir = std::fs::canonicalize(wt_repo.path())
			.unwrap_or_else(|_| wt_repo.path().to_path_buf());
		if wt_gitdir == self_gitdir {
			continue;
		}
		if head_points_to(&wt_repo, ref_name) {
			let wt_path =
				wt_repo.workdir().unwrap_or_else(|| wt.path());
			return Err(Error::Generic(format!(
				"branch '{branch_name}' is checked out at {}",
				wt_path.display()
			)));
		}
	}

	Ok(())
}

enum DiscardTarget<'repo> {
	Local {
		ref_name: String,
		tree: git2::Tree<'repo>,
	},
	Remote {
		local_name: String,
		upstream_name: String,
		commit: git2::Commit<'repo>,
		tree: git2::Tree<'repo>,
	},
}

impl<'repo> DiscardTarget<'repo> {
	const fn tree(&self) -> &git2::Tree<'repo> {
		match self {
			Self::Local { tree, .. } | Self::Remote { tree, .. } => {
				tree
			}
		}
	}
}

fn resolve_discard_target<'repo>(
	repo: &'repo Repository,
	branch: &BranchInfo,
) -> Result<DiscardTarget<'repo>> {
	if branch.is_local() {
		let target_branch =
			repo.find_branch(&branch.name, BranchType::Local)?;
		let target_ref = target_branch.into_reference();
		let ref_name = bytes2string(target_ref.name_bytes())?;
		let tree = target_ref.peel_to_tree()?;
		return Ok(DiscardTarget::Local { ref_name, tree });
	}

	let remote_branch =
		repo.find_branch(&branch.name, BranchType::Remote)?;
	let commit = remote_branch.into_reference().peel_to_commit()?;
	let tree = commit.tree()?;
	let local_name = local_name_for_remote(repo, branch);

	if !Branch::name_is_valid(&local_name)? {
		return Err(Error::Generic(format!(
			"invalid branch name: {local_name}"
		)));
	}

	let target_ref = format!("refs/heads/{local_name}");
	for existing in repo.branches(Some(BranchType::Local))? {
		let (existing_branch, _) = existing?;
		if let Ok(existing_ref) =
			bytes2string(existing_branch.get().name_bytes())
		{
			if existing_ref == target_ref
				|| existing_ref
					.strip_prefix(&target_ref)
					.is_some_and(|rest| rest.starts_with('/'))
				|| target_ref
					.strip_prefix(existing_ref.as_str())
					.is_some_and(|rest| rest.starts_with('/'))
			{
				return Err(Error::Generic(format!(
					"branch '{local_name}' already exists"
				)));
			}
		}
	}

	Ok(DiscardTarget::Remote {
		local_name,
		upstream_name: branch.name.clone(),
		commit,
		tree,
	})
}

/// Outcome of [`checkout_branch_discard`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiscardOutcome {
	/// Switch completed and any pending merge/revert/cherry-pick state was cleared.
	Switched,
	/// `HEAD` moved to the target branch, but clearing merge state failed.
	StateNotCleared(String),
}

#[cfg(any(test, feature = "test-hooks"))]
thread_local! {
	static FAIL_CLEANUP_STATE: std::cell::Cell<bool> =
		const { std::cell::Cell::new(false) };
}

/// Test-only hook to force `cleanup_repo_state` to fail on the
/// current thread.
#[cfg(feature = "test-hooks")]
#[doc(hidden)]
pub fn set_fail_cleanup_state(fail: bool) {
	FAIL_CLEANUP_STATE.set(fail);
}

fn cleanup_repo_state(repo: &Repository) -> Result<()> {
	#[cfg(any(test, feature = "test-hooks"))]
	if FAIL_CLEANUP_STATE.get() {
		return Err(Error::Generic(String::from(
			"simulated cleanup_state failure",
		)));
	}
	repo.cleanup_state()?;
	Ok(())
}

fn apply_discard_checkout(
	repo: &Repository,
	target_tree: &git2::Tree<'_>,
	target_ref_name: &str,
	target_branch_name: &str,
	cleanup_state: bool,
) -> Result<DiscardOutcome> {
	let mut checkout_opts = git2::build::CheckoutBuilder::new();
	checkout_opts.force().overwrite_ignored(true);
	repo.checkout_tree(
		target_tree.as_object(),
		Some(&mut checkout_opts),
	)?;

	let mut index = repo.index()?;
	index.read_tree(target_tree)?;
	index.write()?;

	repo.set_head(target_ref_name)?;

	if cleanup_state {
		if let Err(err) = cleanup_repo_state(repo) {
			return Ok(DiscardOutcome::StateNotCleared(format!(
				"switched to '{target_branch_name}', but could not clear the merge state: {err}"
			)));
		}
	}

	Ok(DiscardOutcome::Switched)
}

/// Discard all staged and unstaged tracked changes and switch to `branch`.
///
/// Validates repository state, HEAD, the target branch (including linked
/// worktrees and local branch creation + upstream setup for remote targets),
/// and checks for un-indexed worktree files that would be overwritten or
/// removed before mutating the worktree, index, HEAD, or pending merge state.
pub fn checkout_branch_discard(
	repo_path: &RepoPath,
	branch: &BranchInfo,
) -> Result<DiscardOutcome> {
	scope_time!("checkout_branch_discard");

	let repo = repo(repo_path)?;

	let cleanup_state = should_cleanup_state_after_discard(&repo)?;

	let head = repo.head().map_err(|e| {
		if e.code() == git2::ErrorCode::UnbornBranch {
			Error::NoHead
		} else {
			e.into()
		}
	})?;
	let head_tree = head.peel_to_tree()?;

	let target = resolve_discard_target(&repo, branch)?;

	if let DiscardTarget::Local { ref ref_name, .. } = target {
		check_branch_not_checked_out_elsewhere(
			&repo,
			&branch.name,
			ref_name,
		)?;
	}

	let collisions =
		find_untracked_collisions(&repo, &head_tree, target.tree())?;
	if !collisions.is_empty() {
		return Err(Error::Generic(format!(
			"untracked working tree file would be overwritten by checkout: {}",
			collisions.join(", ")
		)));
	}

	let (target_ref_name, target_branch_name, created_branch) =
		match &target {
			DiscardTarget::Local { ref_name, .. } => {
				(ref_name.clone(), branch.name.as_str(), None)
			}
			DiscardTarget::Remote {
				local_name,
				upstream_name,
				commit,
				..
			} => {
				let mut new_branch =
					repo.branch(local_name, commit, false)?;
				if let Err(e) =
					new_branch.set_upstream(Some(upstream_name))
				{
					let _ = new_branch.delete();
					return Err(Error::Generic(format!(
						"could not set upstream '{upstream_name}' for '{local_name}': {e}"
					)));
				}
				let ref_name =
					match bytes2string(new_branch.get().name_bytes())
					{
						Ok(name) => name,
						Err(e) => {
							let _ = new_branch.delete();
							return Err(e);
						}
					};
				(ref_name, local_name.as_str(), Some(new_branch))
			}
		};

	let switch_result = apply_discard_checkout(
		&repo,
		target.tree(),
		&target_ref_name,
		target_branch_name,
		cleanup_state,
	);

	match switch_result {
		Ok(outcome) => Ok(outcome),
		Err(err) => {
			if let Some(mut new_branch) = created_branch {
				let _ = new_branch.delete();
			}
			Err(err)
		}
	}
}

/// The user must not be on the branch for the branch to be deleted
pub fn delete_branch(
	repo_path: &RepoPath,
	branch_ref: &str,
) -> Result<()> {
	scope_time!("delete_branch");

	let repo = repo(repo_path)?;
	let branch_as_ref = repo.find_reference(branch_ref)?;
	let mut branch = git2::Branch::wrap(branch_as_ref);
	if branch.is_head() {
		return Err(Error::Generic("You cannot be on the branch you want to delete, switch branch, then delete this branch".to_string()));
	}
	branch.delete()?;
	Ok(())
}

/// creates a new branch pointing to current HEAD commit and updating HEAD to new branch
pub fn create_branch(
	repo_path: &RepoPath,
	name: &str,
) -> Result<String> {
	scope_time!("create_branch");

	let repo = repo(repo_path)?;

	let head_id = get_head_repo(&repo)?;
	let head_commit = repo.find_commit(head_id.into())?;

	let branch = repo.branch(name, &head_commit, false)?;
	let branch_ref = branch.into_reference();
	let branch_ref_name = bytes2string(branch_ref.name_bytes())?;
	repo.set_head(branch_ref_name.as_str())?;

	Ok(branch_ref_name)
}

#[cfg(test)]
mod tests_branch_name {
	use super::*;
	use crate::sync::tests::{repo_init, repo_init_empty};

	#[test]
	fn test_smoke() {
		let (_td, repo) = repo_init().unwrap();
		let root = repo.path().parent().unwrap();
		let repo_path: &RepoPath =
			&root.as_os_str().to_str().unwrap().into();

		assert_eq!(
			get_branch_name(repo_path).unwrap().as_str(),
			"master"
		);
	}

	#[test]
	fn test_empty_repo() {
		let (_td, repo) = repo_init_empty().unwrap();
		let root = repo.path().parent().unwrap();
		let repo_path: &RepoPath =
			&root.as_os_str().to_str().unwrap().into();

		assert!(matches!(
			get_branch_name(repo_path),
			Err(Error::NoHead)
		));
	}
}

#[cfg(test)]
mod tests_create_branch {
	use super::*;
	use crate::sync::tests::repo_init;

	#[test]
	fn test_smoke() {
		let (_td, repo) = repo_init().unwrap();
		let root = repo.path().parent().unwrap();
		let repo_path: &RepoPath =
			&root.as_os_str().to_str().unwrap().into();

		create_branch(repo_path, "branch1").unwrap();

		assert_eq!(
			get_branch_name(repo_path).unwrap().as_str(),
			"branch1"
		);
	}
}

#[cfg(test)]
mod tests_branch_compare {
	use super::*;
	use crate::sync::tests::repo_init;

	#[test]
	fn test_smoke() {
		let (_td, repo) = repo_init().unwrap();
		let root = repo.path().parent().unwrap();
		let repo_path: &RepoPath =
			&root.as_os_str().to_str().unwrap().into();

		create_branch(repo_path, "test").unwrap();

		let res = branch_compare_upstream(repo_path, "test");

		assert!(res.is_err());
	}
}

#[cfg(test)]
mod tests_branches {
	use super::*;
	use crate::sync::{
		remotes::{get_remotes, push::push_branch},
		rename_branch,
		tests::{
			debug_cmd_print, repo_clone, repo_init, repo_init_bare,
			write_commit_file,
		},
	};

	#[test]
	fn test_smoke() {
		let (_td, repo) = repo_init().unwrap();
		let root = repo.path().parent().unwrap();
		let repo_path: &RepoPath =
			&root.as_os_str().to_str().unwrap().into();

		assert_eq!(
			get_branches_info(repo_path, true)
				.unwrap()
				.iter()
				.map(|b| b.name.clone())
				.collect::<Vec<_>>(),
			vec!["master"]
		);
	}

	#[test]
	fn test_multiple() {
		let (_td, repo) = repo_init().unwrap();
		let root = repo.path().parent().unwrap();
		let repo_path: &RepoPath =
			&root.as_os_str().to_str().unwrap().into();

		create_branch(repo_path, "test").unwrap();

		assert_eq!(
			get_branches_info(repo_path, true)
				.unwrap()
				.iter()
				.map(|b| b.name.clone())
				.collect::<Vec<_>>(),
			vec!["master", "test"]
		);
	}

	fn clone_branch_commit_push(target: &str, branch_name: &str) {
		let (dir, repo) = repo_clone(target).unwrap();
		let dir = dir.path().to_str().unwrap();

		write_commit_file(&repo, "f1.txt", "foo", "c1");
		rename_branch(&dir.into(), "refs/heads/master", branch_name)
			.unwrap();
		push_branch(
			&dir.into(),
			"origin",
			branch_name,
			false,
			false,
			None,
			None,
		)
		.unwrap();
	}

	#[test]
	fn test_remotes_of_branches() {
		let (r1_path, _remote1) = repo_init_bare().unwrap();
		let (r2_path, _remote2) = repo_init_bare().unwrap();
		let (_r, repo) = repo_init().unwrap();

		let r1_path = r1_path.path().to_str().unwrap();
		let r2_path = r2_path.path().to_str().unwrap();

		//Note: create those test branches in our remotes
		clone_branch_commit_push(r1_path, "r1branch");
		clone_branch_commit_push(r2_path, "r2branch");

		let root = repo.path().parent().unwrap();
		let repo_path: &RepoPath =
			&root.as_os_str().to_str().unwrap().into();

		//add the remotes
		repo.remote("r1", r1_path).unwrap();
		repo.remote("r2", r2_path).unwrap();

		//verify we got the remotes
		let remotes = get_remotes(repo_path).unwrap();
		assert_eq!(
			remotes,
			vec![String::from("r1"), String::from("r2")]
		);

		//verify we got only master right now
		let branches = get_branches_info(repo_path, true).unwrap();
		assert_eq!(branches.len(), 1);
		assert_eq!(branches[0].name, String::from("master"));

		//pull stuff from the two remotes
		debug_cmd_print(repo_path, "git pull r1");
		debug_cmd_print(repo_path, "git pull r2");

		//create local tracking branches
		debug_cmd_print(
			repo_path,
			"git checkout --track r1/r1branch",
		);
		debug_cmd_print(
			repo_path,
			"git checkout --track r2/r2branch",
		);

		let branches = get_branches_info(repo_path, true).unwrap();
		assert_eq!(branches.len(), 3);
		assert_eq!(
			branches[1]
				.local_details()
				.unwrap()
				.remote
				.as_ref()
				.unwrap(),
			"r1"
		);
		assert_eq!(
			branches[2]
				.local_details()
				.unwrap()
				.remote
				.as_ref()
				.unwrap(),
			"r2"
		);

		assert_eq!(
			get_branch_remote(repo_path, "r1branch")
				.unwrap()
				.unwrap(),
			String::from("r1")
		);

		assert_eq!(
			get_branch_remote(repo_path, "r2branch")
				.unwrap()
				.unwrap(),
			String::from("r2")
		);
	}

	#[test]
	fn test_branch_remote_no_upstream() {
		let (_r, repo) = repo_init().unwrap();
		let root = repo.path().parent().unwrap();
		let repo_path: &RepoPath =
			&root.as_os_str().to_str().unwrap().into();

		assert_eq!(
			get_branch_remote(repo_path, "master").unwrap(),
			None
		);
	}

	#[test]
	fn test_branch_remote_no_branch() {
		let (_r, repo) = repo_init().unwrap();
		let root = repo.path().parent().unwrap();
		let repo_path: &RepoPath =
			&root.as_os_str().to_str().unwrap().into();

		assert!(get_branch_remote(repo_path, "foo").is_err());
	}

	#[test]
	fn test_branch_no_upstream_merge_config() {
		let (_r, repo) = repo_init().unwrap();
		let root = repo.path().parent().unwrap();
		let repo_path: &RepoPath =
			&root.as_os_str().to_str().unwrap().into();

		let upstream_merge_res =
			get_branch_upstream_merge(repo_path, "master");
		assert!(
			upstream_merge_res.is_ok_and(|v| v.as_ref().is_none())
		);
	}

	#[test]
	fn test_branch_with_upstream_merge_config() {
		let (_r, repo) = repo_init().unwrap();
		let root = repo.path().parent().unwrap();
		let repo_path: &RepoPath =
			&root.as_os_str().to_str().unwrap().into();

		let branch_name = "master";
		let upstream_merge = "refs/heads/master";

		let mut config = repo.config().unwrap();
		config
			.set_str(
				&format!("branch.{branch_name}.merge"),
				upstream_merge,
			)
			.expect("fail set branch merge config");

		let upstream_merge_res =
			get_branch_upstream_merge(repo_path, branch_name);
		assert!(upstream_merge_res
			.as_ref()
			.is_ok_and(|v| v.as_ref().is_some()));
		assert_eq!(
			&upstream_merge_res.unwrap().unwrap(),
			upstream_merge
		);
	}
}

#[cfg(test)]
mod tests_checkout {
	use super::*;
	use crate::sync::{stage_add_file, tests::repo_init};
	use std::{fs::File, path::Path};

	#[test]
	fn test_smoke() {
		let (_td, repo) = repo_init().unwrap();
		let root = repo.path().parent().unwrap();
		let repo_path: &RepoPath =
			&root.as_os_str().to_str().unwrap().into();

		assert!(checkout_branch(repo_path, "master").is_ok());
		assert!(checkout_branch(repo_path, "foobar").is_err());
	}

	#[test]
	fn test_multiple() {
		let (_td, repo) = repo_init().unwrap();
		let root = repo.path().parent().unwrap();
		let repo_path: &RepoPath =
			&root.as_os_str().to_str().unwrap().into();

		create_branch(repo_path, "test").unwrap();

		assert!(checkout_branch(repo_path, "test").is_ok());
		assert!(checkout_branch(repo_path, "master").is_ok());
		assert!(checkout_branch(repo_path, "test").is_ok());
	}

	#[test]
	fn test_branch_with_slash_in_name() {
		let (_td, repo) = repo_init().unwrap();
		let root = repo.path().parent().unwrap();
		let repo_path: &RepoPath =
			&root.as_os_str().to_str().unwrap().into();

		create_branch(repo_path, "foo/bar").unwrap();
		checkout_branch(repo_path, "foo/bar").unwrap();
	}

	#[test]
	fn test_staged_new_file() {
		let (_td, repo) = repo_init().unwrap();
		let root = repo.path().parent().unwrap();
		let repo_path: &RepoPath =
			&root.as_os_str().to_str().unwrap().into();

		create_branch(repo_path, "test").unwrap();

		let filename = "file.txt";
		let file = root.join(filename);
		File::create(&file).unwrap();

		stage_add_file(repo_path, Path::new(filename)).unwrap();

		assert!(checkout_branch(repo_path, "test").is_ok());
	}
}

#[cfg(test)]
mod tests_checkout_commit {
	use super::*;
	use crate::sync::tests::{repo_init, write_commit_file};
	use crate::sync::RepoPath;

	#[test]
	fn test_smoke() {
		let (_td, repo) = repo_init().unwrap();
		let root = repo.path().parent().unwrap();
		let repo_path: &RepoPath =
			&root.as_os_str().to_str().unwrap().into();

		let commit =
			write_commit_file(&repo, "test_1.txt", "test", "commit1");
		write_commit_file(&repo, "test_2.txt", "test", "commit2");

		checkout_commit(repo_path, commit).unwrap();

		assert!(repo.head_detached().unwrap());
		assert_eq!(
			repo.head().unwrap().target().unwrap(),
			commit.get_oid()
		);
	}
}

#[cfg(test)]
mod test_delete_branch {
	use super::*;
	use crate::sync::tests::repo_init;

	#[test]
	fn test_delete_branch() {
		let (_td, repo) = repo_init().unwrap();
		let root = repo.path().parent().unwrap();
		let repo_path: &RepoPath =
			&root.as_os_str().to_str().unwrap().into();

		create_branch(repo_path, "branch1").unwrap();
		create_branch(repo_path, "branch2").unwrap();

		checkout_branch(repo_path, "branch1").unwrap();

		assert_eq!(
			repo.branches(None)
				.unwrap()
				.nth(1)
				.unwrap()
				.unwrap()
				.0
				.name()
				.unwrap()
				.unwrap(),
			"branch2"
		);

		delete_branch(repo_path, "refs/heads/branch2").unwrap();

		assert_eq!(
			repo.branches(None)
				.unwrap()
				.nth(1)
				.unwrap()
				.unwrap()
				.0
				.name()
				.unwrap()
				.unwrap(),
			"master"
		);
	}
}

#[cfg(test)]
mod test_remote_branches {
	use super::*;
	use crate::sync::remotes::push::push_branch;
	use crate::sync::tests::{
		repo_clone, repo_init_bare, write_commit_file,
	};

	impl BranchInfo {
		/// returns details about remote branch or None
		const fn remote_details(&self) -> Option<&RemoteBranch> {
			if let BranchDetails::Remote(details) = &self.details {
				Some(details)
			} else {
				None
			}
		}
	}

	#[test]
	fn test_remote_branches() {
		let (r1_dir, _repo) = repo_init_bare().unwrap();

		let (clone1_dir, clone1) =
			repo_clone(r1_dir.path().to_str().unwrap()).unwrap();

		let clone1_dir = clone1_dir.path().to_str().unwrap();

		// clone1

		write_commit_file(&clone1, "test.txt", "test", "commit1");

		push_branch(
			&clone1_dir.into(),
			"origin",
			"master",
			false,
			false,
			None,
			None,
		)
		.unwrap();

		create_branch(&clone1_dir.into(), "foo").unwrap();

		write_commit_file(&clone1, "test.txt", "test2", "commit2");

		push_branch(
			&clone1_dir.into(),
			"origin",
			"foo",
			false,
			false,
			None,
			None,
		)
		.unwrap();

		// clone2

		let (clone2_dir, _clone2) =
			repo_clone(r1_dir.path().to_str().unwrap()).unwrap();

		let clone2_dir = clone2_dir.path().to_str().unwrap();

		let local_branches =
			get_branches_info(&clone2_dir.into(), true).unwrap();

		assert_eq!(local_branches.len(), 1);

		let branches =
			get_branches_info(&clone2_dir.into(), false).unwrap();
		assert_eq!(dbg!(&branches).len(), 3);
		assert_eq!(&branches[0].name, "origin/HEAD");
		assert_eq!(&branches[1].name, "origin/foo");
		assert_eq!(&branches[2].name, "origin/master");
	}

	#[test]
	fn test_checkout_remote_branch() {
		let (r1_dir, _repo) = repo_init_bare().unwrap();

		let (clone1_dir, clone1) =
			repo_clone(r1_dir.path().to_str().unwrap()).unwrap();
		let clone1_dir = clone1_dir.path().to_str().unwrap();

		// clone1

		write_commit_file(&clone1, "test.txt", "test", "commit1");
		push_branch(
			&clone1_dir.into(),
			"origin",
			"master",
			false,
			false,
			None,
			None,
		)
		.unwrap();
		create_branch(&clone1_dir.into(), "foo").unwrap();
		write_commit_file(&clone1, "test.txt", "test2", "commit2");
		push_branch(
			&clone1_dir.into(),
			"origin",
			"foo",
			false,
			false,
			None,
			None,
		)
		.unwrap();

		// clone2

		let (clone2_dir, _clone2) =
			repo_clone(r1_dir.path().to_str().unwrap()).unwrap();

		let clone2_dir = clone2_dir.path().to_str().unwrap();

		let local_branches =
			get_branches_info(&clone2_dir.into(), true).unwrap();

		assert_eq!(local_branches.len(), 1);

		let branches =
			get_branches_info(&clone2_dir.into(), false).unwrap();

		// checkout origin/foo
		checkout_remote_branch(&clone2_dir.into(), &branches[1])
			.unwrap();

		assert_eq!(
			get_branches_info(&clone2_dir.into(), true)
				.unwrap()
				.len(),
			2
		);

		assert_eq!(
			&get_branch_name(&clone2_dir.into()).unwrap(),
			"foo"
		);
	}

	#[test]
	fn test_checkout_remote_branch_hierarchical() {
		let (r1_dir, _repo) = repo_init_bare().unwrap();

		let (clone1_dir, clone1) =
			repo_clone(r1_dir.path().to_str().unwrap()).unwrap();
		let clone1_dir = clone1_dir.path().to_str().unwrap();

		// clone1

		let branch_name = "bar/foo";

		write_commit_file(&clone1, "test.txt", "test", "commit1");
		push_branch(
			&clone1_dir.into(),
			"origin",
			"master",
			false,
			false,
			None,
			None,
		)
		.unwrap();
		create_branch(&clone1_dir.into(), branch_name).unwrap();
		write_commit_file(&clone1, "test.txt", "test2", "commit2");
		push_branch(
			&clone1_dir.into(),
			"origin",
			branch_name,
			false,
			false,
			None,
			None,
		)
		.unwrap();

		// clone2

		let (clone2_dir, _clone2) =
			repo_clone(r1_dir.path().to_str().unwrap()).unwrap();
		let clone2_dir = clone2_dir.path().to_str().unwrap();

		let branches =
			get_branches_info(&clone2_dir.into(), false).unwrap();

		checkout_remote_branch(&clone2_dir.into(), &branches[1])
			.unwrap();

		assert_eq!(
			&get_branch_name(&clone2_dir.into()).unwrap(),
			branch_name
		);
	}

	#[test]
	fn test_has_tracking() {
		let (r1_dir, _repo) = repo_init_bare().unwrap();

		let (clone1_dir, clone1) =
			repo_clone(r1_dir.path().to_str().unwrap()).unwrap();
		let clone1_dir = clone1_dir.path().to_str().unwrap();

		// clone1

		write_commit_file(&clone1, "test.txt", "test", "commit1");
		push_branch(
			&clone1_dir.into(),
			"origin",
			"master",
			false,
			false,
			None,
			None,
		)
		.unwrap();
		create_branch(&clone1_dir.into(), "foo").unwrap();
		write_commit_file(&clone1, "test.txt", "test2", "commit2");
		push_branch(
			&clone1_dir.into(),
			"origin",
			"foo",
			false,
			false,
			None,
			None,
		)
		.unwrap();

		let branches_1 =
			get_branches_info(&clone1_dir.into(), false).unwrap();

		assert!(branches_1[0].remote_details().unwrap().has_tracking);
		assert!(branches_1[1].remote_details().unwrap().has_tracking);

		// clone2

		let (clone2_dir, _clone2) =
			repo_clone(r1_dir.path().to_str().unwrap()).unwrap();

		let clone2_dir = clone2_dir.path().to_str().unwrap();

		let branches_2 =
			get_branches_info(&clone2_dir.into(), false).unwrap();

		assert!(
			!branches_2[0].remote_details().unwrap().has_tracking
		);
		assert!(
			!branches_2[1].remote_details().unwrap().has_tracking
		);
		assert!(branches_2[2].remote_details().unwrap().has_tracking);
	}

	#[test]
	fn t10c_cleanup_failure_keeps_switch() {
		struct FailCleanupGuard;
		impl Drop for FailCleanupGuard {
			fn drop(&mut self) {
				FAIL_CLEANUP_STATE.set(false);
			}
		}

		let (r1_dir, _bare) = repo_init_bare().unwrap();
		let (clone1_dir, clone1) =
			repo_clone(r1_dir.path().to_str().unwrap()).unwrap();
		let clone1_path: RepoPath =
			clone1_dir.path().to_str().unwrap().into();

		write_commit_file(&clone1, "test.txt", "base", "commit1");
		push_branch(
			&clone1_path,
			"origin",
			"master",
			false,
			false,
			None,
			None,
		)
		.unwrap();

		create_branch(&clone1_path, "topic").unwrap();
		write_commit_file(
			&clone1,
			"test.txt",
			"topic_content",
			"commit2",
		);
		push_branch(
			&clone1_path,
			"origin",
			"topic",
			false,
			false,
			None,
			None,
		)
		.unwrap();

		let (clone2_dir, clone2) =
			repo_clone(r1_dir.path().to_str().unwrap()).unwrap();
		let clone2_path: RepoPath =
			clone2_dir.path().to_str().unwrap().into();

		// Create a conflicted merge in progress in clone2
		create_branch(&clone2_path, "side").unwrap();
		write_commit_file(&clone2, "test.txt", "side", "side");
		checkout_branch(&clone2_path, "master").unwrap();
		write_commit_file(&clone2, "test.txt", "master", "master");
		crate::sync::merge::merge_branch(
			&clone2_path,
			"side",
			BranchType::Local,
		)
		.unwrap();
		assert_eq!(
			crate::sync::repo_state(&clone2_path).unwrap(),
			crate::sync::RepoState::Merge
		);

		let target = get_branches_info(&clone2_path, false)
			.unwrap()
			.into_iter()
			.find(|b| b.name == "origin/topic")
			.unwrap();

		FAIL_CLEANUP_STATE.set(true);
		let _guard = FailCleanupGuard;

		let outcome =
			checkout_branch_discard(&clone2_path, &target).unwrap();
		let DiscardOutcome::StateNotCleared(msg) = outcome else {
			panic!("expected StateNotCleared, got {outcome:?}");
		};
		assert!(
			msg.starts_with(
				"switched to 'topic', but could not clear the merge state: "
			),
			"unexpected message: {msg}"
		);
		assert_eq!(
			clone2.head().unwrap().name().unwrap(),
			"refs/heads/topic"
		);
		let local_topic =
			clone2.find_branch("topic", BranchType::Local).unwrap();
		assert_eq!(
			local_topic.upstream().unwrap().name().unwrap(),
			Some("origin/topic")
		);
		assert_eq!(
			std::fs::read_to_string(
				clone2_dir.path().join("test.txt")
			)
			.unwrap(),
			"topic_content"
		);
	}
}
