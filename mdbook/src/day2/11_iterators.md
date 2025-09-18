# Chapter 11: Iterators and Functional Programming
## Iterator Trait, Closures, and Lazy Evaluation in Rust

### Learning Objectives
By the end of this chapter, you'll be able to:
- Understand the Iterator trait and implement custom iterators
- Use iterator adaptors like map, filter, fold, and collect effectively
- Write and use closures with proper capture semantics
- Leverage lazy evaluation for performance
- Choose between imperative loops and functional iterator chains
- Handle iterator errors and edge cases
- Write efficient, readable functional-style Rust code

---

## Iterator Trait vs Other Languages

### Comparison with Other Languages

| Feature | C++ STL | C# LINQ | Java Streams | Rust Iterators |
|---------|---------|---------|--------------|----------------|
| Lazy evaluation | Partial | Yes | Yes | Yes |
| Zero-cost | Yes | No | No | Yes |
| Chaining | Limited | Extensive | Extensive | Extensive |
| Error handling | Exceptions | Exceptions | Exceptions | Result<T, E> |
| Memory safety | No | GC | GC | Compile-time |
| Parallel processing | Limited | PLINQ | Parallel streams | Rayon |

### The Iterator Trait

```rust
trait Iterator {
    type Item;
    
    // Required method
    fn next(&mut self) -> Option<Self::Item>;
    
    // Many default implementations built on next()
    fn collect<B: FromIterator<Self::Item>>(self) -> B { ... }
    fn map<B, F>(self, f: F) -> Map<Self, F> { ... }
    fn filter<P>(self, predicate: P) -> Filter<Self, P> { ... }
    // ... and many more
}

// Example: Custom iterator
struct Counter {
    current: u32,
    max: u32,
}

impl Counter {
    fn new(max: u32) -> Counter {
        Counter { current: 0, max }
    }
}

impl Iterator for Counter {
    type Item = u32;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.max {
            let current = self.current;
            self.current += 1;
            Some(current)
        } else {
            None
        }
    }
}

fn main() {
    let counter = Counter::new(5);
    for n in counter {
        println!("{}", n); // 0, 1, 2, 3, 4
    }
}
```

---

## Creating Iterators

### From Collections

```rust
fn main() {
    let vec = vec![1, 2, 3, 4, 5];
    
    // iter() - borrows elements
    for item in vec.iter() {
        println!("Borrowed: {}", item); // item is &i32
    }
    
    // into_iter() - takes ownership
    for item in vec.into_iter() {
        println!("Owned: {}", item); // item is i32
    }
    // vec is no longer accessible here
    
    let mut vec = vec![1, 2, 3, 4, 5];
    
    // iter_mut() - mutable borrows
    for item in vec.iter_mut() {
        *item *= 2; // item is &mut i32
    }
    println!("{:?}", vec); // [2, 4, 6, 8, 10]
}

// Range iterators
fn range_examples() {
    // Inclusive range
    for i in 0..=5 {
        println!("{}", i); // 0, 1, 2, 3, 4, 5
    }
    
    // Exclusive range
    let squares: Vec<i32> = (1..6)
        .map(|x| x * x)
        .collect();
    println!("{:?}", squares); // [1, 4, 9, 16, 25]
    
    // Step by
    let evens: Vec<i32> = (0..10)
        .step_by(2)
        .collect();
    println!("{:?}", evens); // [0, 2, 4, 6, 8]
}
```

### Custom Iterator Implementation

```rust
// Fibonacci iterator
struct Fibonacci {
    current: u64,
    next: u64,
}

impl Fibonacci {
    fn new() -> Self {
        Fibonacci { current: 0, next: 1 }
    }
}

impl Iterator for Fibonacci {
    type Item = u64;
    
    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current;
        self.current = self.next;
        self.next = current + self.next;
        
        // Prevent overflow
        if self.current > u64::MAX / 2 {
            None
        } else {
            Some(current)
        }
    }
}

// File line iterator
use std::fs::File;
use std::io::{BufRead, BufReader, Lines};

struct FileLines {
    lines: Lines<BufReader<File>>,
}

impl FileLines {
    fn new(file: File) -> Self {
        let reader = BufReader::new(file);
        FileLines {
            lines: reader.lines(),
        }
    }
}

impl Iterator for FileLines {
    type Item = Result<String, std::io::Error>;
    
    fn next(&mut self) -> Option<Self::Item> {
        self.lines.next()
    }
}

fn main() {
    // Fibonacci sequence
    let fib: Vec<u64> = Fibonacci::new()
        .take(10)
        .collect();
    println!("Fibonacci: {:?}", fib);
    
    // File processing (if file exists)
    if let Ok(file) = File::open("data.txt") {
        for line_result in FileLines::new(file) {
            match line_result {
                Ok(line) => println!("Line: {}", line),
                Err(e) => eprintln!("Error reading line: {}", e),
            }
        }
    }
}
```

