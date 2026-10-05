use pybox::sandbox::PySandbox;

#[test]
fn test_exec_simple_expression() {
    let mut sandbox = PySandbox::new_for_test(None).expect("Failed to create sandbox");
    let result = sandbox.exec("1 + 1").unwrap();
    assert_eq!(result.value, "2");
}

#[test]
fn test_exec_returns_error_for_invalid_syntax() {
    let mut sandbox = PySandbox::new_for_test(None).expect("Failed to create sandbox");
    // Invalid Python syntax should return an error
    let result = sandbox.exec("this is definitely not valid python");
    assert!(result.is_err());
}

#[test]
fn test_exec_handles_empty_string() {
    let mut sandbox = PySandbox::new_for_test(None).expect("Failed to create sandbox");
    // Empty string might be valid or error depending on implementation
    let _result = sandbox.exec("");
    // Just ensure it doesn't panic - we don't assert on result
}

#[test]
fn test_timeout_actually_triggers() {
    let mut sandbox = PySandbox::new_for_test(Some(1)).expect("Failed to create sandbox");
    // Infinite loop should timeout
    let result = sandbox.exec("while True: pass");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "Execution timed out");
}

#[test]
fn test_exec_with_complex_expression() {
    let mut sandbox = PySandbox::new_for_test(None).expect("Failed to create sandbox");
    // More complex expression
    let result = sandbox.exec("(1 + 2) * (3 + 4)").unwrap();
    assert_eq!(result.value, "21");
}

#[test]
fn test_exec_handles_whitespace() {
    let mut sandbox = PySandbox::new_for_test(None).expect("Failed to create sandbox");
    // Expression with lots of whitespace
    let result = sandbox.exec("  5   +   10  ").unwrap();
    assert_eq!(result.value, "15");
}

#[test]
fn test_exec_handles_multi_line_code() {
    let mut sandbox = PySandbox::new_for_test(None).expect("Failed to create sandbox");

    let code = r#"
def fibonacci(n: int):
    """Return a list with the first n Fibonacci numbers."""
    if n <= 0:
        return []
    seq = [0, 1]
    while len(seq) < n:
        seq.append(seq[-1] + seq[-2])
    return seq[:n]

fibonacci(10)
"#;

    let result = sandbox.exec(code).unwrap();
    assert_eq!(result.value, "[0, 1, 1, 2, 3, 5, 8, 13, 21, 34]");
}

#[test]
fn test_multiple_sandbox_instances_are_isolated() {
    // Create multiple independent sandbox instances
    let mut sandbox1 = PySandbox::new_for_test(None).expect("Failed to create first sandbox");
    let mut sandbox2 = PySandbox::new_for_test(Some(10)).expect("Failed to create second sandbox");

    let code1 = r#"
a = 1
b = 2
a + b
"#;
    let code2 = "a + b";
    let result1 = sandbox1.exec(code1);
    let result2 = sandbox2.exec(code2);

    assert_eq!(result1.unwrap().value, "3");
    // Second result should fail because it references undefined vars
    assert!(result2.is_err());
}

#[test]
fn test_exec_captures_print_output() {
    let mut sandbox = PySandbox::new_for_test(None).expect("Failed to create sandbox");
    // print() goes through WASI stdout, which is captured into `stdout`
    // instead of being written to the host's stdout.
    let result = sandbox.exec("print('hello from print')\n42").unwrap();
    assert_eq!(result.value, "42");
    assert_eq!(result.stdout, "hello from print\n");
    assert_eq!(result.stderr, "");
}

#[test]
fn test_exec_captures_stderr() {
    let mut sandbox = PySandbox::new_for_test(None).expect("Failed to create sandbox");
    let result = sandbox
        .exec("import sys; sys.stderr.write('boom\\n')\n1")
        .unwrap();
    assert_eq!(result.value, "1");
    assert_eq!(result.stdout, "");
    assert_eq!(result.stderr, "boom\n");
}

#[test]
fn test_stdin_is_empty() {
    let mut sandbox = PySandbox::new_for_test(None).expect("Failed to create sandbox");
    // Guest stdin is an empty pipe, so reads return EOF immediately rather
    // than blocking on or exposing the host's stdin.
    let result = sandbox
        .exec("import sys; sys.stdin.read()")
        .unwrap();
    assert_eq!(result.value, "\"\"");
}

#[test]
fn test_sys_exit_returns_error_not_timeout() {
    let mut sandbox = PySandbox::new_for_test(Some(5)).expect("Failed to create sandbox");
    let start = std::time::Instant::now();
    let result = sandbox.exec("import sys; sys.exit(0)");
    assert!(result.is_err());
    // Must fail fast via the BaseException handler, not hang until timeout.
    assert!(
        start.elapsed() < std::time::Duration::from_secs(5),
        "sys.exit() should not consume the full timeout"
    );
    assert!(result.unwrap_err().to_string().contains("SystemExit"));
}

#[test]
fn test_memory_limit_enforced() {
    let mut sandbox = PySandbox::new_for_test(None).expect("Failed to create sandbox");
    // Allocating well beyond the configured limit must fail, not OOM the host.
    let result = sandbox.exec("bytearray(1024 * 1024 * 1024)");
    assert!(result.is_err());
}

#[test]
fn test_output_limit_enforced() {
    let mut sandbox = PySandbox::new_for_test(None).expect("Failed to create sandbox");
    // Writing more than the bounded stdout pipe capacity must fail.
    let result = sandbox.exec("print('x' * (4 * 1024 * 1024))");
    assert!(result.is_err());
}

#[test]
fn test_reused_sandbox_has_no_stale_timeout() {
    // A fast call must not leave a timer behind that kills a later call.
    let mut sandbox = PySandbox::new_for_test(Some(5)).expect("Failed to create sandbox");
    sandbox.exec("1 + 1").unwrap();
    std::thread::sleep(std::time::Duration::from_millis(4500));
    let result = sandbox
        .exec("total = 0\nfor i in range(5_000_000):\n    total += i\ntotal")
        .unwrap();
    assert_eq!(result.value, "12499997500000");
}
