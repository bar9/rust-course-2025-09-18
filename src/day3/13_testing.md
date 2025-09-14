# Chapter 13: Testing & Documentation
## Unit Tests, Integration Tests, Documentation Tests, and Test Organization

### Learning Objectives
By the end of this chapter, you'll be able to:
- Write effective unit tests using the built-in test framework
- Organize and structure integration tests properly
- Create and maintain documentation tests
- Use test attributes and conditional compilation
- Mock dependencies and external systems
- Benchmark code performance
- Apply test-driven development (TDD) practices in Rust
- Debug failing tests effectively

---

## Rust Testing Philosophy vs Other Languages

### Comparison with Other Testing Frameworks

| Feature | C++ (Google Test) | C# (NUnit/MSTest) | Java (JUnit) | Rust (built-in) |
|---------|------------------|-------------------|--------------|------------------|
| Built-in framework | No | No | No | Yes |
| Doc tests | No | Limited | No | Yes |
| Parallel execution | Manual | Yes | Yes | Yes (default) |
| Mocking | External libs | Built-in/external | External libs | External libs |
| Benchmarking | External | External | External | Built-in (nightly) |
| Test discovery | Manual/CMake | Automatic | Automatic | Automatic |

### Basic Test Structure

```rust
// lib.rs or main.rs
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn divide(a: f64, b: f64) -> Result<f64, &'static str> {
    if b == 0.0 {
        Err("Division by zero")
    } else {
        Ok(a / b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
        assert_eq!(add(-1, 1), 0);
        assert_eq!(add(0, 0), 0);
    }

    #[test]
    fn test_divide_success() {
        assert_eq!(divide(10.0, 2.0), Ok(5.0));
        assert_eq!(divide(7.0, 2.0), Ok(3.5));
    }

    #[test]
    fn test_divide_by_zero() {
        assert_eq!(divide(10.0, 0.0), Err("Division by zero"));
    }

    #[test]
    #[should_panic]
    fn test_panic_case() {
        panic!("This test should panic");
    }

    #[test]
    #[should_panic(expected = "specific error message")]
    fn test_specific_panic() {
        panic!("specific error message");
    }
}
```

---

## Unit Testing

### Test Organization and Attributes

```rust
// src/calculator.rs
pub struct Calculator {
    memory: f64,
}

impl Calculator {
    pub fn new() -> Self {
        Calculator { memory: 0.0 }
    }

    pub fn add(&mut self, value: f64) -> f64 {
        self.memory += value;
        self.memory
    }

    pub fn multiply(&mut self, value: f64) -> f64 {
        self.memory *= value;
        self.memory
    }

    pub fn clear(&mut self) {
        self.memory = 0.0;
    }

    pub fn get_memory(&self) -> f64 {
        self.memory
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function for tests
    fn setup_calculator() -> Calculator {
        Calculator::new()
    }

    #[test]
    fn test_new_calculator() {
        let calc = Calculator::new();
        assert_eq!(calc.get_memory(), 0.0);
    }

    #[test]
    fn test_add_operation() {
        let mut calc = setup_calculator();
        assert_eq!(calc.add(5.0), 5.0);
        assert_eq!(calc.add(3.0), 8.0);
    }

    #[test]
    fn test_multiply_operation() {
        let mut calc = setup_calculator();
        calc.add(4.0);
        assert_eq!(calc.multiply(3.0), 12.0);
    }

    #[test]
    fn test_clear_memory() {
        let mut calc = setup_calculator();
        calc.add(10.0);
        calc.clear();
        assert_eq!(calc.get_memory(), 0.0);
    }

    // Test with custom error messages
    #[test]
    fn test_with_custom_message() {
        let calc = Calculator::new();
        assert_eq!(
            calc.get_memory(), 
            0.0, 
            "New calculator should start with zero memory"
        );
    }

    // Ignored test (won't run by default)
    #[test]
    #[ignore]
    fn expensive_test() {
        // This test takes a long time and is usually skipped
        // Run with: cargo test -- --ignored
        std::thread::sleep(std::time::Duration::from_secs(1));
        assert!(true);
    }

    // Test that only runs on specific platforms
    #[test]
    #[cfg(target_os = "linux")]
    fn linux_specific_test() {
        // This test only runs on Linux
        assert!(true);
    }
}
```

