# Chapter 12: Macros - Code Generation and Metaprogramming

## Learning Objectives
- Understand declarative macros (macro_rules!)
- Learn procedural macros basics
- Compare with C++ preprocessor and C# source generators
- Write your own macros for DRY code
- Recognize common macro patterns in Rust

## Introduction

Rust macros are not text substitution like C preprocessor macros. They operate on the Abstract Syntax Tree (AST), making them hygienic and type-aware.

## Declarative Macros (macro_rules!)

### Basic Syntax

```rust
// Simple macro
macro_rules! say_hello {
    () => {
        println!("Hello, world!");
    };
}

// Macro with parameters
macro_rules! create_function {
    ($func_name:ident) => {
        fn $func_name() {
            println!("You called {:?}()", stringify!($func_name));
        }
    };
}

// Usage
fn main() {
    say_hello!();
    
    create_function!(foo);
    create_function!(bar);
    
    foo();  // Prints: You called "foo"()
    bar();  // Prints: You called "bar"()
}
```

### Pattern Matching in Macros

```rust
macro_rules! calculate {
    // Match single expression
    (eval $e:expr) => {
        {
            let val: usize = $e;
            println!("{} = {}", stringify!{$e}, val);
        }
    };
    
    // Match multiple patterns
    (add $a:expr, $b:expr) => {
        $a + $b
    };
    
    (mul $a:expr, $b:expr) => {
        $a * $b
    };
}

fn main() {
    calculate!(eval 1 + 2 * 3);        // Prints: 1 + 2 * 3 = 7
    let sum = calculate!(add 10, 20);  // Returns 30
    let product = calculate!(mul 4, 5); // Returns 20
}
```

### Repetition Patterns

```rust
macro_rules! vec_of_strings {
    ($($x:expr),*) => {
        {
            let mut temp_vec = Vec::new();
            $(
                temp_vec.push($x.to_string());
            )*
            temp_vec
        }
    };
}

macro_rules! create_struct {
    ($name:ident { $($field:ident: $type:ty),* }) => {
        #[derive(Debug)]
        struct $name {
            $($field: $type),*
        }
    };
}

fn main() {
    let v = vec_of_strings!["hello", "world", "rust"];
    
    create_struct!(Person {
        name: String,
        age: u32,
        email: String
    });
    
    let p = Person {
        name: "Alice".to_string(),
        age: 30,
        email: "alice@example.com".to_string(),
    };
}
```

### Common Standard Library Macros

```rust
// vec! - Create vectors
let v = vec![1, 2, 3, 4, 5];

// format! - Format strings
let s = format!("Hello, {}!", "world");

// println!/eprintln! - Print with formatting
println!("Debug: {:?}", some_struct);
eprintln!("Error: {}", error_message);

// assert!/assert_eq!/assert_ne! - Testing
assert!(x > 0, "x must be positive");
assert_eq!(result, expected);

// dbg! - Debug printing
let x = dbg!(5 + 3);  // Prints: [src/main.rs:10] 5 + 3 = 8

// include_str!/include_bytes! - Compile-time file inclusion
const CONFIG: &str = include_str!("config.txt");

// concat!/stringify! - Compile-time string operations
const VERSION: &str = concat!("v", env!("CARGO_PKG_VERSION"));

// todo!/unimplemented!/unreachable! - Development helpers
fn future_feature() {
    todo!("Implement this later")
}
```

## Comparison with C++ Preprocessor

| Feature | Rust Macros | C++ Preprocessor |
|---------|-------------|------------------|
| Type safety | Yes (AST-based) | No (text substitution) |
| Hygiene | Yes (no name collisions) | No |
| Debugging | Good error messages | Difficult |
| Recursion | Supported | Limited |
| Pattern matching | Yes | No |
| IDE support | Good | Poor |

### C++ Preprocessor Example
```cpp
// C++ - Text substitution, error-prone
#define MAX(a, b) ((a) > (b) ? (a) : (b))
int x = MAX(i++, j++);  // Undefined behavior!

// C++ - No type safety
#define MULTIPLY(x, y) x * y
int result = MULTIPLY(2 + 3, 4 + 5);  // Wrong: 2 + 3 * 4 + 5 = 19
```

### Rust Macro Equivalent
```rust
// Rust - Hygienic, type-safe
macro_rules! max {
    ($a:expr, $b:expr) => {
        {
            let a = $a;
            let b = $b;
            if a > b { a } else { b }
        }
    };
}

let x = max!(i, j);  // No side effects, i and j evaluated once

macro_rules! multiply {
    ($x:expr, $y:expr) => {
        ($x) * ($y)  // Parentheses ensure correct precedence
    };
}

let result = multiply!(2 + 3, 4 + 5);  // Correct: (2 + 3) * (4 + 5) = 45
```

## Writing Useful Macros

### Builder Pattern Macro

