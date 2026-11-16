// build.rs

use std::process::Command;

fn main() {
    // commit for frame stats recordings, "-dirty" with uncommitted changes
    // no rerun-if-changed, so cargo reruns this when any package file changes
    let git = |args: &[&str]| {
        Command::new("git")
            .args(args)
            .output()
            .ok()
            .filter(|out| out.status.success())
            .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
    };
    let commit = git(&["rev-parse", "--short", "HEAD"]).unwrap_or_else(|| "unknown".into());
    let dirty = git(&["status", "--porcelain", "--untracked-files=no"])
        .is_some_and(|status| !status.is_empty());
    let suffix = if dirty { "-dirty" } else { "" };
    println!("cargo:rustc-env=BLOCKS_COMMIT={commit}{suffix}");

    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("resources/blocks256x256.ico");
        res.compile().unwrap();
    }
}