### Advanced Testing Patterns

```rust
use std::collections::HashMap;

pub struct UserService {
    users: HashMap<u32, String>,
    next_id: u32,
}

impl UserService {
    pub fn new() -> Self {
        UserService {
            users: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn add_user(&mut self, name: String) -> u32 {
        let id = self.next_id;
        self.users.insert(id, name);
        self.next_id += 1;
        id
    }

    pub fn get_user(&self, id: u32) -> Option<&String> {
        self.users.get(&id)
    }

    pub fn update_user(&mut self, id: u32, name: String) -> Result<(), &'static str> {
        if self.users.contains_key(&id) {
            self.users.insert(id, name);
            Ok(())
        } else {
            Err("User not found")
        }
    }

    pub fn delete_user(&mut self, id: u32) -> Result<String, &'static str> {
        self.users.remove(&id).ok_or("User not found")
    }

    pub fn user_count(&self) -> usize {
        self.users.len()
    }
}

#[cfg(test)]
mod user_service_tests {
    use super::*;

    // Fixture setup
    fn setup_service_with_users() -> (UserService, Vec<u32>) {
        let mut service = UserService::new();
        let mut ids = Vec::new();
        
        ids.push(service.add_user("Alice".to_string()));
        ids.push(service.add_user("Bob".to_string()));
        ids.push(service.add_user("Charlie".to_string()));
        
        (service, ids)
    }

    #[test]
    fn test_add_and_get_user() {
        let mut service = UserService::new();
        let id = service.add_user("John".to_string());
        
        assert_eq!(service.get_user(id), Some(&"John".to_string()));
        assert_eq!(service.user_count(), 1);
    }

    #[test]
    fn test_update_existing_user() {
        let (mut service, ids) = setup_service_with_users();
        
        let result = service.update_user(ids[0], "Alice Smith".to_string());
        assert!(result.is_ok());
        assert_eq!(service.get_user(ids[0]), Some(&"Alice Smith".to_string()));
    }

    #[test]
    fn test_update_nonexistent_user() {
        let mut service = UserService::new();
        let result = service.update_user(999, "Nobody".to_string());
        assert_eq!(result, Err("User not found"));
    }

    #[test]
    fn test_delete_user() {
        let (mut service, ids) = setup_service_with_users();
        
        let deleted = service.delete_user(ids[1]);
        assert_eq!(deleted, Ok("Bob".to_string()));
        assert_eq!(service.user_count(), 2);
        assert_eq!(service.get_user(ids[1]), None);
    }

    // Parameterized test pattern
    #[test]
    fn test_multiple_scenarios() {
        let test_cases = vec![
            ("Alice", 1),
            ("Bob", 2),
            ("", 3), // Edge case: empty string
            ("Very Long Name That Might Cause Issues", 4),
        ];

        let mut service = UserService::new();
        
        for (name, expected_id) in test_cases {
            let id = service.add_user(name.to_string());
            assert_eq!(id, expected_id);
            assert_eq!(service.get_user(id), Some(&name.to_string()));
        }
    }
}
```

---

## Integration Tests

### Test Directory Structure

```
my_project/
├── src/
│   ├── lib.rs
│   └── calculator.rs
├── tests/
│   ├── integration_test.rs
│   ├── api_tests.rs
│   └── common/
│       └── mod.rs
└── Cargo.toml
```

### Integration Test Example

```rust
// tests/integration_test.rs
use my_project::Calculator;

#[test]
fn test_calculator_integration() {
    let mut calc = Calculator::new();
    
    // Test a complete workflow
    calc.add(10.0);
    calc.multiply(2.0);
    calc.add(5.0);
    
    assert_eq!(calc.get_memory(), 25.0);
}

#[test]
fn test_calculator_multiple_instances() {
    let mut calc1 = Calculator::new();
    let mut calc2 = Calculator::new();
    
    calc1.add(10.0);
    calc2.add(20.0);
    
    assert_eq!(calc1.get_memory(), 10.0);
    assert_eq!(calc2.get_memory(), 20.0);
    
    // They should be independent
    calc1.clear();
    assert_eq!(calc1.get_memory(), 0.0);
    assert_eq!(calc2.get_memory(), 20.0);
}
```

