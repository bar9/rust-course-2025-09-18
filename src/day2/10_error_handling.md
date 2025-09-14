# Chapter 10: Error Handling - Result, ?, and Custom Errors
## Robust Error Management in Rust

### Learning Objectives
By the end of this chapter, you'll be able to:
- Use Result<T, E> for recoverable error handling
- Master the ? operator for error propagation
- Create custom error types with proper error handling
- Understand when to use Result vs panic!
- Work with popular error handling crates (anyhow, thiserror)
- Implement error conversion and chaining
- Handle multiple error types gracefully

---

## Rust's Error Handling Philosophy

### Error Categories

| Type | Examples | Rust Approach |
|------|----------|---------------|
| **Recoverable** | File not found, network timeout | `Result<T, E>` |
| **Unrecoverable** | Array out of bounds, null pointer | `panic!` |

### Comparison with Other Languages

| Language | Approach | Pros | Cons |
|----------|----------|------|------|
| **C++** | Exceptions, error codes | Familiar | Runtime overhead, can be ignored |
| **C#/.NET** | Exceptions | Clean syntax | Performance cost, hidden control flow |
| **Go** | Explicit error returns | Explicit, fast | Verbose |
| **Rust** | Result<T, E> | Explicit, zero-cost | Must be handled |

---

## Result<T, E>: The Foundation

### Basic Result Usage

```rust
use std::fs::File;
use std::io::ErrorKind;

fn open_file(filename: &str) -> Result<File, std::io::Error> {
    File::open(filename)
}

fn main() {
    // Pattern matching
    match open_file("test.txt") {
        Ok(file) => println!("File opened successfully"),
        Err(error) => match error.kind() {
            ErrorKind::NotFound => println!("File not found"),
            ErrorKind::PermissionDenied => println!("Permission denied"),
            other_error => println!("Other error: {:?}", other_error),
        },
    }
    
    // Using if let
    if let Ok(file) = open_file("test.txt") {
        println!("File opened with if let");
    }
    
    // Unwrap variants (use carefully!)
    // let file1 = open_file("test.txt").unwrap();                    // Panics on error
    // let file2 = open_file("test.txt").expect("Failed to open");    // Panics with message
}
```

---

## The ? Operator: Error Propagation Made Easy

### Basic ? Usage

```rust
use std::fs::File;
use std::io::{self, Read};

// Without ? operator (verbose)
fn read_file_old_way(filename: &str) -> Result<String, io::Error> {
    let mut file = match File::open(filename) {
        Ok(file) => file,
        Err(e) => return Err(e),
    };
    
    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => Ok(contents),
        Err(e) => Err(e),
    }
}

// With ? operator (concise)
fn read_file_new_way(filename: &str) -> Result<String, io::Error> {
    let mut file = File::open(filename)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

// Even more concise
fn read_file_shortest(filename: &str) -> Result<String, io::Error> {
    std::fs::read_to_string(filename)
}
```

---

## Custom Error Types

### Simple Custom Errors

```rust
use std::fmt;

#[derive(Debug)]
enum MathError {
    DivisionByZero,
    NegativeSquareRoot,
}

impl fmt::Display for MathError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MathError::DivisionByZero => write!(f, "Cannot divide by zero"),
            MathError::NegativeSquareRoot => write!(f, "Cannot take square root of negative number"),
        }
    }
}

impl std::error::Error for MathError {}

fn divide(a: f64, b: f64) -> Result<f64, MathError> {
    if b == 0.0 {
        Err(MathError::DivisionByZero)
    } else {
        Ok(a / b)
    }
}

fn square_root(x: f64) -> Result<f64, MathError> {
    if x < 0.0 {
        Err(MathError::NegativeSquareRoot)
    } else {
        Ok(x.sqrt())
    }
}
```

---

## Key Takeaways

1. **Use Result<T, E>** for recoverable errors, panic! for unrecoverable ones
2. **The ? operator** makes error propagation clean and efficient
3. **Custom error types** should implement Display and Error traits
4. **Test error cases** as thoroughly as success cases
5. **Zero-cost abstractions** - proper error handling is fast

**Next Up:** In Chapter 11, we'll explore iterators and closures - Rust's functional programming features that make data processing both efficient and expressive.