---

## Iterator Adaptors

### Map, Filter, and Collect

```rust
fn basic_adaptors() {
    let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    
    // Chain multiple operations
    let result: Vec<i32> = numbers
        .iter()
        .filter(|&x| x % 2 == 0)  // Keep even numbers
        .map(|x| x * x)           // Square them
        .collect();               // Collect into Vec
    println!("Even squares: {:?}", result); // [4, 16, 36, 64, 100]
    
    // Different collection types
    use std::collections::HashSet;
    
    let unique_lengths: HashSet<usize> = vec!["hello", "world", "rust", "is", "awesome"]
        .iter()
        .map(|s| s.len())
        .collect();
    println!("Unique lengths: {:?}", unique_lengths);
    
    // Collect to String
    let concatenated: String = vec!["Hello", " ", "world", "!"]
        .iter()
        .cloned()
        .collect();
    println!("{}", concatenated); // Hello world!
}

// Working with Results
fn process_with_results() -> Result<Vec<i32>, std::num::ParseIntError> {
    let strings = vec!["1", "2", "3", "4", "not_a_number", "6"];
    
    // This will short-circuit on first error
    let numbers: Result<Vec<i32>, _> = strings
        .iter()
        .map(|s| s.parse::<i32>())
        .collect();
    
    numbers
}

fn process_filtering_errors() -> Vec<i32> {
    let strings = vec!["1", "2", "3", "4", "not_a_number", "6"];
    
    // Filter out errors, keep only successful parses
    strings
        .iter()
        .filter_map(|s| s.parse::<i32>().ok())
        .collect()
}
```

### Enumerate, Zip, and Take

```rust
fn advanced_adaptors() {
    let names = vec!["Alice", "Bob", "Charlie", "Diana"];
    let ages = vec![25, 30, 35, 28];
    
    // Enumerate - add indices
    for (index, name) in names.iter().enumerate() {
        println!("{}: {}", index, name);
    }
    
    // Zip - combine two iterators
    let people: Vec<(&&str, &i32)> = names
        .iter()
        .zip(ages.iter())
        .collect();
    
    for (name, age) in people {
        println!("{} is {} years old", name, age);
    }
    
    // Take and skip
    let first_three: Vec<&str> = names
        .iter()
        .take(3)
        .cloned()
        .collect();
    println!("First three: {:?}", first_three);
    
    let skip_first_two: Vec<&str> = names
        .iter()
        .skip(2)
        .cloned()
        .collect();
    println!("Skip first two: {:?}", skip_first_two);
    
    // Take while predicate is true
    let numbers = vec![1, 3, 5, 8, 9, 11];
    let odds_until_even: Vec<i32> = numbers
        .iter()
        .take_while(|&&x| x % 2 == 1)
        .cloned()
        .collect();
    println!("Odds until even: {:?}", odds_until_even); // [1, 3, 5]
}

// Chain and flatten
fn combining_iterators() {
    let vec1 = vec![1, 2, 3];
    let vec2 = vec![4, 5, 6];
    
    // Chain iterators
    let combined: Vec<i32> = vec1
        .iter()
        .chain(vec2.iter())
        .cloned()
        .collect();
    println!("Combined: {:?}", combined); // [1, 2, 3, 4, 5, 6]
    
    // Flatten nested structures
    let nested = vec![vec![1, 2], vec![3, 4, 5], vec![6]];
    let flattened: Vec<i32> = nested
        .iter()
        .flatten()
        .cloned()
        .collect();
    println!("Flattened: {:?}", flattened); // [1, 2, 3, 4, 5, 6]
    
    // flat_map - map then flatten
    let words = vec!["hello world", "rust programming"];
    let all_words: Vec<&str> = words
        .iter()
        .flat_map(|s| s.split_whitespace())
        .collect();
    println!("All words: {:?}", all_words); // ["hello", "world", "rust", "programming"]
}
```

