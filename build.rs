use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{exit, Command};

const HOOKS: &[&str] = &[
    "pre-commit",
    "prepare-commit-msg",
    "commit-msg",
    "post-commit",
    "applypatch-msg",
    "pre-applypatch",
    "post-applypatch",
    "pre-rebase",
    "post-rewrite",
    "post-checkout",
    "post-merge",
    "pre-push",
    "pre-auto-gc",
];

const HOOKS_PATH: &str = ".rusky/_";

fn main() {
    if let Some("0") = option_env!("RUSKY") {
        println!("RUSKY=0 skip install");
        return;
    }

    let output = Command::new("git")
        .arg("--version")
        .output()
        .expect("failed to execute process");
    if !output.status.success() {
        eprintln!("git command not found");
        exit(1)
    }
    let target_dir = std::env::var("OUT_DIR").expect("failed to get target directory");
    let output = Command::new("git")
        .current_dir(target_dir)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .expect("failed to execute process");
    if !output.status.success() {
        eprintln!("failed to get git root directory");
        exit(1)
    }
    let project_dir = String::from_utf8_lossy(&output.stdout);
    let project_dir = project_dir.trim();

    let output = Command::new("git")
        .args(["config", "core.hooksPath", HOOKS_PATH])
        .output()
        .expect("failed to execute process");
    if !output.status.success() {
        eprintln!("failed to set hooks path");
        exit(1)
    }

    let hooks_dir = Path::new(project_dir).join(HOOKS_PATH);
    std::fs::create_dir_all(&hooks_dir).expect("failed to execute process");
    std::fs::write(hooks_dir.join(".gitignore"), "*").expect("failed to execute process");
    std::fs::write(hooks_dir.join("rusky"), include_str!("rusky"))
        .expect("failed to execute process");

    for hook in HOOKS {
        let hook_file = hooks_dir.join(hook);
        std::fs::write(&hook_file, "#!/usr/bin/env sh\n. \"${0%/*}/rusky\"")
            .expect("failed to execute process");
        std::fs::set_permissions(&hook_file, Permissions::from_mode(0o755))
            .expect("failed to execute process");
    }
}
