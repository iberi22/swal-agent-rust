use std::process::Command;

#[test]
fn test_run_e2e() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("-p")
        .arg("swal-agent")
        .arg("--")
        .arg("run")
        .arg("hello")
        .output()
        .expect("failed to execute cargo run");

    assert!(output.status.success(), "Command did not exit successfully");

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 stdout");
    let stderr = String::from_utf8(output.stderr).expect("Invalid UTF-8 stderr");

    println!("STDOUT: {}", stdout);
    println!("STDERR: {}", stderr);

    assert!(stdout.contains("done"), "Output does not contain 'done'");
}
