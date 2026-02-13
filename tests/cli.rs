use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn help_runs() {
    let mut cmd = Command::cargo_bin("skills-cli").unwrap();

    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Universal Agent Skill Installer"));
}

#[test]
fn list_runs() {
    let mut cmd = Command::cargo_bin("skills-cli").unwrap();

    cmd.arg("list")
        .assert()
        .success();
}

#[test]
fn install_requires_repo_or_url() {
    let mut cmd = Command::cargo_bin("skills-cli").unwrap();

    cmd.args([
        "install",
        "gemini",
        "--path",
        "foo"
    ])
    .assert()
    .failure();
}