---

## Closures and Capture

### Closure Syntax and Types

```rust
fn closure_basics() {
    let numbers = vec![1, 2, 3, 4, 5];
    
    // Different closure syntaxes
    let add_one = |x| x + 1;
    let add_two = |x: i32| -> i32 { x + 2 };
    let add_three = |x: i32| {
        let result = x + 3;
        result
    };
    
    // Using closures with iterators
    let incremented: Vec<i32> = numbers
        .iter()
        .map(|&x| add_one(x))
        .collect();
    println!("Incremented: {:?}", incremented);
    
    // Inline closures
    let evens: Vec<i32> = numbers
        .iter()
        .filter(|&&x| x % 2 == 0)
        .cloned()
        .collect();
    println!("Evens: {:?}", evens);
}

// Capture modes
fn capture_modes() {
    let multiplier = 10;
    let mut counter = 0;
    let mut data = vec![1, 2, 3];
    
    // Fn - immutable borrow
    let multiply_by = |x| x * multiplier;
    println!("5 * {} = {}", multiplier, multiply_by(5));
    
    // FnMut - mutable borrow
    let mut count_calls = || {
        counter += 1;
        counter
    };
    println!("Call count: {}", count_calls());
    println!("Call count: {}", count_calls());
    
    // FnOnce - takes ownership
    let consume_data = || {
        let owned_data = data; // Takes ownership
        owned_data.len()
    };
    println!("Data length: {}", consume_data());
    // data is no longer accessible
    
    // Move keyword forces ownership
    let value = 42;
    let thread_closure = move || {
        println!("Value in thread: {}", value);
    };
    // value is moved into closure
    
    std::thread::spawn(thread_closure).join().unwrap();
}
```

### Advanced Closure Patterns

```rust
// Higher-order functions
fn apply_twice<F>(f: F, x: i32) -> i32 
where 
    F: Fn(i32) -> i32,
{
    f(f(x))
}

fn create_multiplier(factor: i32) -> impl Fn(i32) -> i32 {
    move |x| x * factor
}

fn conditional_processor<F, G>(
    condition: bool,
    true_fn: F,
    false_fn: G,
) -> Box<dyn Fn(i32) -> i32>
where
    F: Fn(i32) -> i32 + 'static,
    G: Fn(i32) -> i32 + 'static,
{
    if condition {
        Box::new(true_fn)
    } else {
        Box::new(false_fn)
    }
}

fn main() {
    // Using higher-order functions
    let double = |x| x * 2;
    let result = apply_twice(double, 5);
    println!("Applied twice: {}", result); // 20
    
    // Factory functions
    let triple = create_multiplier(3);
    println!("Triple of 7: {}", triple(7)); // 21
    
    // Dynamic closure selection
    let processor = conditional_processor(
        true,
        |x| x * 2,
        |x| x + 10,
    );
    println!("Processed: {}", processor(5)); // 10
}

// Closure performance considerations
fn performance_comparison() {
    let data = (0..1_000_000).collect::<Vec<i32>>();
    
    // Functional style (often optimizes well)
    let start = std::time::Instant::now();
    let sum1: i32 = data
        .iter()
        .filter(|&&x| x % 2 == 0)
        .map(|&x| x * x)
        .sum();
    let duration1 = start.elapsed();
    
    // Imperative style
    let start = std::time::Instant::now();
    let mut sum2 = 0;
    for &x in &data {
        if x % 2 == 0 {
            sum2 += x * x;
        }
    }
    let duration2 = start.elapsed();
    
    println!("Functional: {} in {:?}", sum1, duration1);
    println!("Imperative: {} in {:?}", sum2, duration2);
}
```

---

## Reduction Operations

### Fold, Reduce, and Sum

