use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

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
        println!("cargo:warning=RUSKY=0 skip install");
        return;
    }

    let output = Command::new("git")
        .arg("--version")
        .output()
        .expect("failed to execute process");
    if !output.status.success() {
        panic!("git command not found");
    }
    let target_dir = std::env::var("OUT_DIR").expect("failed to get target directory");
    let output = Command::new("git")
        .current_dir(&target_dir)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .expect("failed to execute process");
    if !output.status.success() {
        panic!("failed to get git root directory");
    }
    let project_dir = String::from_utf8_lossy(&output.stdout);
    let project_dir = project_dir.trim();

    let hooks_dir = Path::new(project_dir).join(HOOKS_PATH);
    if !hooks_dir.exists() {
        std::fs::create_dir_all(&hooks_dir).expect("failed to create hooks directory");
    }
    std::fs::write(hooks_dir.join(".gitignore"), "*").expect("failed to write .gitignore");
    std::fs::write(hooks_dir.join("rusky"), include_str!("rusky"))
        .expect("failed to write rusky script");

    for hook in HOOKS {
        let hook_file = hooks_dir.join(hook);
        std::fs::write(&hook_file, "#!/usr/bin/env sh\n. \"${0%/*}/rusky\"")
            .expect("failed to write hook script");
        std::fs::set_permissions(&hook_file, Permissions::from_mode(0o755))
            .expect("failed to set hook script permissions");
    }

    let output = Command::new("git")
        .current_dir(project_dir)
        .args(["config", "core.hooksPath", HOOKS_PATH])
        .output()
        .expect("failed to execute process");
    if !output.status.success() {
        panic!("failed to set hooks path");
    }
}
