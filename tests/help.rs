use assert_cmd::prelude::*; // Add methods on commands
// use predicates::prelude::*; // Used for writing assertions
use std::process::Command; // Run programs

#[test]
fn backup_requires_args() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("backrest")?;
    cmd.current_dir("backrest");

    cmd.arg("backup");
    cmd.assert().failure();

    Ok(())
}

#[test]
fn backup_does_not_create_empty_backups_on_nonexistent_dirs() -> Result<(), Box<dyn std::error::Error>> {
    unimplemented!()
    // let mut cmd = Command::cargo_bin("backrest")?;
    // cmd.current_dir("backrest");

    // cmd.args(&["backup", "a/s/d/f", "--"]);
    // cmd.assert();

    // Ok(())
}