### Common Test Utilities

```rust
// tests/common/mod.rs
use std::fs;
use std::io;
use tempfile::tempdir;

pub fn setup_test_environment() -> io::Result<tempfile::TempDir> {
    let dir = tempdir()?;
    // Setup test files, directories, etc.
    Ok(dir)
}

pub fn create_test_file(dir: &std::path::Path, name: &str, content: &str) -> io::Result<()> {
    let file_path = dir.join(name);
    fs::write(file_path, content)
}

// tests/api_tests.rs
mod common;

use common::*;
use my_project::*;

#[test]
fn test_file_operations() {
    let temp_dir = setup_test_environment().unwrap();
    
    create_test_file(
        temp_dir.path(), 
        "test.txt", 
        "Hello, World!"
    ).unwrap();
    
    // Test your file processing logic here
    let file_path = temp_dir.path().join("test.txt");
    assert!(file_path.exists());
    
    // temp_dir is automatically cleaned up when it goes out of scope
}
```

---

## Documentation Tests

### Basic Documentation Tests

```rust
// src/math.rs

/// Calculates the factorial of a number.
/// 
/// # Examples
/// 
/// ```
/// use my_project::factorial;
/// 
/// assert_eq!(factorial(0), 1);
/// assert_eq!(factorial(1), 1);
/// assert_eq!(factorial(5), 120);
/// ```
/// 
/// # Panics
/// 
/// This function will panic if the input is negative:
/// 
/// ```should_panic
/// use my_project::factorial;
/// 
/// factorial(-1); // This will panic
/// ```
pub fn factorial(n: i32) -> i32 {
    match n {
        0 | 1 => 1,
        n if n < 0 => panic!("Factorial is not defined for negative numbers"),
        _ => n * factorial(n - 1),
    }
}

/// Performs integer division with proper error handling.
/// 
/// # Examples
/// 
/// Basic usage:
/// ```
/// use my_project::safe_divide;
/// 
/// assert_eq!(safe_divide(10, 2), Ok(5));
/// assert_eq!(safe_divide(10, 3), Ok(3)); // Integer division
/// ```
/// 
/// Error handling:
/// ```
/// use my_project::safe_divide;
/// 
/// assert!(safe_divide(10, 0).is_err());
/// match safe_divide(10, 0) {
///     Ok(_) => panic!("Should not succeed"),
///     Err(e) => assert_eq!(e, "Division by zero"),
/// }
/// ```
/// 
/// # Errors
/// 
/// Returns an error if the divisor is zero.
pub fn safe_divide(dividend: i32, divisor: i32) -> Result<i32, &'static str> {
    if divisor == 0 {
        Err("Division by zero")
    } else {
        Ok(dividend / divisor)
    }
}

/// A simple calculator struct.
/// 
/// # Examples
/// 
/// ```
/// use my_project::SimpleCalculator;
/// 
/// let mut calc = SimpleCalculator::new();
/// calc.add(5);
/// calc.multiply(2);
/// assert_eq!(calc.result(), 10);
/// ```
pub struct SimpleCalculator {
    value: i32,
}

impl SimpleCalculator {
    /// Creates a new calculator with value 0.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use my_project::SimpleCalculator;
    /// 
    /// let calc = SimpleCalculator::new();
    /// assert_eq!(calc.result(), 0);
    /// ```
    pub fn new() -> Self {
        SimpleCalculator { value: 0 }
    }

    /// Adds a value to the calculator.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use my_project::SimpleCalculator;
    /// 
    /// let mut calc = SimpleCalculator::new();
    /// calc.add(5);
    /// calc.add(3);
    /// assert_eq!(calc.result(), 8);
    /// ```
    pub fn add(&mut self, value: i32) {
        self.value += value;
    }

