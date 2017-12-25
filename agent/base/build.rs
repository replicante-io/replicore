extern crate git2;

use git2::STATUS_INDEX_DELETED;
use git2::STATUS_INDEX_MODIFIED;
use git2::STATUS_INDEX_NEW;
use git2::STATUS_INDEX_RENAMED;
use git2::STATUS_INDEX_TYPECHANGE;

use git2::STATUS_WT_DELETED;
use git2::STATUS_WT_MODIFIED;
use git2::STATUS_WT_NEW;
use git2::STATUS_WT_RENAMED;
use git2::STATUS_WT_TYPECHANGE;

use git2::Repository;
use git2::Status;


fn is_index_status(status: &Status) -> bool {
    status.intersects(
        STATUS_INDEX_DELETED |
        STATUS_INDEX_MODIFIED |
        STATUS_INDEX_NEW |
        STATUS_INDEX_RENAMED |
        STATUS_INDEX_TYPECHANGE
    )
}

fn is_workdir_status(status: &Status) -> bool {
    status.intersects(
        STATUS_WT_DELETED |
        STATUS_WT_MODIFIED |
        STATUS_WT_NEW |
        STATUS_WT_RENAMED |
        STATUS_WT_TYPECHANGE
    )
}


fn main() {
    println!("cargo:rustc-env=GIT_BUILD_HASH={}", git_hash());
    println!("cargo:rustc-env=GIT_BUILD_TAINT={}", git_taint());
}


fn git_hash() -> String {
    let repo = Repository::discover(".").unwrap();
    let checkout = repo.revparse_single("HEAD").unwrap();
    format!("{}", checkout.id())
}

fn git_taint() -> String {
    let repo = Repository::discover(".").unwrap();
    let mut index_changed = false;
    let mut workdir_changed = false;

    for entry in repo.statuses(None).unwrap().iter() {
        let status = entry.status();
        if is_index_status(&status) {
            index_changed = true;
        }
        if is_workdir_status(&status) {
            workdir_changed = true;
        }
    }

    match (index_changed, workdir_changed) {
        (true, true) => String::from("index and working directory tainted"),
        (true, false) => String::from("index tainted"),
        (false, true) => String::from("working directory tainted"),
        (false, false) => String::from("not tainted")
    }
}
