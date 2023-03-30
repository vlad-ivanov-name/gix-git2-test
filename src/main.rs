use clap::Parser;
use gix::bstr::BString;
use gix::ObjectId;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Parser, Debug)]
#[command()]
struct Args {
    #[arg(long)]
    use_gix: bool,

    #[arg(index = 1)]
    path: PathBuf,
}

static COMMIT_REF: &str = "8d83139ab7fd7bdb569417dc17494f9730b1b6ec";
static COMMIT_COUNT: usize = 10000;

fn make_message(i: usize) -> anyhow::Result<String> {
    let time = SystemTime::now();
    let millis = time.duration_since(UNIX_EPOCH)?.as_millis();
    let message = format!("gix-commit-test-{}-{}", i, millis);
    Ok(message)
}

fn create_commits_gix(path: &Path) -> anyhow::Result<()> {
    let repo = gix::open(path)?;
    let id = gix::ObjectId::from_str(COMMIT_REF)?;
    let commit = repo.find_object(id)?.try_into_commit()?;

    for i in 0..COMMIT_COUNT {
        let message = make_message(i)?;

        let new_commit = gix_object::Commit {
            tree: commit.id,
            parents: commit.parent_ids().map(|id| ObjectId::from(id)).collect(),
            author: commit.author()?.to_owned(),
            committer: commit.committer()?.to_owned(),
            encoding: None,
            message: BString::from(message),
            extra_headers: vec![],
        };

        repo.write_object(new_commit)?;
    }

    Ok(())
}

fn create_commits_git2(path: &Path) -> anyhow::Result<()> {
    let repo = git2::Repository::open(path)?;
    let id = git2::Oid::from_str(COMMIT_REF)?;
    let commit = repo.find_commit(id)?;

    for i in 0..COMMIT_COUNT {
        let message = make_message(i)?;
        let parents = commit
            .parents()
            .collect::<Vec<_>>();

        let parents = parents
            .iter()
            .map(|commit| commit)
            .collect::<Vec<_>>();

        let new_commit = repo.commit_create_buffer(
            &commit.author(),
            &commit.committer(),
            &message,
            &commit.tree()?,
            &parents,
        )?;

        repo.odb()?.write(git2::ObjectType::Commit, &new_commit)?;
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if args.use_gix {
        create_commits_gix(&args.path)
    } else {
        create_commits_git2(&args.path)
    }
}