    /// Multiplies the current value by the given value.
    pub fn multiply(&mut self, value: i32) {
        self.value *= value;
    }

    /// Returns the current result.
    pub fn result(&self) -> i32 {
        self.value
    }
}
```

### Advanced Documentation Test Patterns

```rust
/// A configuration parser with various options.
/// 
/// # Examples
/// 
/// ```
/// use my_project::ConfigParser;
/// use std::collections::HashMap;
/// 
/// let mut parser = ConfigParser::new();
/// parser.set("debug", "true");
/// parser.set("port", "8080");
/// 
/// assert_eq!(parser.get("debug"), Some("true"));
/// assert_eq!(parser.get_int("port"), Ok(8080));
/// ```
/// 
/// Error handling:
/// ```
/// use my_project::ConfigParser;
/// 
/// let parser = ConfigParser::new();
/// assert!(parser.get_int("nonexistent").is_err());
/// ```
/// 
/// You can also test compilation failures:
/// ```compile_fail
/// use my_project::ConfigParser;
/// 
/// let parser = ConfigParser::new();
/// parser.set("key", 123); // This should fail to compile
/// ```
pub struct ConfigParser {
    config: std::collections::HashMap<String, String>,
}

impl ConfigParser {
    pub fn new() -> Self {
        ConfigParser {
            config: std::collections::HashMap::new(),
        }
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.config.insert(key.to_string(), value.to_string());
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.config.get(key).map(|s| s.as_str())
    }

    pub fn get_int(&self, key: &str) -> Result<i32, Box<dyn std::error::Error>> {
        match self.get(key) {
            Some(value) => Ok(value.parse()?),
            None => Err(format!("Key '{}' not found", key).into()),
        }
    }
}
```

---

## Test Organization and Best Practices

### Test Configuration in Cargo.toml

```toml
# Cargo.toml
[package]
name = "my_project"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }

[dev-dependencies]
tempfile = "3.0"
mockall = "0.11"
criterion = "0.5"
proptest = "1.0"

# Test profiles
[profile.test]
opt-level = 1  # Faster compilation for tests

# Benchmark configuration
[[bench]]
name = "my_benchmarks"
harness = false
```

### Property-Based Testing

```rust
// Add to [dev-dependencies]: proptest = "1.0"

use proptest::prelude::*;

fn reverse_string(s: &str) -> String {
    s.chars().rev().collect()
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_reverse_twice_is_identity(s in ".*") {
            let reversed_twice = reverse_string(&reverse_string(&s));
            prop_assert_eq!(s, reversed_twice);
        }

        #[test]
        fn test_reverse_length_unchanged(s in ".*") {
            let reversed = reverse_string(&s);
            prop_assert_eq!(s.len(), reversed.len());
        }

        #[test]
        fn test_factorial_properties(n in 0u32..10) {
            let result = factorial(n as i32);
            prop_assert!(result > 0);
            if n > 0 {
                prop_assert_eq!(result, n as i32 * factorial((n - 1) as i32));
            }
        }
    }
}
```

### Mocking with Mockall

```rust
// Add to [dev-dependencies]: mockall = "0.11"

use mockall::{automock, predicate::*};

#[automock]
trait DatabaseService {
    fn get_user(&self, id: u32) -> Result<String, String>;
    fn save_user(&mut self, id: u32, name: String) -> Result<(), String>;
}

struct UserManager<D: DatabaseService> {
    db: D,
}

impl<D: DatabaseService> UserManager<D> {
    fn new(db: D) -> Self {
        UserManager { db }
    }

    fn get_user_display_name(&self, id: u32) -> String {
        match self.db.get_user(id) {
            Ok(name) => format!("User: {}", name),
            Err(_) => "Unknown User".to_string(),
        }
    }

    fn update_user(&mut self, id: u32, name: String) -> Result<String, String> {
        self.db.save_user(id, name.clone())?;
        Ok(format!("Updated user {} to {}", id, name))
    }
}

#[cfg(test)]
mod mock_tests {
    use super::*;

