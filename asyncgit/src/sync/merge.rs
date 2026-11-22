use crate::{
	error::{Error, Result},
	sync::{
		branch::merge_commit::commit_merge_with_head,
		rebase::{
			abort_rebase, continue_rebase, get_rebase_progress,
		},
		repository::repo,
		CommitId,
	},
};
use git2::{
	build::CheckoutBuilder, BranchType, Commit, MergeOptions,
	ObjectType, Oid, Repository, Tree, TreeWalkMode, TreeWalkResult,
};
use scopetime::scope_time;
use std::{
	collections::{BTreeMap, BTreeSet},
	fs, io,
	path::Path,
};

use super::{
	rebase::{RebaseProgress, RebaseState},
	RepoPath,
};

#[derive(Clone, Copy, PartialEq, Eq)]
struct EntrySpec {
	id: Oid,
	mode: u32,
}

fn bytes_to_path(bytes: &[u8]) -> Result<&Path> {
	if bytes.is_empty() {
		return Err(Error::Generic(
			"empty path in repository state".to_string(),
		));
	}
	#[cfg(unix)]
	{
		use std::os::unix::ffi::OsStrExt;
		Ok(Path::new(std::ffi::OsStr::from_bytes(bytes)))
	}
	#[cfg(not(unix))]
	{
		std::str::from_utf8(bytes).map(Path::new).map_err(|e| {
			Error::Generic(format!(
				"invalid utf-8 path {}: {e}",
				display_path(bytes)
			))
		})
	}
}

fn display_path(bytes: &[u8]) -> String {
	String::from_utf8_lossy(bytes).into_owned()
}

fn is_absent(err: &io::Error) -> bool {
	matches!(
		err.kind(),
		io::ErrorKind::NotFound | io::ErrorKind::NotADirectory
	)
}

fn collect_head_entries(
	head_tree: Option<&Tree<'_>>,
) -> Result<BTreeMap<Vec<u8>, EntrySpec>> {
	let mut entries = BTreeMap::new();
	if let Some(tree) = head_tree {
		tree.walk(TreeWalkMode::PreOrder, |root, entry| {
			if entry.kind() == Some(ObjectType::Tree) {
				return TreeWalkResult::Ok;
			}
			let mut path = root.as_bytes().to_vec();
			path.extend_from_slice(entry.name_bytes());
			if let Ok(mode) = u32::try_from(entry.filemode()) {
				entries.insert(
					path,
					EntrySpec {
						id: entry.id(),
						mode,
					},
				);
			}
			TreeWalkResult::Ok
		})?;
	}
	Ok(entries)
}

fn collect_index_entries(
	index: &git2::Index,
) -> (BTreeMap<Vec<u8>, EntrySpec>, BTreeSet<Vec<u8>>) {
	let mut stage0 = BTreeMap::new();
	let mut conflicted = BTreeSet::new();
	for entry in index.iter() {
		let stage = (entry.flags >> 12) & 0x3;
		if stage == 0 {
			stage0.insert(
				entry.path,
				EntrySpec {
					id: entry.id,
					mode: entry.mode,
				},
			);
		} else {
			conflicted.insert(entry.path);
		}
	}
	(stage0, conflicted)
}

fn worktree_matches_index(
	full: &Path,
	idx_entry: EntrySpec,
) -> Result<bool> {
	let meta = match fs::symlink_metadata(full) {
		Ok(m) => m,
		Err(e) if is_absent(&e) => return Ok(false),
		Err(e) => return Err(e.into()),
	};

	match idx_entry.mode {
		0o100_644 | 0o100_755 => {
			if meta.file_type().is_dir()
				|| meta.file_type().is_symlink()
			{
				return Ok(false);
			}
			let bytes = fs::read(full)?;
			let oid = Oid::hash_object(ObjectType::Blob, &bytes)?;
			Ok(oid == idx_entry.id)
		}
		0o120_000 => {
			if !meta.file_type().is_symlink() {
				return Ok(false);
			}
			let target = fs::read_link(full)?;
			let oid = Oid::hash_object(
				ObjectType::Blob,
				target.as_os_str().as_encoded_bytes(),
			)?;
			Ok(oid == idx_entry.id)
		}
		0o160_000 => Ok(meta.file_type().is_dir()),
		_ => Ok(false),
	}
}

