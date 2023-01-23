use std::io::Write;
use std::process::{Command, Stdio};

fn check_step01(expected: i64, input: i64) {
    let mut command = Command::new("cargo");

    command.args([
        "run",
        "--bin",
        "step01",
        "--",
        input.to_string().as_str(),
    ]);

    let child = command.stdout(Stdio::piped()).spawn().unwrap();
    let output = child.wait_with_output().unwrap().stdout;
    let mut tmp_file = std::fs::File::create("tmp.s").unwrap();
    tmp_file.write_all(&output).unwrap();

    let _ = Command::new("cc").args([
        "-o",
        "tmp",
        "tmp.s"
    ]).output().unwrap();

    let output = Command::new("./tmp").output().unwrap().status.code().unwrap();

    assert_eq!(output, expected as i32);
}

fn main() {
    check_step01(0, 0);
    check_step01(42, 42);
}