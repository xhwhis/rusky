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
        install [dir]    Install rusky to git hooks directory, default to .rusky
        uninstall        Uninstall rusky from git hooks directory
        set <file> <cmd> Set command to hook file
        add <file> <cmd> Add command to hook file

    Environment variables:
        RUSKY=0          Skip install

    Examples:
        rusky install
        rusky install .rusky
        rusky uninstall
        rusky set .rusky/commit-msg "cargo fmt"
        rusky add .rusky/pre-commit "cargo clippy -- -D warning""#
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

fn set(file: &str, cmd: &str) {
    std::fs::write(file, cmd).expect("failed to execute process");
}

fn add(file: &str, cmd: &str) {
    let mut content = std::fs::read_to_string(file).expect("failed to execute process");
    content.push_str(cmd);
    std::fs::write(file, content).expect("failed to execute process");
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
            "set" => {
                if let Some(arg1) = args.next() {
                    if let Some(arg2) = args.next() {
                        set(&arg1, &arg2);
                    } else {
                        help();
                    }
                } else {
                    help();
                }
            }
            "add" => {
                if let Some(arg1) = args.next() {
                    if let Some(arg2) = args.next() {
                        add(&arg1, &arg2);
                    } else {
                        help();
                    }
                } else {
                    help();
                }
            }
            _ => {
                help();
            }
        }
    } else {
        help();
    }
}