```rust
macro_rules! builder {
    ($name:ident { $($field:ident: $type:ty),* }) => {
        #[derive(Debug, Default)]
        pub struct $name {
            $($field: Option<$type>),*
        }
        
        impl $name {
            pub fn new() -> Self {
                Default::default()
            }
            
            $(
                pub fn $field(mut self, value: $type) -> Self {
                    self.$field = Some(value);
                    self
                }
            )*
            
            pub fn build(self) -> Result<($($type),*), &'static str> {
                Ok((
                    $(
                        self.$field.ok_or(concat!(stringify!($field), " is required"))?
                    ),*
                ))
            }
        }
    };
}

// Usage
builder!(PersonBuilder {
    name: String,
    age: u32,
    email: String
});

fn main() {
    let result = PersonBuilder::new()
        .name("Alice".to_string())
        .age(30)
        .email("alice@example.com".to_string())
        .build();
    
    match result {
        Ok((name, age, email)) => {
            println!("Built person: {} ({}) - {}", name, age, email);
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

### Test Generation Macro

```rust
macro_rules! generate_tests {
    ($($name:ident: $value:expr, $expected:expr),*) => {
        $(
            #[test]
            fn $name() {
                let result = process($value);
                assert_eq!(result, $expected);
            }
        )*
    };
}

fn process(x: i32) -> i32 {
    x * 2
}

generate_tests! {
    test_zero: 0, 0,
    test_one: 1, 2,
    test_negative: -5, -10,
    test_large: 1000, 2000
}
```

## Procedural Macros (Brief Introduction)

Procedural macros are more powerful but require a separate crate. They come in three types:

### 1. Derive Macros

```rust
// In your code
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Person {
    name: String,
    age: u32,
}

// Custom derive (requires proc-macro crate)
#[derive(MyTrait)]
struct MyStruct {
    field: String,
}
```

### 2. Attribute Macros

```rust
// Like #[test] or #[tokio::main]
#[route(GET, "/")]
fn index() -> &'static str {
    "Hello, World!"
}

#[cached]
fn expensive_computation(n: u64) -> u64 {
    // Function result will be cached
    fibonacci(n)
}
```

### 3. Function-like Procedural Macros

```rust
// More powerful than macro_rules!
let sql = sql!(SELECT * FROM users WHERE age > 18);

html! {
    <div class="container">
        <h1>{"Hello, World!"}</h1>
    </div>
}
```

## Macro Hygiene

Rust macros are hygienic - they don't accidentally capture or pollute namespaces:

```rust
macro_rules! using_a {
    ($e:expr) => {
        {
            let a = 42;
            $e
        }
    };
}

fn main() {
    let a = "outer";
    let result = using_a!(a);  // Uses outer 'a', not macro's 'a'
    println!("{}", result);     // Prints: "outer"
}
```

## When to Use Macros

### Good Use Cases
1. **Eliminating boilerplate** - Derive macros, builders
2. **Domain-specific languages** - SQL, HTML, routing
3. **Conditional compilation** - Platform-specific code
4. **Testing utilities** - Test generation, benchmarking
5. **Performance** - Zero-cost abstractions

### When NOT to Use Macros
1. When a function would work
2. When it makes code harder to understand
3. For simple type aliasing (use `type` instead)
4. When generics would be clearer

## Common Pitfalls

### 1. Forgetting Semicolons
```rust
macro_rules! bad {
    () => {
        let x = 5  // Missing semicolon!
    };
}

macro_rules! good {
    () => {
        let x = 5;  // Semicolon included
    };
}
```

### 2. Multiple Evaluation
```rust
macro_rules! bad_twice {
    ($x:expr) => {
        $x + $x  // $x evaluated twice
    };
}

macro_rules! good_twice {
    ($x:expr) => {
        {
            let val = $x;  // Evaluate once
            val + val
        }
    };
}
```

### 3. Type Inference Issues
```rust
macro_rules! make_vec {
    () => {
        Vec::new()  // Type unknown
    };
}

macro_rules! make_vec_typed {
    ($t:ty) => {
        Vec::<$t>::new()
    };
}
```

## Debugging Macros

### Techniques

```rust
// 1. Use trace_macros! (nightly)
#![feature(trace_macros)]
trace_macros!(true);
my_macro!(args);
trace_macros!(false);

// 2. Use log_syntax! (nightly)
#![feature(log_syntax)]
macro_rules! debug_macro {
    ($x:expr) => {
        log_syntax!($x);
        $x
    };
}

// 3. Expand macros with cargo expand
// cargo install cargo-expand
// cargo expand

// 4. Use compile_error! for debugging
macro_rules! check {
    ($condition:expr) => {
        #[cfg(not($condition))]
        compile_error!("Condition not met");
    };
}
```

## Exercises

### Exercise 12.1: Create a HashMap Macro
Write a macro that creates a HashMap with initial values:
```rust
let map = hashmap! {
    "one" => 1,
    "two" => 2,
    "three" => 3
};
```

### Exercise 12.2: Enum Visitor Macro
Create a macro that generates a visitor pattern for an enum:
```rust
generate_visitor!(MyEnum {
    Variant1(String),
    Variant2(i32, i32),
    Variant3 { field: bool }
});
```

### Exercise 12.3: Benchmark Macro
Write a macro that times code execution:
```rust
benchmark! {
    "sorting" => {
        let mut v = vec![3, 1, 4, 1, 5];
        v.sort();
    }
}
```

## Key Takeaways

✅ **Rust macros are hygienic and type-aware** - Not text substitution like C++

✅ **macro_rules! for pattern-based code generation** - Powerful and safe

✅ **Procedural macros for complex transformations** - Derive, attributes, function-like

✅ **Use macros to eliminate boilerplate** - But don't overuse them

✅ **Standard library has many useful macros** - vec!, format!, dbg!, etc.

✅ **Macros operate on AST, not text** - Safer than preprocessor macros

---

Next: [Chapter 13: Testing & Documentation](../day3/13_testing.md)