fn collect_unmanaged_in_dir(
	dir: &Path,
	rel_prefix: &[u8],
	to_remove: &BTreeSet<Vec<u8>>,
	blocking: &mut BTreeSet<String>,
) -> Result<()> {
	for entry in fs::read_dir(dir)? {
		let entry = entry?;
		let file_type = entry.file_type()?;
		let name = entry.file_name();
		let mut child_rel = rel_prefix.to_vec();
		child_rel.push(b'/');
		child_rel.extend_from_slice(name.as_encoded_bytes());
		if file_type.is_dir() {
			collect_unmanaged_in_dir(
				&entry.path(),
				&child_rel,
				to_remove,
				blocking,
			)?;
		} else if !to_remove.contains(&child_rel) {
			blocking.insert(display_path(&child_rel));
		}
	}
	Ok(())
}

fn find_blocking_local_changes(
	workdir: &Path,
	non_conflict_changed: &[Vec<u8>],
	stage0: &BTreeMap<Vec<u8>, EntrySpec>,
	to_remove: &BTreeSet<Vec<u8>>,
	to_restore: &[Vec<u8>],
) -> Result<BTreeSet<String>> {
	let mut blocking = BTreeSet::new();

	for path in non_conflict_changed {
		let full = workdir.join(bytes_to_path(path)?);
		if let Some(idx_entry) = stage0.get(path) {
			if !worktree_matches_index(&full, *idx_entry)? {
				blocking.insert(display_path(path));
			}
		} else {
			match fs::symlink_metadata(&full) {
				Err(e) if is_absent(&e) => {}
				Err(e) => return Err(e.into()),
				Ok(meta) if meta.file_type().is_dir() => {
					collect_unmanaged_in_dir(
						&full,
						path,
						to_remove,
						&mut blocking,
					)?;
				}
				Ok(_) => {
					blocking.insert(display_path(path));
				}
			}
		}
	}

	for path in to_restore {
		let mut slice = path.as_slice();
		while let Some(pos) = slice.iter().rposition(|&b| b == b'/') {
			slice = &slice[..pos];
			if to_remove.contains(slice) {
				continue;
			}
			let anc_full = workdir.join(bytes_to_path(slice)?);
			match fs::symlink_metadata(&anc_full) {
				Ok(meta) if !meta.file_type().is_dir() => {
					blocking.insert(display_path(slice));
				}
				Ok(_) => {}
				Err(e) if is_absent(&e) => {}
				Err(e) => return Err(e.into()),
			}
		}
	}

	Ok(blocking)
}

fn prune_empty_parents(path: &Path, workdir: &Path) {
	let mut dir = path.parent();
	while let Some(d) = dir {
		if d == workdir {
			break;
		}
		if fs::remove_dir(d).is_ok() {
			dir = d.parent();
		} else {
			break;
		}
	}
}

///
pub fn mergehead_ids(repo_path: &RepoPath) -> Result<Vec<CommitId>> {
	scope_time!("mergehead_ids");

	let mut repo = repo(repo_path)?;

	let mut ids: Vec<CommitId> = Vec::new();
	repo.mergehead_foreach(|id| {
		ids.push(CommitId::from(*id));
		true
	})?;

	Ok(ids)
}

