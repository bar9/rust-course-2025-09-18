# Chapter 26: Performance & Optimization

## Learning Objectives
- Understand zero-cost abstractions in practice
- Master benchmarking with criterion
- Apply profile-guided optimization
- Learn memory layout optimization
- Compare performance with C++ and .NET

## Zero-Cost Abstractions

Rust's philosophy: "What you don't use, you don't pay for."

### Iterator Performance

```rust
// This high-level code...
let sum: i32 = (0..1000)
    .filter(|x| x % 2 == 0)
    .map(|x| x * x)
    .sum();

// ...compiles to the same assembly as:
let mut sum = 0i32;
for i in 0..1000 {
    if i % 2 == 0 {
        sum += i * i;
    }
}

// Benchmark proof
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_iterator(c: &mut Criterion) {
    c.bench_function("iterator", |b| {
        b.iter(|| {
            (0..1000)
                .filter(|x| x % 2 == 0)
                .map(|x| x * x)
                .sum::<i32>()
        })
    });
}

fn bench_loop(c: &mut Criterion) {
    c.bench_function("manual loop", |b| {
        b.iter(|| {
            let mut sum = 0i32;
            for i in 0..1000 {
                if i % 2 == 0 {
                    sum += i * i;
                }
            }
            sum
        })
    });
}
```

### Option and Result Optimization

```rust
// Option<&T> is optimized to a single pointer
use std::mem::size_of;

assert_eq!(size_of::<Option<&i32>>(), size_of::<*const i32>());

// Result<T, ()> for non-zero types uses niche optimization
#[repr(transparent)]
struct NonZeroU32(std::num::NonZeroU32);

assert_eq!(size_of::<Option<NonZeroU32>>(), size_of::<u32>());

// Compiler optimizes match on Option/Result
fn process(opt: Option<i32>) -> i32 {
    match opt {
        Some(x) => x * 2,
        None => 0,
    }
    // Compiles to branchless code when beneficial
}
```

## Benchmarking with Criterion

### Setup

```toml
# Cargo.toml
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }

[[bench]]
name = "my_benchmark"
harness = false
```

### Writing Benchmarks

```rust
// benches/my_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        n => fibonacci(n-1) + fibonacci(n-2),
    }
}

fn fibonacci_iterative(n: u64) -> u64 {
    let mut a = 0;
    let mut b = 1;
    
    for _ in 0..n {
        let tmp = a;
        a = b;
        b = tmp + b;
    }
    b
}

fn bench_fibonacci(c: &mut Criterion) {
    let mut group = c.benchmark_group("fibonacci");
    
    for i in [20, 25, 30].iter() {
        group.bench_with_input(BenchmarkId::new("recursive", i), i, |b, i| {
            b.iter(|| fibonacci(black_box(*i)));
        });
        
        group.bench_with_input(BenchmarkId::new("iterative", i), i, |b, i| {
            b.iter(|| fibonacci_iterative(black_box(*i)));
        });
    }
    group.finish();
}

// Compare different data structures
fn bench_collections(c: &mut Criterion) {
    let mut group = c.benchmark_group("collections");
    let data: Vec<i32> = (0..1000).collect();
    
    group.bench_function("vec_search", |b| {
        b.iter(|| {
            data.iter().find(|&&x| x == 500)
        });
    });
    
    use std::collections::HashSet;
    let set: HashSet<i32> = data.iter().cloned().collect();
    
    group.bench_function("hashset_search", |b| {
        b.iter(|| {
            set.contains(&500)
        });
    });
    
    group.finish();
}

criterion_group!(benches, bench_fibonacci, bench_collections);
criterion_main!(benches);
```

### Running and Analyzing

```bash
# Run benchmarks
cargo bench

# View HTML report
open target/criterion/report/index.html

# Compare with baseline
cargo bench -- --baseline my_baseline --save-baseline new_baseline
```

## Profile-Guided Optimization (PGO)

### Setting Up PGO

```bash
# Step 1: Build with profile generation
RUSTFLAGS="-Cprofile-generate=/tmp/pgo-data" \
    cargo build --release

# Step 2: Run with representative workload
./target/release/my_program --typical-workload

# Step 3: Merge profile data
llvm-profdata merge -o /tmp/pgo-data/merged.profdata /tmp/pgo-data

# Step 4: Build with profile use
RUSTFLAGS="-Cprofile-use=/tmp/pgo-data/merged.profdata" \
    cargo build --release
```