    #[test]
    fn test_get_user_display_name_success() {
        let mut mock_db = MockDatabaseService::new();
        mock_db
            .expect_get_user()
            .with(eq(1))
            .times(1)
            .returning(|_| Ok("Alice".to_string()));

        let manager = UserManager::new(mock_db);
        let result = manager.get_user_display_name(1);
        assert_eq!(result, "User: Alice");
    }

    #[test]
    fn test_get_user_display_name_error() {
        let mut mock_db = MockDatabaseService::new();
        mock_db
            .expect_get_user()
            .with(eq(999))
            .times(1)
            .returning(|_| Err("User not found".to_string()));

        let manager = UserManager::new(mock_db);
        let result = manager.get_user_display_name(999);
        assert_eq!(result, "Unknown User");
    }

    #[test]
    fn test_update_user() {
        let mut mock_db = MockDatabaseService::new();
        mock_db
            .expect_save_user()
            .with(eq(1), eq("Bob".to_string()))
            .times(1)
            .returning(|_, _| Ok(()));

        let mut manager = UserManager::new(mock_db);
        let result = manager.update_user(1, "Bob".to_string());
        assert_eq!(result, Ok("Updated user 1 to Bob".to_string()));
    }
}
```

---

## Benchmarking

### Basic Benchmarks with Criterion

```rust
// benches/my_benchmarks.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use my_project::*;

fn bench_factorial(c: &mut Criterion) {
    c.bench_function("factorial 10", |b| {
        b.iter(|| factorial(black_box(10)))
    });
    
    c.bench_function("factorial 20", |b| {
        b.iter(|| factorial(black_box(20)))
    });
}

fn bench_string_operations(c: &mut Criterion) {
    let test_string = "Hello, World! This is a test string for benchmarking.";
    
    c.bench_function("reverse string", |b| {
        b.iter(|| reverse_string(black_box(test_string)))
    });
    
    c.bench_function("reverse and reverse again", |b| {
        b.iter(|| {
            let reversed = reverse_string(black_box(test_string));
            reverse_string(&reversed)
        })
    });
}

fn bench_calculator_operations(c: &mut Criterion) {
    c.bench_function("calculator workflow", |b| {
        b.iter(|| {
            let mut calc = SimpleCalculator::new();
            calc.add(black_box(100));
            calc.multiply(black_box(2));
            calc.add(black_box(50));
            calc.result()
        })
    });
}

criterion_group!(
    benches, 
    bench_factorial, 
    bench_string_operations,
    bench_calculator_operations
);
criterion_main!(benches);
```

---

## Test-Driven Development (TDD) Example

```rust
// Following TDD: Red -> Green -> Refactor

// Step 1: Write failing tests first
#[cfg(test)]
mod stack_tests {
    use super::*;

    #[test]
    fn test_new_stack_is_empty() {
        let stack = Stack::<i32>::new();
        assert!(stack.is_empty());
        assert_eq!(stack.size(), 0);
    }

    #[test]
    fn test_push_item() {
        let mut stack = Stack::new();
        stack.push(42);
        assert!(!stack.is_empty());
        assert_eq!(stack.size(), 1);
    }

    #[test]
    fn test_pop_item() {
        let mut stack = Stack::new();
        stack.push(42);
        assert_eq!(stack.pop(), Some(42));
        assert!(stack.is_empty());
    }

    #[test]
    fn test_pop_empty_stack() {
        let mut stack = Stack::<i32>::new();
        assert_eq!(stack.pop(), None);
    }

    #[test]
    fn test_peek_item() {
        let mut stack = Stack::new();
        stack.push(42);
        assert_eq!(stack.peek(), Some(&42));
        assert_eq!(stack.size(), 1); // Should not modify stack
    }

    #[test]
    fn test_stack_lifo_order() {
        let mut stack = Stack::new();
        stack.push(1);
        stack.push(2);
        stack.push(3);
        
        assert_eq!(stack.pop(), Some(3));
        assert_eq!(stack.pop(), Some(2));
        assert_eq!(stack.pop(), Some(1));
        assert_eq!(stack.pop(), None);
    }
}