```rust
fn reduction_operations() {
    let numbers = vec![1, 2, 3, 4, 5];
    
    // sum() - built-in reduction
    let sum: i32 = numbers.iter().sum();
    println!("Sum: {}", sum); // 15
    
    // fold() - with initial value
    let product = numbers
        .iter()
        .fold(1, |acc, &x| acc * x);
    println!("Product: {}", product); // 120
    
    // reduce() - no initial value (returns Option)
    let max = numbers
        .iter()
        .reduce(|acc, x| if acc > x { acc } else { x });
    println!("Max: {:?}", max); // Some(5)
    
    // Complex fold example: word frequency
    let text = "hello world hello rust world";
    let word_count = text
        .split_whitespace()
        .fold(std::collections::HashMap::new(), |mut acc, word| {
            *acc.entry(word).or_insert(0) += 1;
            acc
        });
    println!("Word count: {:?}", word_count);
}

fn advanced_reductions() {
    use std::collections::HashMap;
    
    #[derive(Debug)]
    struct Sale {
        product: String,
        amount: f64,
        region: String,
    }
    
    let sales = vec![
        Sale { product: "Widget".to_string(), amount: 100.0, region: "North".to_string() },
        Sale { product: "Gadget".to_string(), amount: 150.0, region: "South".to_string() },
        Sale { product: "Widget".to_string(), amount: 200.0, region: "North".to_string() },
        Sale { product: "Gadget".to_string(), amount: 175.0, region: "North".to_string() },
    ];
    
    // Group sales by region
    let sales_by_region = sales
        .iter()
        .fold(HashMap::new(), |mut acc, sale| {
            acc.entry(&sale.region)
                .or_insert(Vec::new())
                .push(sale);
            acc
        });
    
    // Calculate totals by region
    let totals_by_region: HashMap<&String, f64> = sales_by_region
        .iter()
        .map(|(region, sales)| {
            let total = sales.iter().map(|s| s.amount).sum();
            (*region, total)
        })
        .collect();
    
    println!("Sales by region: {:?}", totals_by_region);
    
    // Find highest sale
    let highest_sale = sales
        .iter()
        .max_by(|a, b| a.amount.partial_cmp(&b.amount).unwrap());
    println!("Highest sale: {:?}", highest_sale);
}
```

---

## Lazy Evaluation and Performance

### Understanding Lazy Evaluation

```rust
fn lazy_evaluation_demo() {
    let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    
    // This creates iterator adaptors but doesn't process anything yet
    let iter = numbers
        .iter()
        .inspect(|&x| println!("Processing: {}", x))  // Debug what's happening
        .filter(|&&x| {
            println!("Filtering: {}", x);
            x % 2 == 0
        })
        .map(|&x| {
            println!("Mapping: {}", x);
            x * x
        });
    
    println!("Iterator created, but nothing processed yet!");
    
    // Only now does processing happen
    let result: Vec<i32> = iter.take(2).collect();
    println!("Result: {:?}", result);
    // Notice: Only processes elements until it gets 2 results
}

// Performance benefits
fn performance_benefits() {
    let large_data = (0..1_000_000).collect::<Vec<i32>>();
    
    // Early termination with lazy evaluation
    let start = std::time::Instant::now();
    let first_large_even = large_data
        .iter()
        .filter(|&&x| x % 2 == 0)
        .filter(|&&x| x > 100_000)
        .next();  // Stops at first match!
    let duration = start.elapsed();
    
    println!("First large even: {:?} in {:?}", first_large_even, duration);
    
    // Compare with eager evaluation (collect before next)
    let start = std::time::Instant::now();
    let all_evens: Vec<_> = large_data
        .iter()
        .filter(|&&x| x % 2 == 0)
        .collect();  // Processes ALL elements
    let first_large = all_evens
        .iter()
        .find(|&&&x| x > 100_000);
    let duration_eager = start.elapsed();
    
    println!("Eager approach: {:?} in {:?}", first_large, duration_eager);
}

// Memory efficiency
fn memory_efficiency() {
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    
    // Process large file without loading everything into memory
    fn process_large_file(filename: &str) -> std::io::Result<usize> {
        let file = File::open(filename)?;
        let reader = BufReader::new(file);
        
        let long_lines_count = reader
            .lines()
            .map(|line| line.unwrap_or_default())
            .filter(|line| line.len() > 100)
            .count();
        
        Ok(long_lines_count)
    }
    
    // This is memory-efficient: processes one line at a time
    match process_large_file("large_file.txt") {
        Ok(count) => println!("Long lines: {}", count),
        Err(e) => println!("Error: {}", e),
    }
}
```

---

## Error Handling with Iterators

### Handling Results in Iterator Chains

