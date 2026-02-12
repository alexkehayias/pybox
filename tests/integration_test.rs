use pybox::sandbox::PySandbox;
use std::path::Path;

/// Helper to check if sandbox.wasm exists
fn has_sandbox_wasm() -> bool {
    Path::new("sandbox.wasm").exists()
}

#[test]
fn test_exec_simple_expression() {
    if !has_sandbox_wasm() {
        return; // Skip if sandbox.wasm doesn't exist
    }

    let mut sandbox = PySandbox::new_for_test(None).expect("Failed to create sandbox");
    let result = sandbox.exec("1 + 1").unwrap();
    assert!(result.contains("2"));
}

#[test]
fn test_exec_returns_error_for_invalid_syntax() {
    if !has_sandbox_wasm() {
        return;
    }

    let mut sandbox = PySandbox::new_for_test(None).expect("Failed to create sandbox");
    // Invalid Python syntax should return an error
    let result = sandbox.exec("this is definitely not valid python");
    assert!(result.is_err());
}

#[test]
fn test_exec_handles_empty_string() {
     if !has_sandbox_wasm() {
        return;
    }

    let mut sandbox = PySandbox::new_for_test(None).expect("Failed to create sandbox");
    // Empty string might be valid or error depending on implementation
    let _result = sandbox.exec("");
    // Just ensure it doesn't panic - we don't assert on result
}

#[test]
fn test_timeout_actually_triggers() {
    if !has_sandbox_wasm() {
        return;
    }

    let mut sandbox = PySandbox::new_for_test(Some(1)).expect("Failed to create sandbox");
    // Infinite loop should timeout
    let result = sandbox.exec("while True: pass");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "Execution timed out");
}

#[test]
fn test_exec_with_complex_expression() {
    if !has_sandbox_wasm() {
        return;
    }

    let mut sandbox = PySandbox::new_for_test(None).expect("Failed to create sandbox");
    // More complex expression
    let result = sandbox.exec("(1 + 2) * (3 + 4)").unwrap_or_else(|_| String::new());
    assert!(result.contains("21"));
}

#[test]
fn test_exec_handles_whitespace() {
    if !has_sandbox_wasm() {
        return;
    }

    let mut sandbox = PySandbox::new_for_test(None).expect("Failed to create sandbox");
    // Expression with lots of whitespace
    let result = sandbox.exec("  5   +   10  ").unwrap_or_else(|_| String::new());
    assert!(result.contains("15"));
}

#[test]
fn test_exec_handles_multi_line_code() {
    if !has_sandbox_wasm() {
        return;
    }

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
    assert_eq!(result, "[0, 1, 1, 2, 3, 5, 8, 13, 21, 34]");
}

#[test]
fn test_multiple_sandbox_instances_are_isolated() {
    if !has_sandbox_wasm() {
        return;
    }

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

    assert_eq!(result1.unwrap(), "3");
    // Second result should fail because it references undefined vars
    assert!(result2.is_err());
}
