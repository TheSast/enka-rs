use std::process::Command;
fn main() {
    let git_hash = String::from_utf8(
        Command::new("git")
            .args(["rev-parse", "HEAD"])
            .output()
            .expect("Failed to execute `git rev-parse HEAD`")
            .stdout,
    )
    .expect("Git output is not valid UTF-8");
    let is_dirty = !{
        Command::new("git")
            .args(["status", "--porcelain"])
            .output()
            .expect("Failed to execute `git status --porcelain`")
            .stdout
            .is_empty()
    };
    println!(
        "cargo:rustc-env=GIT_HASH={}{}",
        git_hash.trim(),
        if is_dirty { "-drity" } else { "" }
    );
    println!("cargo:rustc-rerun-if-changed=.git/HEAD");
    println!("cargo:rustc-rerun-if-changed=.git/index");
    println!("cargo:rustc-rerun-if-changed=.git/refs/heads/");
}