/// Aborts a pending merge or revert with `git reset --merge` semantics:
/// * refuses without writing if a path changed by the merge/revert has local worktree edits
/// * restores paths changed by the merge/revert back to HEAD (worktree, then index)
/// * leaves untracked files, ignored files, and unrelated unstaged edits untouched
/// * cleans up repo merge/revert state
pub fn abort_pending_state(repo_path: &RepoPath) -> Result<()> {
	scope_time!("abort_pending_state");

	let repo = repo(repo_path)?;
	let workdir = repo.workdir().ok_or(Error::NoWorkDir)?;

	let head_tree = match repo.head() {
		Ok(head) => Some(head.peel_to_tree()?),
		Err(e) if e.code() == git2::ErrorCode::UnbornBranch => None,
		Err(e) => return Err(e.into()),
	};

	let head_entries = collect_head_entries(head_tree.as_ref())?;
	let mut index = repo.index()?;
	let (stage0, conflicted) = collect_index_entries(&index);

	let mut all_paths: BTreeSet<&[u8]> = BTreeSet::new();
	for k in head_entries.keys() {
		all_paths.insert(k.as_slice());
	}
	for k in stage0.keys() {
		all_paths.insert(k.as_slice());
	}
	for k in &conflicted {
		all_paths.insert(k.as_slice());
	}

	let mut non_conflict_changed = Vec::new();
	let mut to_remove = BTreeSet::new();
	let mut to_restore = Vec::new();

	for path in all_paths {
		let in_conflict = conflicted.contains(path);
		let head_entry = head_entries.get(path);
		let idx_entry = stage0.get(path);
		if !in_conflict && head_entry == idx_entry {
			continue;
		}
		if !in_conflict {
			non_conflict_changed.push(path.to_vec());
		}
		if head_entry.is_some() {
			to_restore.push(path.to_vec());
		} else {
			to_remove.insert(path.to_vec());
		}
	}

	let blocking = find_blocking_local_changes(
		workdir,
		&non_conflict_changed,
		&stage0,
		&to_remove,
		&to_restore,
	)?;
	if !blocking.is_empty() {
		let paths =
			blocking.into_iter().collect::<Vec<_>>().join(", ");
		return Err(Error::Generic(format!(
			"cannot abort: local changes to {paths} would be lost"
		)));
	}

	// 1. Worktree first: remove merge-added paths, then restore HEAD paths.
	for path in &to_remove {
		let full = workdir.join(bytes_to_path(path)?);
		match fs::symlink_metadata(&full) {
			Ok(meta) => {
				if meta.file_type().is_dir() {
					fs::remove_dir_all(&full)?;
				} else {
					fs::remove_file(&full)?;
				}
			}
			Err(e) if is_absent(&e) => {}
			Err(e) => return Err(e.into()),
		}
		prune_empty_parents(&full, workdir);
	}

	if !to_restore.is_empty() {
		if let Some(tree) = &head_tree {
			let mut checkout = CheckoutBuilder::new();
			checkout
				.force()
				.update_index(false)
				.disable_pathspec_match(true);
			for path in &to_restore {
				checkout.path(path.as_slice());
			}
			repo.checkout_tree(
				tree.as_object(),
				Some(&mut checkout),
			)?;
		}
	}

	// 2. Index next.
	if let Some(tree) = &head_tree {
		index.read_tree(tree)?;
	} else {
		index.clear()?;
	}
	index.write()?;

	// 3. Repo state last.
	repo.cleanup_state()?;

	Ok(())
}

///
pub fn merge_branch(
	repo_path: &RepoPath,
	branch: &str,
	branch_type: BranchType,
) -> Result<()> {
	scope_time!("merge_branch");

	let repo = repo(repo_path)?;

	merge_branch_repo(&repo, branch, branch_type)?;

	Ok(())
}

///
pub fn rebase_progress(
	repo_path: &RepoPath,
) -> Result<RebaseProgress> {
	scope_time!("rebase_progress");

	let repo = repo(repo_path)?;

	get_rebase_progress(&repo)
}

///
pub fn continue_pending_rebase(
	repo_path: &RepoPath,
) -> Result<RebaseState> {
	scope_time!("continue_pending_rebase");

	let repo = repo(repo_path)?;

	continue_rebase(&repo)
}

///
pub fn abort_pending_rebase(repo_path: &RepoPath) -> Result<()> {
	scope_time!("abort_pending_rebase");

	let repo = repo(repo_path)?;

	abort_rebase(&repo)
}

///
pub fn merge_branch_repo(
	repo: &Repository,
	branch: &str,
	branch_type: BranchType,
) -> Result<()> {
	let branch = repo.find_branch(branch, branch_type)?;

	let annotated =
		repo.reference_to_annotated_commit(&branch.into_reference())?;

	let (analysis, _) = repo.merge_analysis(&[&annotated])?;

	//TODO: support merge on unborn
	if analysis.is_unborn() {
		return Err(Error::Generic("head is unborn".into()));
	}

	let mut opt = MergeOptions::default();

	repo.merge(&[&annotated], Some(&mut opt), None)?;

	Ok(())
}

///
pub fn merge_msg(repo_path: &RepoPath) -> Result<String> {
	scope_time!("merge_msg");

	let repo = repo(repo_path)?;
	let content = repo.message()?;

	Ok(content)
}

