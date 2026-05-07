use crate::settings;
use std::io::Write;
use std::process::ExitStatus;

pub fn run_cmd(commit: &str) -> ExitStatus {
    let tmp_path = settings::get_temp_commit_msq_path();
    let mut file = std::fs::File::create(&tmp_path).expect("Unable to create file");
    file.write_all(commit.as_bytes())
        .expect("Unable to write data to file");

    let status = std::process::Command::new("git")
        .arg("commit")
        .arg("-F")
        .arg(&tmp_path)
        .status()
        .expect("Failed to execute git commit");

    // Clean up the temporary file
    std::fs::remove_file(tmp_path).expect("Failed to remove temporary file");

    status
}