### Cargo Configuration for PGO

```toml
# Cargo.toml
[profile.release-pgo-generate]
inherits = "release"
lto = "fat"

[profile.release-pgo-use]
inherits = "release"
lto = "fat"
```

## Memory Layout Optimization

### Struct Layout

```rust
use std::mem::{size_of, align_of};

// Bad: Poor alignment causes padding
#[derive(Debug)]
struct Inefficient {
    a: u8,   // 1 byte
    b: u64,  // 8 bytes (7 bytes padding before)
    c: u8,   // 1 byte
    d: u32,  // 4 bytes (3 bytes padding before)
}  // Total: 24 bytes

// Good: Ordered by alignment
#[derive(Debug)]
struct Efficient {
    b: u64,  // 8 bytes
    d: u32,  // 4 bytes
    a: u8,   // 1 byte
    c: u8,   // 1 byte (2 bytes padding after)
}  // Total: 16 bytes

assert_eq!(size_of::<Inefficient>(), 24);
assert_eq!(size_of::<Efficient>(), 16);

// Use repr(C) for predictable layout
#[repr(C)]
struct PredictableLayout {
    field1: u32,
    field2: u32,
}

// Pack tightly (loses alignment benefits)
#[repr(packed)]
struct Packed {
    a: u8,
    b: u64,
    c: u8,
}  // Total: 10 bytes, but unaligned access
```

### Cache-Friendly Data Structures

```rust
// Structure of Arrays (SoA) vs Array of Structures (AoS)

// AoS - Poor cache locality for single field access
struct Particle {
    x: f32,
    y: f32,
    z: f32,
    mass: f32,
    velocity_x: f32,
    velocity_y: f32,
    velocity_z: f32,
}

struct ParticleSystem {
    particles: Vec<Particle>,
}

// SoA - Better cache locality
struct ParticleSystemSoA {
    x: Vec<f32>,
    y: Vec<f32>,
    z: Vec<f32>,
    mass: Vec<f32>,
    velocity_x: Vec<f32>,
    velocity_y: Vec<f32>,
    velocity_z: Vec<f32>,
}

impl ParticleSystemSoA {
    fn update_positions(&mut self, dt: f32) {
        // All position data is contiguous
        for i in 0..self.x.len() {
            self.x[i] += self.velocity_x[i] * dt;
            self.y[i] += self.velocity_y[i] * dt;
            self.z[i] += self.velocity_z[i] * dt;
        }
    }
}
```

## Common Performance Patterns

### 1. Avoid Allocations in Hot Paths

```rust
// Bad: Allocates every iteration
fn process_bad(data: &[String]) -> Vec<String> {
    data.iter()
        .map(|s| s.to_uppercase())  // Allocates new String
        .collect()
}

// Good: Reuse buffer
fn process_good(data: &[String], output: &mut Vec<String>) {
    output.clear();
    output.reserve(data.len());
    
    for s in data {
        let mut upper = String::with_capacity(s.len());
        upper.push_str(&s.to_uppercase());
        output.push(upper);
    }
}

// Better: Use SmallVec for small allocations
use smallvec::SmallVec;

fn process_small(data: &[u8]) -> SmallVec<[u8; 32]> {
    let mut result = SmallVec::new();
    // No heap allocation if result fits in 32 bytes
    result.extend_from_slice(data);
    result
}
```

### 2. SIMD Operations

```rust
use std::simd::*;

fn dot_product_scalar(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

fn dot_product_simd(a: &[f32], b: &[f32]) -> f32 {
    let chunks = a.chunks_exact(4).zip(b.chunks_exact(4));
    
    let mut sum = f32x4::splat(0.0);
    for (a_chunk, b_chunk) in chunks {
        let a_vec = f32x4::from_slice(a_chunk);
        let b_vec = f32x4::from_slice(b_chunk);
        sum += a_vec * b_vec;
    }
    
    sum.reduce_sum()
}
```

### 3. Branch Prediction

```rust
// Help the branch predictor with likely/unlikely hints
use std::intrinsics::{likely, unlikely};

unsafe fn process_with_hints(data: &[i32]) -> i32 {
    let mut sum = 0;
    for &x in data {
        if likely(x > 0) {  // Most values are positive
            sum += x;
        } else if unlikely(x < -1000) {  // Rare case
            sum -= 1000;
        }
    }
    sum
}

// Sort data to improve branch prediction
fn process_sorted(mut data: Vec<i32>) -> i32 {
    data.sort_unstable();  // Now branches are predictable
    data.iter().filter(|&&x| x > 0).sum()
}
```