// Step 2: Implement minimum code to make tests pass
pub struct Stack<T> {
    items: Vec<T>,
}

impl<T> Stack<T> {
    pub fn new() -> Self {
        Stack { items: Vec::new() }
    }

    pub fn push(&mut self, item: T) {
        self.items.push(item);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.items.pop()
    }

    pub fn peek(&self) -> Option<&T> {
        self.items.last()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn size(&self) -> usize {
        self.items.len()
    }
}

// Step 3: Refactor and add more tests for edge cases
#[cfg(test)]
mod advanced_stack_tests {
    use super::*;

    #[test]
    fn test_stack_with_different_types() {
        let mut string_stack = Stack::new();
        string_stack.push("hello".to_string());
        string_stack.push("world".to_string());
        
        assert_eq!(string_stack.pop(), Some("world".to_string()));
    }

    #[test]
    fn test_large_stack() {
        let mut stack = Stack::new();
        
        // Push many items
        for i in 0..1000 {
            stack.push(i);
        }
        
        assert_eq!(stack.size(), 1000);
        
        // Pop them all
        for i in (0..1000).rev() {
            assert_eq!(stack.pop(), Some(i));
        }
        
        assert!(stack.is_empty());
    }
}
```

---

## Common Testing Pitfalls and Best Practices

### Pitfall 1: Non-Deterministic Tests

```rust
// BAD: Test that depends on timing or randomness
#[test]
fn bad_timing_test() {
    use std::time::{Instant, Duration};
    
    let start = Instant::now();
    std::thread::sleep(Duration::from_millis(100));
    let elapsed = start.elapsed();
    
    // This might fail on slow systems or under load
    assert!(elapsed >= Duration::from_millis(100));
    assert!(elapsed < Duration::from_millis(110)); // Too strict!
}

// GOOD: Deterministic test with controlled conditions
#[test]
fn good_deterministic_test() {
    let mut calculator = Calculator::new();
    
    calculator.add(5.0);
    calculator.multiply(2.0);
    
    assert_eq!(calculator.get_memory(), 10.0);
}
```

### Pitfall 2: Testing Implementation Instead of Behavior

```rust
// BAD: Testing internal implementation details
#[test]
fn bad_implementation_test() {
    let mut vec = Vec::new();
    vec.push(1);
    vec.push(2);
    
    // Don't test internal capacity or specific implementation details
    // This test is brittle and doesn't test actual behavior
    assert!(vec.capacity() >= 2);
}

// GOOD: Testing public behavior
#[test]
fn good_behavior_test() {
    let mut vec = Vec::new();
    vec.push(1);
    vec.push(2);
    
    // Test the actual behavior users care about
    assert_eq!(vec.len(), 2);
    assert_eq!(vec[0], 1);
    assert_eq!(vec[1], 2);
}
```

### Best Practices

```rust
// 1. Use descriptive test names
#[test]
fn should_return_error_when_dividing_by_zero() {
    let result = safe_divide(10, 0);
    assert!(result.is_err());
}

// 2. Test one thing at a time
#[test]
fn should_add_user_and_return_id() {
    let mut service = UserService::new();
    let id = service.add_user("Alice".to_string());
    
    assert_eq!(id, 1);
}

#[test]
fn should_retrieve_added_user() {
    let mut service = UserService::new();
    let id = service.add_user("Alice".to_string());
    
    assert_eq!(service.get_user(id), Some(&"Alice".to_string()));
}

// 3. Use setup and teardown appropriately
#[cfg(test)]
mod user_tests {
    use super::*;
    
    fn setup() -> UserService {
        UserService::new()
    }
    