```rust
fn error_handling_patterns() {
    let inputs = vec!["1", "2", "invalid", "4", "5"];
    
    // Pattern 1: Collect results, short-circuit on error
    let results: Result<Vec<i32>, _> = inputs
        .iter()
        .map(|s| s.parse::<i32>())
        .collect();
    
    match results {
        Ok(numbers) => println!("All parsed: {:?}", numbers),
        Err(e) => println!("Parse error: {}", e),
    }
    
    // Pattern 2: Filter out errors, keep successful results
    let successful_parses: Vec<i32> = inputs
        .iter()
        .filter_map(|s| s.parse().ok())
        .collect();
    println!("Successful parses: {:?}", successful_parses);
    
    // Pattern 3: Partition results
    let (successes, errors): (Vec<_>, Vec<_>) = inputs
        .iter()
        .map(|s| s.parse::<i32>())
        .partition(Result::is_ok);
    
    let successes: Vec<i32> = successes.into_iter().map(Result::unwrap).collect();
    let errors: Vec<_> = errors.into_iter().map(Result::unwrap_err).collect();
    
    println!("Successes: {:?}", successes);
    println!("Errors: {:?}", errors);
}

// Custom error handling
#[derive(Debug)]
struct ProcessingError {
    input: String,
    reason: String,
}

fn process_with_custom_errors(inputs: &[&str]) -> Result<Vec<i32>, Vec<ProcessingError>> {
    let mut successes = Vec::new();
    let mut errors = Vec::new();
    
    for &input in inputs {
        match input.parse::<i32>() {
            Ok(num) if num >= 0 => successes.push(num),
            Ok(_) => errors.push(ProcessingError {
                input: input.to_string(),
                reason: "Negative numbers not allowed".to_string(),
            }),
            Err(_) => errors.push(ProcessingError {
                input: input.to_string(),
                reason: "Invalid number format".to_string(),
            }),
        }
    }
    
    if errors.is_empty() {
        Ok(successes)
    } else {
        Err(errors)
    }
}

fn main() {
    let inputs = vec!["1", "2", "-3", "invalid", "5"];
    
    match process_with_custom_errors(&inputs) {
        Ok(numbers) => println!("Processed: {:?}", numbers),
        Err(errors) => {
            println!("Errors occurred:");
            for error in errors {
                println!("  {}: {}", error.input, error.reason);
            }
        }
    }
}
```

---

## Common Pitfalls and Best Practices

### Pitfall 1: Forgetting to Consume Iterators

```rust
fn pitfall_unused_iterators() {
    let numbers = vec![1, 2, 3, 4, 5];
    
    // BAD: This does nothing! Iterator adaptors are lazy
    numbers
        .iter()
        .map(|x| x * 2)
        .filter(|&&x| x > 5); // Warning: unused iterator that must be used
    
    // GOOD: Consume the iterator
    let result: Vec<i32> = numbers
        .iter()
        .map(|x| x * 2)
        .filter(|&&x| x > 5)
        .cloned()
        .collect();
    
    println!("Result: {:?}", result);
}
```

### Pitfall 2: Inefficient Cloning

```rust
fn avoid_unnecessary_cloning() {
    let strings = vec!["hello".to_string(), "world".to_string()];
    
    // BAD: Clones every string
    let lengths: Vec<usize> = strings
        .iter()
        .cloned()  // Expensive!
        .map(|s| s.len())
        .collect();
    
    // GOOD: Work with references
    let lengths: Vec<usize> = strings
        .iter()
        .map(|s| s.len())  // s is &String
        .collect();
    
    println!("Lengths: {:?}", lengths);
}
```

### Best Practices

