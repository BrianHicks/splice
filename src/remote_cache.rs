use eyre::{Context, Result};
use gix::remote::fetch::Shallow::DepthAtRemote;
use std::{num::NonZeroU32, path::PathBuf, sync::atomic::AtomicBool};
use tempdir::TempDir;

#[derive(Debug)]
pub struct RemoteCache {
    root: PathBuf,
}

impl RemoteCache {
    pub fn new(root: PathBuf) -> Self {
        RemoteCache { root }
    }

    fn sanitize_path(path: &str) -> String {
        path.replace(['/', '\\'], "__")
    }

    fn git_path(repo: &str, rev: &str) -> String {
        format!("git-{}-{}", Self::sanitize_path(repo), rev)
    }

    #[tracing::instrument(skip_all)]
    pub fn fetch_git(&self, repo: &str, rev: &Option<String>) -> Result<PathBuf> {
        match rev
            .as_ref()
            .map(|r| self.root.join(&Self::git_path(repo, r)))
        {
            Some(path) if path.exists() => Ok(path),
            _ => {
                let temp = TempDir::new("splice-fetch")
                    .wrap_err("could not create temporary directory for clone")?;

                let mut clone = gix::prepare_clone(repo, &temp)?
                    .with_ref_name(rev.as_deref())?
                    .with_shallow(DepthAtRemote(NonZeroU32::MIN));

                tracing::info!(?repo, ?rev, "cloning");
                let mut prepare_checkout = clone
                    .fetch_then_checkout(gix::progress::Discard, &AtomicBool::new(false))
                    .wrap_err("failed to fetch remote")?
                    .0;

                let git_repo = prepare_checkout
                    .main_worktree(gix::progress::Discard, &AtomicBool::new(false))
                    .wrap_err("failed to checkout worktree")?
                    .0;

                let sha = git_repo
                    .head_id()
                    .wrap_err("could not get HEAD SHA")?
                    .to_string();

                let dest = self.root.join(&Self::git_path(repo, &sha));

                if !dest.exists() {
                    std::fs::create_dir_all(&self.root)
                        .wrap_err("could not create cache root directory")?;
                    std::fs::rename(temp.into_path(), &dest)
                        .wrap_err("could not move clone to cache directory")?;
                }

                Ok(dest)
            }
        }
    }
}
