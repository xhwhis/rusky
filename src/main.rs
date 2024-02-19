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

fn help() {
    println!(
        r#"Usage: rusky <command> [args]

    Commands:
        install [dir] Install rusky to git hooks directory (default: .rusky)
        uninstall     Uninstall rusky from git hooks directory

    Environment variables:
        RUSKY=0       Skip install

    Examples:
        rusky install
        rusky install .rusky
        rusky uninstall
        echo "cargo fmt" > .rusky/commit-msg
        echo "cargo clippy" >> .rusky/pre-commit"#
    );
}

fn install(dir: &str) {
    let output = Command::new("git")
        .arg("--version")
        .output()
        .expect("failed to execute process");
    if !output.status.success() {
        eprintln!("git command not found");
        exit(1)
    }

    let hooks_dir = Path::new(dir).join("_");
    if !hooks_dir.exists() {
        std::fs::create_dir_all(&hooks_dir).expect("failed to create git hooks directory");
    }
    std::fs::write(hooks_dir.join(".gitignore"), "*").expect("failed to write .gitignore");
    std::fs::write(hooks_dir.join("rusky"), include_str!("../rusky"))
        .expect("failed to write rusky script");

    for hook in HOOKS {
        let hook_file = hooks_dir.join(hook);
        std::fs::write(&hook_file, "#!/usr/bin/env sh\n. \"${0%/*}/rusky\"")
            .expect("failed to write hook file");
        std::fs::set_permissions(&hook_file, Permissions::from_mode(0o755))
            .expect("failed to set hook file permissions");
    }

    let hooks_path = format!("{dir}/_");
    let output = Command::new("git")
        .args(["config", "core.hooksPath", &hooks_path])
        .output()
        .expect("failed to execute process");
    if !output.status.success() {
        eprintln!("failed to set hooks path");
        exit(1)
    }
}

fn uninstall() {
    let output = Command::new("git")
        .arg("--version")
        .output()
        .expect("failed to execute process");
    if !output.status.success() {
        eprintln!("git command not found");
        exit(1)
    }

    let output = Command::new("git")
        .args(["config", "--unset", "core.hooksPath"])
        .output()
        .expect("failed to execute process");
    if !output.status.success() {
        eprintln!("failed to unset hooks path");
        exit(1)
    }
}

fn main() {
    let mut args = std::env::args().skip(1);

    if let Some(cmd) = args.next() {
        match cmd.as_str() {
            "install" => {
                if let Some("0") = option_env!("RUSKY") {
                    println!("RUSKY=0 skip install");
                    return;
                }

                if let Some(dir) = args.next() {
                    install(&dir);
                } else {
                    install(".rusky");
                }
            }
            "uninstall" => {
                uninstall();
            }
            _ => {
                help();
            }
        }
    } else {
        help();
    }
}
