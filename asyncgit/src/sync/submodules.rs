use std::path::{Path, PathBuf};

use git2::{
	Repository, RepositoryOpenFlags, Submodule,
	SubmoduleUpdateOptions,
};
use scopetime::scope_time;

use super::{repo, CommitId, RepoPath};
use crate::{error::Result, sync::utils::work_dir, Error};

pub use git2::SubmoduleStatus;

///
#[derive(Debug)]
pub struct SubmoduleInfo {
	///
	pub name: String,
	///
	pub path: PathBuf,
	///
	pub url: Option<String>,
	///
	pub id: Option<CommitId>,
	///
	pub head_id: Option<CommitId>,
	///
	pub status: SubmoduleStatus,
}

///
#[derive(Debug)]
pub struct SubmoduleParentInfo {
	/// where to find parent repo
	pub parent_gitpath: PathBuf,
	/// where to find submodule git path
	pub submodule_gitpath: PathBuf,
	/// `submodule_info` from perspective of parent repo
	pub submodule_info: SubmoduleInfo,
}

/// Snapshot of a submodule's recorded and checked-out commits
/// for stale-confirmation checks.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct SubmoduleSnapshot {
	/// Commit recorded for the submodule in the parent repo's index
	pub index_id: Option<CommitId>,
	/// Commit recorded for the submodule in the parent repo's HEAD
	pub head_id: Option<CommitId>,
	/// Commit currently checked out in the submodule workdir
	pub workdir_id: Option<CommitId>,
	/// Whether the submodule is currently uninitialized
	pub uninitialized: bool,
}

impl SubmoduleSnapshot {
	/// Commit `update_submodule` will check out.
	pub fn target_commit(&self) -> Option<CommitId> {
		self.index_id.or(self.head_id)
	}
}

/// Capture a submodule's recorded and checked-out commits for
/// stale-confirmation checks. Returns `Ok(None)` if the submodule
/// no longer exists.
pub fn submodule_snapshot(
	repo_path: &RepoPath,
	name: &str,
) -> Result<Option<SubmoduleSnapshot>> {
	scope_time!("submodule_snapshot");

	let repo = repo(repo_path)?;
	let submodule = match repo.find_submodule(name) {
		Ok(s) => s,
		Err(e) if e.code() == git2::ErrorCode::NotFound => {
			return Ok(None);
		}
		Err(e) => return Err(e.into()),
	};
	let status = repo
		.submodule_status(name, git2::SubmoduleIgnore::None)
		.unwrap_or(SubmoduleStatus::empty());

	Ok(Some(SubmoduleSnapshot {
		index_id: submodule.index_id().map(CommitId::from),
		head_id: submodule.head_id().map(CommitId::from),
		workdir_id: submodule.workdir_id().map(CommitId::from),
		uninitialized: status.is_wd_uninitialized()
			|| submodule.workdir_id().is_none(),
	}))
}

impl SubmoduleInfo {
	///
	pub fn get_repo_path(
		&self,
		repo_path: &RepoPath,
	) -> Result<RepoPath> {
		let repo = repo(repo_path)?;
		let wd = repo.workdir().ok_or(Error::NoWorkDir)?;

		Ok(RepoPath::Path(wd.join(self.path.clone())))
	}
}

fn submodule_to_info(s: &Submodule, r: &Repository) -> SubmoduleInfo {
	let status = r
		.submodule_status(
			s.name().unwrap_or_default(),
			git2::SubmoduleIgnore::None,
		)
		.unwrap_or(SubmoduleStatus::empty());

	SubmoduleInfo {
		name: s.name().unwrap_or_default().into(),
		path: s.path().to_path_buf(),
		id: s.workdir_id().map(CommitId::from),
		head_id: s.head_id().map(CommitId::from),
		url: s.url().ok().flatten().map(String::from),
		status,
	}
}

///
pub fn get_submodules(
	repo_path: &RepoPath,
) -> Result<Vec<SubmoduleInfo>> {
	scope_time!("get_submodules");

	let (r, repo2) = (repo(repo_path)?, repo(repo_path)?);

	let res = r
		.submodules()?
		.iter()
		.map(|s| submodule_to_info(s, &repo2))
		.collect();

	Ok(res)
}

///
pub fn update_submodule(
	repo_path: &RepoPath,
	name: &str,
) -> Result<()> {
	scope_time!("update_submodule");

	let repo = repo(repo_path)?;

	let mut submodule = repo.find_submodule(name)?;

	let mut options = SubmoduleUpdateOptions::new();
	options.allow_fetch(true);

	submodule.update(true, Some(&mut options))?;

	Ok(())
}

/// query whether `repo_path` points to a repo that is part of a parent git which contains it as a submodule
pub fn submodule_parent_info(
	repo_path: &RepoPath,
) -> Result<Option<SubmoduleParentInfo>> {
	scope_time!("submodule_parent_info");

	let repo = repo(repo_path)?;
	let repo_wd = work_dir(&repo)?.to_path_buf();

	log::trace!("[sub] repo_wd: {repo_wd:?}");
	log::trace!("[sub] repo_path: {:?}", repo.path());

	if let Some(parent_path) = repo_wd.parent() {
		log::trace!("[sub] parent_path: {parent_path:?}");

		if let Ok(parent) = Repository::open_ext(
			parent_path,
			RepositoryOpenFlags::FROM_ENV,
			Vec::<&Path>::new(),
		) {
			let parent_wd = work_dir(&parent)?.to_path_buf();
			log::trace!("[sub] parent_wd: {parent_wd:?}");

			let submodule_name = repo_wd
				.strip_prefix(parent_wd)?
				.to_string_lossy()
				.to_string();

			log::trace!("[sub] submodule_name: {submodule_name:?}");

			if let Ok(submodule) =
				parent.find_submodule(&submodule_name)
			{
				return Ok(Some(SubmoduleParentInfo {
					parent_gitpath: parent.path().to_path_buf(),
					submodule_gitpath: repo.path().to_path_buf(),
					submodule_info: submodule_to_info(
						&submodule, &parent,
					),
				}));
			}
		}
	}

	Ok(None)
}

#[cfg(test)]
mod tests {
	use super::get_submodules;
	use crate::sync::{
		submodules::submodule_parent_info, tests::repo_init, RepoPath,
	};
	use git2::Repository;
	use pretty_assertions::assert_eq;
	use std::path::Path;

	#[test]
	fn test_smoke() {
		let (dir, _r) = repo_init().unwrap();

		{
			let r = Repository::open(dir.path()).unwrap();
			let mut s = r
				.submodule(
					//TODO: use local git
					"https://github.com/extrawurst/brewdump.git",
					Path::new("foo/bar"),
					false,
				)
				.unwrap();

			let _sub_r = s.clone(None).unwrap();
			s.add_finalize().unwrap();
		}

		let repo_p = RepoPath::Path(dir.keep());
		let subs = get_submodules(&repo_p).unwrap();

		assert_eq!(subs.len(), 1);
		assert_eq!(&subs[0].name, "foo/bar");

		let info = submodule_parent_info(
			&subs[0].get_repo_path(&repo_p).unwrap(),
		)
		.unwrap()
		.unwrap();

		dbg!(&info);

		assert_eq!(&info.submodule_info.name, "foo/bar");
	}
}