///
pub fn merge_commit(
	repo_path: &RepoPath,
	msg: &str,
	ids: &[CommitId],
) -> Result<CommitId> {
	scope_time!("merge_commit");

	let repo = repo(repo_path)?;

	let mut commits: Vec<Commit> = Vec::new();

	for id in ids {
		commits.push(repo.find_commit((*id).into())?);
	}

	let id = commit_merge_with_head(&repo, &commits, msg)?;

	Ok(id)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::sync::{
		checkout_branch, commit, create_branch, repo_state,
		revert_commit, stage_add_file, stage_addremoved,
		tests::{repo_init, write_commit_file},
		utils::repo_write_file,
		RepoPath, RepoState,
	};
	use pretty_assertions::assert_eq;
	use std::{fs, path::Path};

	fn make_conflicting_merge(
		repo: &Repository,
		repo_path: &RepoPath,
	) {
		write_commit_file(repo, "test.txt", "base", "base");
		create_branch(repo_path, "feature").unwrap();
		write_commit_file(repo, "test.txt", "feature", "feature");
		checkout_branch(repo_path, "master").unwrap();
		write_commit_file(repo, "test.txt", "master", "master");
		merge_branch(repo_path, "feature", BranchType::Local)
			.unwrap();
		assert_eq!(repo_state(repo_path).unwrap(), RepoState::Merge);
	}

	#[test]
	fn test_smoke() {
		let (_td, repo) = repo_init().unwrap();
		let root = repo.path().parent().unwrap();
		let repo_path: &RepoPath =
			&root.as_os_str().to_str().unwrap().into();

		let c1 =
			write_commit_file(&repo, "test.txt", "test", "commit1");

		create_branch(repo_path, "foo").unwrap();

		write_commit_file(&repo, "test.txt", "test2", "commit2");

		merge_branch(repo_path, "master", BranchType::Local).unwrap();

		let msg = merge_msg(repo_path).unwrap();

		assert_eq!(&msg[0..12], "Merge branch");

		let mergeheads = mergehead_ids(repo_path).unwrap();

		assert_eq!(mergeheads[0], c1);
	}

	#[test]
	fn t9a_untracked_survives() {
		let (_td, repo) = repo_init().unwrap();
		let root = repo.path().parent().unwrap();
		let repo_path: &RepoPath =
			&root.as_os_str().to_str().unwrap().into();

		make_conflicting_merge(&repo, repo_path);

		let notes = root.join("notes.txt");
		fs::write(&notes, "keep me").unwrap();

		abort_pending_state(repo_path).unwrap();

		assert_eq!(fs::read_to_string(&notes).unwrap(), "keep me");
		assert_eq!(repo_state(repo_path).unwrap(), RepoState::Clean);
		assert!(!repo.path().join("MERGE_HEAD").exists());
	}

	#[test]
	fn t9a_unrelated_edit_survives() {
		let (_td, repo) = repo_init().unwrap();
		let root = repo.path().parent().unwrap();
		let repo_path: &RepoPath =
			&root.as_os_str().to_str().unwrap().into();

		write_commit_file(&repo, "other.txt", "orig", "add other");
		make_conflicting_merge(&repo, repo_path);

		let other = root.join("other.txt");
		fs::write(&other, "local edit").unwrap();

		abort_pending_state(repo_path).unwrap();

		assert_eq!(fs::read_to_string(&other).unwrap(), "local edit");
		assert_eq!(repo_state(repo_path).unwrap(), RepoState::Clean);
	}

	#[test]
	fn t9a_conflict_restored() {
		let (_td, repo) = repo_init().unwrap();
		let root = repo.path().parent().unwrap();
		let repo_path: &RepoPath =
			&root.as_os_str().to_str().unwrap().into();

		write_commit_file(&repo, "test.txt", "base", "base");
		write_commit_file(&repo, "del_mod.txt", "base", "add dm");
		create_branch(repo_path, "feature").unwrap();
		write_commit_file(&repo, "test.txt", "feature", "feature");
		write_commit_file(
			&repo,
			"del_mod.txt",
			"feature_dm",
			"mod dm",
		);
		checkout_branch(repo_path, "master").unwrap();
		write_commit_file(&repo, "test.txt", "master", "master");
		fs::remove_file(root.join("del_mod.txt")).unwrap();
		stage_addremoved(repo_path, Path::new("del_mod.txt"))
			.unwrap();
		commit(repo_path, "del dm").unwrap();

		merge_branch(repo_path, "feature", BranchType::Local)
			.unwrap();

		let fresh = crate::sync::repository::repo(repo_path).unwrap();
		assert!(fresh.index().unwrap().has_conflicts());

		abort_pending_state(repo_path).unwrap();

		assert_eq!(
			fs::read_to_string(root.join("test.txt")).unwrap(),
			"master"
		);
		assert!(!root.join("del_mod.txt").exists());
		let fresh = crate::sync::repository::repo(repo_path).unwrap();
		let index = fresh.index().unwrap();
		assert!(!index.has_conflicts());
		let head_tree = fresh.head().unwrap().peel_to_tree().unwrap();
		let diff = fresh
			.diff_tree_to_index(Some(&head_tree), Some(&index), None)
			.unwrap();
		assert_eq!(diff.deltas().len(), 0);
		assert_eq!(repo_state(repo_path).unwrap(), RepoState::Clean);
	}

	#[test]
	fn t9a_merge_changes_undone() {
		let (_td, repo) = repo_init().unwrap();
		let root = repo.path().parent().unwrap();
		let repo_path: &RepoPath =
			&root.as_os_str().to_str().unwrap().into();

		write_commit_file(&repo, "test.txt", "base", "base");
		write_commit_file(&repo, "b.txt", "b_head", "add b");
		write_commit_file(&repo, "c.txt", "c_head", "add c");

		create_branch(repo_path, "feature").unwrap();
		write_commit_file(&repo, "test.txt", "feature", "f test");
		write_commit_file(&repo, "new.txt", "added", "add new");
		write_commit_file(&repo, "b.txt", "b_feature", "mod b");
		fs::remove_file(root.join("c.txt")).unwrap();
		stage_addremoved(repo_path, Path::new("c.txt")).unwrap();
		commit(repo_path, "del c").unwrap();

		checkout_branch(repo_path, "master").unwrap();
		write_commit_file(&repo, "test.txt", "master", "m test");

		merge_branch(repo_path, "feature", BranchType::Local)
			.unwrap();

		assert!(root.join("new.txt").exists());
		assert_eq!(
			fs::read_to_string(root.join("b.txt")).unwrap(),
			"b_feature"
		);
		assert!(!root.join("c.txt").exists());

		abort_pending_state(repo_path).unwrap();

		assert!(!root.join("new.txt").exists());
		assert_eq!(
			fs::read_to_string(root.join("b.txt")).unwrap(),
			"b_head"
		);
		assert_eq!(
			fs::read_to_string(root.join("c.txt")).unwrap(),
			"c_head"
		);
		assert_eq!(repo_state(repo_path).unwrap(), RepoState::Clean);
	}

	#[test]
	fn t9a_edited_merge_file_refuses() {
		let (_td, repo) = repo_init().unwrap();
		let root = repo.path().parent().unwrap();
		let repo_path: &RepoPath =
			&root.as_os_str().to_str().unwrap().into();

		write_commit_file(&repo, "test.txt", "base", "base");
		write_commit_file(&repo, "b.txt", "b_head", "add b");

		create_branch(repo_path, "feature").unwrap();
		write_commit_file(&repo, "test.txt", "feature", "f test");
		write_commit_file(&repo, "b.txt", "b_feat", "mod b");

		checkout_branch(repo_path, "master").unwrap();
		write_commit_file(&repo, "test.txt", "master", "m test");

		merge_branch(repo_path, "feature", BranchType::Local)
			.unwrap();

		// Edit cleanly merged b.txt (same length as "b_feat" to test racy stat too)
		fs::write(root.join("b.txt"), "b_edit").unwrap();

		let err = abort_pending_state(repo_path).unwrap_err();
		let msg = err.to_string();
		assert!(
			msg.contains(
				"cannot abort: local changes to b.txt would be lost"
			),
			"unexpected error: {msg}"
		);

		// Nothing changed: worktree, index, MERGE_HEAD all intact
		assert_eq!(
			fs::read_to_string(root.join("b.txt")).unwrap(),
			"b_edit"
		);
		assert_eq!(repo_state(repo_path).unwrap(), RepoState::Merge);
		assert!(repo.path().join("MERGE_HEAD").exists());
		let fresh = crate::sync::repository::repo(repo_path).unwrap();
		assert!(fresh.index().unwrap().has_conflicts());
	}

	#[test]
	fn t9a_file_became_dir() {
		let (_td, repo) = repo_init().unwrap();
		let root = repo.path().parent().unwrap();
		let repo_path: &RepoPath =
			&root.as_os_str().to_str().unwrap().into();

		write_commit_file(&repo, "test.txt", "base", "base");
		write_commit_file(&repo, "a", "a_head", "add file a");

		create_branch(repo_path, "feature").unwrap();
		write_commit_file(&repo, "test.txt", "feature", "f test");
		fs::remove_file(root.join("a")).unwrap();
		stage_addremoved(repo_path, Path::new("a")).unwrap();
		fs::create_dir(root.join("a")).unwrap();
		repo_write_file(&repo, "a/x.txt", "inside dir").unwrap();
		stage_add_file(repo_path, Path::new("a/x.txt")).unwrap();
		commit(repo_path, "replace file a with dir a/x.txt").unwrap();

		checkout_branch(repo_path, "master").unwrap();
		write_commit_file(&repo, "test.txt", "master", "m test");

		merge_branch(repo_path, "feature", BranchType::Local)
			.unwrap();
		assert!(root.join("a").is_dir());
		assert!(root.join("a/x.txt").is_file());

		abort_pending_state(repo_path).unwrap();

		assert!(root.join("a").is_file());
		assert_eq!(
			fs::read_to_string(root.join("a")).unwrap(),
			"a_head"
		);
		assert_eq!(repo_state(repo_path).unwrap(), RepoState::Clean);
	}

	#[test]
	fn t9a_dir_became_file() {
		let (_td, repo) = repo_init().unwrap();
		let root = repo.path().parent().unwrap();
		let repo_path: &RepoPath =
			&root.as_os_str().to_str().unwrap().into();

		write_commit_file(&repo, "test.txt", "base", "base");
		fs::create_dir(root.join("d")).unwrap();
		write_commit_file(&repo, "d/x.txt", "d_head", "add d/x.txt");

		create_branch(repo_path, "feature").unwrap();
		write_commit_file(&repo, "test.txt", "feature", "f test");
		fs::remove_file(root.join("d/x.txt")).unwrap();
		fs::remove_dir(root.join("d")).unwrap();
		stage_addremoved(repo_path, Path::new("d/x.txt")).unwrap();
		repo_write_file(&repo, "d", "file d").unwrap();
		stage_add_file(repo_path, Path::new("d")).unwrap();
		commit(repo_path, "replace dir d with file d").unwrap();

		checkout_branch(repo_path, "master").unwrap();
		write_commit_file(&repo, "test.txt", "master", "m test");

		merge_branch(repo_path, "feature", BranchType::Local)
			.unwrap();
		assert!(root.join("d").is_file());

		abort_pending_state(repo_path).unwrap();

		assert!(root.join("d").is_dir());
		assert_eq!(
			fs::read_to_string(root.join("d/x.txt")).unwrap(),
			"d_head"
		);
		assert_eq!(repo_state(repo_path).unwrap(), RepoState::Clean);
	}

	#[test]
	fn t9a_revert_abort_keeps_untracked() {
		let (_td, repo) = repo_init().unwrap();
		let root = repo.path().parent().unwrap();
		let repo_path: &RepoPath =
			&root.as_os_str().to_str().unwrap().into();

		write_commit_file(&repo, "test.txt", "line1\n", "c1");
		let c2 =
			write_commit_file(&repo, "test.txt", "line2\n", "c2");
		write_commit_file(&repo, "test.txt", "line3\n", "c3");

		let notes = root.join("notes.txt");
		fs::write(&notes, "keep me").unwrap();

		revert_commit(repo_path, c2).unwrap();
		assert_eq!(repo_state(repo_path).unwrap(), RepoState::Revert);

		abort_pending_state(repo_path).unwrap();

		assert_eq!(fs::read_to_string(&notes).unwrap(), "keep me");
		assert_eq!(
			fs::read_to_string(root.join("test.txt")).unwrap(),
			"line3\n"
		);
		assert_eq!(repo_state(repo_path).unwrap(), RepoState::Clean);
		assert!(!repo.path().join("REVERT_HEAD").exists());
	}
}