## Compile-Time Optimizations

### Release Profile Settings

```toml
# Cargo.toml
[profile.release]
opt-level = 3          # Maximum optimizations
lto = "fat"            # Link-time optimization
codegen-units = 1      # Single codegen unit for better optimization
panic = "abort"        # Smaller binary, no unwinding
strip = true           # Strip symbols

[profile.release-fast]
inherits = "release"
opt-level = 3
lto = "fat"
codegen-units = 1

[profile.release-small]
inherits = "release"
opt-level = "z"        # Optimize for size
strip = true
panic = "abort"
```

### Const Functions and Evaluation

```rust
// Compute at compile time
const fn factorial(n: u32) -> u32 {
    match n {
        0 | 1 => 1,
        _ => n * factorial(n - 1),
    }
}

const FACT_10: u32 = factorial(10);  // Computed at compile time

// Const generics for compile-time sized arrays
fn process_array<const N: usize>(arr: [i32; N]) -> i32 {
    arr.iter().sum()  // Size known at compile time
}
```

## Comparison with C++ and .NET

### Performance Characteristics

| Aspect | Rust | C++ | .NET |
|--------|------|-----|------|
| Predictability | Excellent | Excellent | Good (GC pauses) |
| Memory usage | Minimal | Minimal | Higher (GC overhead) |
| Startup time | Fast | Fast | Slower (JIT) |
| Peak performance | Native | Native | Near-native |
| Optimization | Compile-time | Compile-time | JIT + AOT |

### Benchmark Example

```rust
// Rust version
fn sum_of_squares_rust(data: &[i32]) -> i64 {
    data.iter()
        .map(|&x| x as i64)
        .map(|x| x * x)
        .sum()
}

// Equivalent C++
// int64_t sum_of_squares_cpp(const std::vector<int32_t>& data) {
//     return std::accumulate(data.begin(), data.end(), 0LL,
//         [](int64_t sum, int32_t x) { 
//             return sum + static_cast<int64_t>(x) * x; 
//         });
// }

// Equivalent C#
// long SumOfSquaresCSharp(int[] data) {
//     return data.Select(x => (long)x * x).Sum();
// }

// All three compile to similar machine code
```

## Profiling Tools

### Using perf on Linux

```bash
# Record profile
perf record --call-graph=dwarf ./target/release/my_program

# View report
perf report

# Generate flame graph
perf script | flamegraph > flame.svg
```

### Using Instruments on macOS

```bash
# Build with debug symbols
cargo build --release

# Profile with Instruments
xcrun instruments -t "Time Profiler" ./target/release/my_program
```

### Built-in Profiling

```rust
use std::time::Instant;

fn measure_performance<F, R>(f: F) -> (R, std::time::Duration)
where
    F: FnOnce() -> R,
{
    let start = Instant::now();
    let result = f();
    let duration = start.elapsed();
    (result, duration)
}

let (result, time) = measure_performance(|| {
    expensive_computation()
});
println!("Took {:?}", time);
```

## Exercises

### Exercise 25.1: Optimize Memory Layout
Reorganize this struct for optimal memory usage:
```rust
struct Data {
    flag: bool,
    id: u64,
    count: u16,
    value: f32,
    status: u8,
}
```

### Exercise 25.2: Benchmark Implementations
Write criterion benchmarks comparing:
1. HashMap vs BTreeMap for lookups
2. Vec vs VecDeque for push/pop operations
3. String concatenation vs format! macro

### Exercise 25.3: Profile and Optimize
Take this function and optimize it:
```rust
fn process(data: Vec<String>) -> Vec<String> {
    data.into_iter()
        .filter(|s| s.len() > 5)
        .map(|s| s.to_uppercase())
        .collect()
}
```

## Key Takeaways

✅ **Zero-cost abstractions are real** - High-level code compiles to optimal assembly

✅ **Benchmark everything** - Use criterion for reliable measurements

✅ **Memory layout matters** - Align structs, consider cache locality

✅ **Profile before optimizing** - Measure, don't guess

✅ **Compile-time work is free** - Use const functions and generics

✅ **Rust matches C++ performance** - Without sacrificing safety

---

Congratulations! You've completed the Rust course and are ready to build high-performance, safe systems!