    #[test]
    fn test_user_creation() {
        let mut service = setup();
        // Test logic here
    }
}

// 4. Test edge cases and error conditions
#[test]
fn should_handle_empty_input() {
    let result = process_data(&[]);
    assert!(result.is_ok());
}

#[test]
fn should_handle_very_large_input() {
    let large_data = vec![0; 10_000];
    let result = process_data(&large_data);
    assert!(result.is_ok());
}
```

---

## Exercises

### Exercise 1: Banking System Tests
Create comprehensive tests for a simple banking system:

```rust
#[derive(Debug, PartialEq)]
pub enum TransactionError {
    InsufficientFunds,
    AccountNotFound,
    InvalidAmount,
}

pub struct BankAccount {
    id: u32,
    balance: f64,
    is_active: bool,
}

pub struct Bank {
    accounts: std::collections::HashMap<u32, BankAccount>,
    next_id: u32,
}

// TODO: Implement these methods
impl Bank {
    pub fn new() -> Self {
        todo!()
    }
    
    pub fn create_account(&mut self, initial_deposit: f64) -> Result<u32, TransactionError> {
        todo!()
    }
    
    pub fn deposit(&mut self, account_id: u32, amount: f64) -> Result<f64, TransactionError> {
        todo!()
    }
    
    pub fn withdraw(&mut self, account_id: u32, amount: f64) -> Result<f64, TransactionError> {
        todo!()
    }
    
    pub fn transfer(&mut self, from_id: u32, to_id: u32, amount: f64) -> Result<(), TransactionError> {
        todo!()
    }
    
    pub fn get_balance(&self, account_id: u32) -> Result<f64, TransactionError> {
        todo!()
    }
    
    pub fn close_account(&mut self, account_id: u32) -> Result<f64, TransactionError> {
        todo!()
    }
}

// TODO: Write comprehensive tests covering:
// - Account creation with valid/invalid initial deposits
// - Deposit and withdrawal operations
// - Transfer between accounts
// - Error cases (insufficient funds, invalid accounts, etc.)
// - Edge cases (zero amounts, closed accounts, etc.)
```

### Exercise 2: Text Processor with Doc Tests
Create a text processing module with comprehensive documentation tests:

```rust
// TODO: Add comprehensive doc tests for each function

/// Counts the number of words in a text string.
pub fn word_count(text: &str) -> usize {
    todo!()
}

/// Finds the most frequent word in a text string.
pub fn most_frequent_word(text: &str) -> Option<String> {
    todo!()
}

/// Reverses the order of words in a sentence.
pub fn reverse_words(text: &str) -> String {
    todo!()
}

/// Extracts sentences from text (split on '.', '!', or '?').
pub fn extract_sentences(text: &str) -> Vec<String> {
    todo!()
}

/// Capitalizes the first letter of each word.
pub fn title_case(text: &str) -> String {
    todo!()
}
```

### Exercise 3: Property-Based Testing
Implement and test a simple hash map with property-based tests:

```rust
// TODO: Implement a simple hash map and write property-based tests
pub struct SimpleHashMap<K, V> {
    // Your implementation here
}

impl<K, V> SimpleHashMap<K, V> 
where 
    K: std::hash::Hash + Eq + Clone,
    V: Clone,
{
    pub fn new() -> Self {
        todo!()
    }
    
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        todo!()
    }
    
    pub fn get(&self, key: &K) -> Option<&V> {
        todo!()
    }
    
    pub fn remove(&mut self, key: &K) -> Option<V> {
        todo!()
    }
    
    pub fn len(&self) -> usize {
        todo!()
    }
    
    pub fn is_empty(&self) -> bool {
        todo!()
    }
}

// TODO: Write property-based tests that verify:
// - Inserting then getting returns the same value
// - Removing a key makes it unavailable
// - Length increases/decreases correctly
// - Keys that were never inserted return None
```

---

## Key Takeaways

1. **Built-in testing** - Rust has excellent testing support out of the box
2. **Documentation tests** - Keep examples in sync with code automatically
3. **Integration tests** - Test public APIs from the user's perspective
4. **Property-based testing** - Generate test cases to find edge cases
5. **Test organization** - Separate unit, integration, and doc tests appropriately
6. **Mocking** - Use external crates for complex dependency injection
7. **Benchmarking** - Measure performance with criterion
8. **TDD workflow** - Red-Green-Refactor helps design better APIs

**Next Up:** In Chapter 13, we'll explore concurrency - Rust's approach to safe parallel programming with threads, channels, and synchronization primitives.