```rust
// 1. Use iterator methods over manual loops when appropriate
fn best_practices() {
    let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    
    // Good: Functional style for complex transformations
    let processed: Vec<String> = numbers
        .iter()
        .filter(|&&x| x % 2 == 0)
        .map(|&x| x * x)
        .map(|x| format!("Square: {}", x))
        .collect();
    
    // Good: Use specific methods when available
    let sum: i32 = numbers.iter().sum();
    let max = numbers.iter().max();
    
    // Good: Early termination
    let first_large = numbers
        .iter()
        .find(|&&x| x > 5);
    
    // Good: Use for_each for side effects without collecting
    numbers
        .iter()
        .filter(|&&x| x % 3 == 0)
        .for_each(|x| println!("Divisible by 3: {}", x));
}

// 2. Choose the right level of functional vs imperative
fn choose_appropriate_style() {
    let data = vec![1, 2, 3, 4, 5];
    
    // Simple case: iterator is cleaner
    let doubled: Vec<i32> = data.iter().map(|x| x * 2).collect();
    
    // Complex case: imperative might be clearer
    fn complex_processing(data: &[i32]) -> Vec<String> {
        let mut results = Vec::new();
        
        for &item in data {
            if item % 2 == 0 {
                let processed = item * item;
                if processed > 10 {
                    results.push(format!("Large square: {}", processed));
                } else {
                    results.push(format!("Small square: {}", processed));
                }
            }
        }
        
        results
    }
    
    // vs functional (might be less readable for complex logic)
    let functional_result: Vec<String> = data
        .iter()
        .filter(|&&x| x % 2 == 0)
        .map(|&x| x * x)
        .map(|x| {
            if x > 10 {
                format!("Large square: {}", x)
            } else {
                format!("Small square: {}", x)
            }
        })
        .collect();
}
```

---

## Exercises

### Exercise 1: Data Processing Pipeline
Create a data processing pipeline that handles a list of employee records:

```rust
#[derive(Debug, Clone)]
struct Employee {
    name: String,
    department: String,
    salary: f64,
    years_of_service: u32,
}

// TODO: Implement these functions using iterators
fn high_earners(employees: &[Employee], threshold: f64) -> Vec<Employee> {
    // Return employees earning more than threshold, sorted by salary (highest first)
    todo!()
}

fn department_stats(employees: &[Employee]) -> std::collections::HashMap<String, (usize, f64)> {
    // Return (count, average_salary) for each department
    todo!()
}

fn senior_employees_by_dept(employees: &[Employee], min_years: u32) -> std::collections::HashMap<String, Vec<String>> {
    // Return employee names grouped by department for employees with >= min_years service
    todo!()
}
```

### Exercise 2: Text Processing
Process a text file and extract various statistics:

```rust
// TODO: Implement these text processing functions
fn word_frequency(text: &str) -> std::collections::HashMap<String, usize> {
    // Return word frequency map (case-insensitive, ignore punctuation)
    todo!()
}

fn longest_words(text: &str, n: usize) -> Vec<String> {
    // Return n longest unique words
    todo!()
}

fn sentences_with_word(text: &str, target_word: &str) -> Vec<String> {
    // Return sentences containing target_word (case-insensitive)
    // Split on ., !, or ?
    todo!()
}
```

### Exercise 3: Number Sequence Processing
Work with mathematical sequences:

```rust
// TODO: Implement a custom iterator for prime numbers
struct PrimeIterator {
    current: u64,
}

impl PrimeIterator {
    fn new() -> Self {
        // Start from 2 (first prime)
        todo!()
    }
    
    fn is_prime(n: u64) -> bool {
        // Helper function to check if number is prime
        todo!()
    }
}

impl Iterator for PrimeIterator {
    type Item = u64;
    
    fn next(&mut self) -> Option<Self::Item> {
        // Return next prime number
        todo!()
    }
}

// TODO: Use the iterator to solve these problems
fn sum_of_primes_below(limit: u64) -> u64 {
    // Sum all prime numbers below limit
    todo!()
}

fn nth_prime(n: usize) -> Option<u64> {
    // Return the nth prime number (1-indexed)
    todo!()
}

fn prime_gaps(limit: u64) -> Vec<u64> {
    // Return gaps between consecutive primes below limit
    // e.g., for primes 2,3,5,7,11: gaps are [1,2,2,4]
    todo!()
}
```

---

## Key Takeaways

1. **Lazy evaluation** makes iterators memory-efficient and fast
2. **Zero-cost abstractions** mean functional style can be as fast as imperative
3. **Closure capture** has three modes: Fn, FnMut, and FnOnce
4. **Iterator adaptors** are composable and chainable
5. **Error handling** in iterator chains requires careful consideration
6. **Choose the right tool** - iterators for transformations, loops for complex control flow
7. **Early termination** with find() and take() can provide significant performance benefits
8. **Memory efficiency** comes from processing one item at a time, not collecting unnecessarily

**Next Up:** In Chapter 12, we'll explore testing - how to write reliable tests for your Rust code with unit tests, integration tests, and documentation tests.
