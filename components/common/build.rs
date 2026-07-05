use std::process::Command;

// Version resolution order:
// 1. BOTLOADER_VERSION env var, set by CI from the release git tag (e.g. "2026.5.1")
// 2. `git describe` for local/untagged builds (e.g. "v2026.5.1-3-gabc1234" or a bare sha)
// 3. "dev" when neither is available
fn main() {
    println!("cargo:rerun-if-env-changed=BOTLOADER_VERSION");

    let version = std::env::var("BOTLOADER_VERSION")
        .ok()
        .filter(|v| !v.is_empty())
        .or_else(git_describe)
        .unwrap_or_else(|| "dev".to_string());

    println!("cargo:rustc-env=BOTLOADER_VERSION={version}");
}

fn git_describe() -> Option<String> {
    let output = Command::new("git")
        .args(["describe", "--tags", "--always", "--dirty"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let described = String::from_utf8(output.stdout).ok()?;
    let trimmed = described.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.trim_start_matches('v').to_string())
    }
}
