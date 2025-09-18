# Chapter 25: Unsafe Rust & FFI

## Learning Objectives
- Understand when and why to use unsafe Rust
- Learn to interface with C/C++ code
- Master bindgen for automatic bindings
- Establish safety contracts and invariants
- Wrap unsafe code in safe abstractions

## When Unsafe is Necessary

Unsafe Rust allows you to:
1. Dereference raw pointers
2. Call unsafe functions
3. Access or modify mutable static variables
4. Implement unsafe traits
5. Access fields of unions

### Common Use Cases

```rust
// 1. Interfacing with C libraries
extern "C" {
    fn strlen(s: *const c_char) -> size_t;
}

// 2. Performance-critical code
unsafe fn fast_copy<T>(src: *const T, dst: *mut T, count: usize) {
    std::ptr::copy_nonoverlapping(src, dst, count);
}

// 3. Implementing fundamental abstractions
struct MyVec<T> {
    ptr: *mut T,
    len: usize,
    capacity: usize,
}

// 4. Hardware interaction
unsafe fn read_sensor() -> u32 {
    std::ptr::read_volatile(0x4000_0000 as *const u32)
}
```

## Raw Pointers

### Creating and Using Raw Pointers

```rust
fn raw_pointer_example() {
    let mut num = 5;
    
    // Create raw pointers
    let r1 = &num as *const i32;
    let r2 = &mut num as *mut i32;
    
    // Can create raw pointers in safe code
    // But dereferencing requires unsafe
    unsafe {
        println!("r1: {}", *r1);
        *r2 = 10;
        println!("r2: {}", *r2);
    }
    
    // Raw pointers can be null
    let null_ptr: *const i32 = std::ptr::null();
    
    // Check before dereferencing
    unsafe {
        if !null_ptr.is_null() {
            println!("Value: {}", *null_ptr);
        }
    }
}
```

### Pointer Arithmetic

```rust
unsafe fn pointer_arithmetic() {
    let arr = [1, 2, 3, 4, 5];
    let ptr = arr.as_ptr();
    
    // Pointer arithmetic
    let second = ptr.add(1);
    let last = ptr.add(arr.len() - 1);
    
    println!("Second: {}", *second); // 2
    println!("Last: {}", *last);     // 5
    
    // Iterate using raw pointers
    let mut current = ptr;
    let end = ptr.add(arr.len());
    
    while current < end {
        println!("Value: {}", *current);
        current = current.add(1);
    }
}
```

## FFI with C

### Basic C Function Binding

```rust
use std::os::raw::{c_char, c_int};
use std::ffi::{CString, CStr};

// Declare external C functions
extern "C" {
    fn printf(format: *const c_char, ...) -> c_int;
    fn sqrt(x: f64) -> f64;
    fn abs(x: c_int) -> c_int;
}

fn call_c_functions() {
    unsafe {
        // Call C math functions
        let result = sqrt(16.0);
        println!("sqrt(16) = {}", result);
        
        let absolute = abs(-42);
        println!("abs(-42) = {}", absolute);
        
        // Call printf (variadic function)
        let format = CString::new("Hello from Rust: %d\n").unwrap();
        printf(format.as_ptr(), 42);
    }
}
```

### Calling Rust from C

```rust
// Make Rust functions callable from C
#[no_mangle]
pub extern "C" fn rust_function(x: i32) -> i32 {
    x * 2
}

#[no_mangle]
pub extern "C" fn rust_string_length(s: *const c_char) -> usize {
    unsafe {
        if s.is_null() {
            return 0;
        }
        CStr::from_ptr(s).to_bytes().len()
    }
}

// Prevent name mangling for structs
#[repr(C)]
pub struct Point {
    x: f64,
    y: f64,
}

#[no_mangle]
pub extern "C" fn create_point(x: f64, y: f64) -> Point {
    Point { x, y }
}

#[no_mangle]
pub extern "C" fn distance(p1: &Point, p2: &Point) -> f64 {
    ((p2.x - p1.x).powi(2) + (p2.y - p1.y).powi(2)).sqrt()
}
```

## FFI with C++

### C++ Interop Challenges

```rust
// C++ has name mangling, classes, templates
// Usually need extern "C" wrapper in C++

// wrapper.hpp
#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    double x;
    double y;
} Point;

Point* create_point_cpp(double x, double y);
void delete_point_cpp(Point* p);
double calculate_distance_cpp(const Point* p1, const Point* p2);

#ifdef __cplusplus
}
#endif

// Rust bindings
extern "C" {
    fn create_point_cpp(x: f64, y: f64) -> *mut Point;
    fn delete_point_cpp(p: *mut Point);
    fn calculate_distance_cpp(p1: *const Point, p2: *const Point) -> f64;
}

// Safe wrapper
pub struct CppPoint {
    ptr: *mut Point,
}

impl CppPoint {
    pub fn new(x: f64, y: f64) -> Self {
        unsafe {
            CppPoint {
                ptr: create_point_cpp(x, y),
            }
        }
    }
    
    pub fn distance(&self, other: &CppPoint) -> f64 {
        unsafe {
            calculate_distance_cpp(self.ptr, other.ptr)
        }
    }
}

impl Drop for CppPoint {
    fn drop(&mut self) {
        unsafe {
            delete_point_cpp(self.ptr);
        }
    }
}
```

## Using Bindgen

### Setup and Configuration

```toml
# Cargo.toml
[build-dependencies]
bindgen = "0.69"

[dependencies]
libc = "0.2"
```

### Build Script

```rust
// build.rs
use bindgen;
use std::env;
use std::path::PathBuf;

fn main() {
    // Tell cargo to link the system library
    println!("cargo:rustc-link-lib=mylib");
    println!("cargo:rerun-if-changed=wrapper.h");
    
    // Generate bindings
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");
    
    // Write bindings to file
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
```

### Using Generated Bindings

```rust
// src/lib.rs
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// Safe wrapper around generated unsafe bindings
pub struct SafeWrapper {
    handle: *mut GeneratedStruct,
}

impl SafeWrapper {
    pub fn new() -> Option<Self> {
        unsafe {
            let handle = generated_create();
            if handle.is_null() {
                None
            } else {
                Some(SafeWrapper { handle })
            }
        }
    }
    
    pub fn process(&mut self, data: &[u8]) -> Result<Vec<u8>, String> {
        unsafe {
            let result = generated_process(
                self.handle,
                data.as_ptr(),
                data.len()
            );
            
            if result.is_null() {
                Err("Processing failed".to_string())
            } else {
                // Convert result to Vec<u8>
                let len = generated_result_length(result);
                let slice = std::slice::from_raw_parts(
                    result as *const u8,
                    len
                );
                let vec = slice.to_vec();
                generated_free_result(result);
                Ok(vec)
            }
        }
    }
}

impl Drop for SafeWrapper {
    fn drop(&mut self) {
        unsafe {
            generated_destroy(self.handle);
        }
    }
}
```

## Safety Contracts

### Establishing Invariants

```rust
/// SAFETY: This struct maintains the following invariants:
/// 1. `ptr` is always valid and points to `capacity` elements
/// 2. `len <= capacity`
/// 3. Elements 0..len are initialized
/// 4. The allocator used is the global allocator
pub struct SafeVec<T> {
    ptr: *mut T,
    len: usize,
    capacity: usize,
}

impl<T> SafeVec<T> {
    /// Creates a new empty vector
    /// 
    /// # Safety
    /// This function is safe because it maintains all invariants
    pub fn new() -> Self {
        SafeVec {
            ptr: std::ptr::null_mut(),
            len: 0,
            capacity: 0,
        }
    }
    
    /// Pushes an element onto the vector
    /// 
    /// # Safety
    /// Safe because:
    /// - Allocation is handled properly
    /// - Capacity is checked and grown if needed
    /// - Length is updated after successful write
    pub fn push(&mut self, value: T) {
        if self.len == self.capacity {
            self.grow();
        }
        
        unsafe {
            // SAFETY: We just ensured capacity > len
            std::ptr::write(self.ptr.add(self.len), value);
            self.len += 1;
        }
    }
    
    fn grow(&mut self) {
        // Implementation maintaining invariants
    }
}
```

### Unsafe Trait Implementation

```rust
// Marker traits that affect compiler behavior
unsafe impl<T: Send> Send for SafeVec<T> {}
unsafe impl<T: Sync> Sync for SafeVec<T> {}

// SAFETY: We only implement Send if T is Send
// because our vector owns T values
```

## Common Undefined Behaviors to Avoid

### 1. Data Races
```rust
// WRONG: Data race
static mut COUNTER: i32 = 0;

fn bad_increment() {
    unsafe {
        COUNTER += 1; // Data race if called from multiple threads
    }
}

// CORRECT: Use synchronization
use std::sync::atomic::{AtomicI32, Ordering};
static COUNTER: AtomicI32 = AtomicI32::new(0);

fn good_increment() {
    COUNTER.fetch_add(1, Ordering::SeqCst);
}
```

### 2. Invalid Memory Access
```rust
// WRONG: Use after free
fn bad_pointer() -> *const i32 {
    let x = 42;
    &x as *const i32 // x is dropped, pointer becomes invalid
}

// CORRECT: Ensure lifetime
fn good_pointer(x: &i32) -> *const i32 {
    x as *const i32 // Pointer valid as long as reference is
}
```

### 3. Aliasing Violations
```rust
// WRONG: Mutable aliasing
fn bad_aliasing() {
    let mut x = 5;
    let r1 = &mut x as *mut i32;
    let r2 = &mut x as *mut i32;
    unsafe {
        *r1 = 10;
        *r2 = 20; // Undefined behavior: two mutable aliases
    }
}
```

## Best Practices

### 1. Minimize Unsafe Scope
```rust
// Bad: Large unsafe block
unsafe {
    let ptr = allocate_memory();
    initialize_data(ptr);
    process_data(ptr);
    cleanup(ptr);
}

// Good: Multiple small unsafe blocks
let ptr = unsafe { allocate_memory() };
unsafe { initialize_data(ptr); }
unsafe { process_data(ptr); }
unsafe { cleanup(ptr); }
```

### 2. Document Safety Requirements
```rust
/// Copies `count` elements from `src` to `dst`
/// 
/// # Safety
/// 
/// - `src` must be valid for reads of `count * size_of::<T>()` bytes
/// - `dst` must be valid for writes of `count * size_of::<T>()` bytes
/// - The regions must not overlap
pub unsafe fn copy_memory<T>(src: *const T, dst: *mut T, count: usize) {
    std::ptr::copy_nonoverlapping(src, dst, count);
}
```

### 3. Provide Safe Abstractions
```rust
pub struct FfiString {
    ptr: *mut c_char,
}

impl FfiString {
    pub fn new(s: &str) -> Result<Self, std::ffi::NulError> {
        let c_string = CString::new(s)?;
        Ok(FfiString {
            ptr: c_string.into_raw(),
        })
    }
    
    pub fn as_ptr(&self) -> *const c_char {
        self.ptr
    }
}

impl Drop for FfiString {
    fn drop(&mut self) {
        unsafe {
            let _ = CString::from_raw(self.ptr);
        }
    }
}
```

## Exercises

### Exercise 24.1: Safe FFI Wrapper
Create a safe Rust wrapper for this C API:
```c
typedef struct Buffer {
    char* data;
    size_t size;
} Buffer;

Buffer* buffer_create(size_t size);
void buffer_destroy(Buffer* buf);
int buffer_write(Buffer* buf, const char* data, size_t len);
```

### Exercise 24.2: Custom Allocator
Implement a simple bump allocator using unsafe code:
```rust
struct BumpAllocator {
    start: *mut u8,
    current: *mut u8,
    end: *mut u8,
}

impl BumpAllocator {
    unsafe fn alloc(&mut self, size: usize) -> *mut u8 {
        // TODO: Implement
        todo!()
    }
}
```

### Exercise 24.3: Bindgen Integration
Use bindgen to create bindings for a simple C library and wrap them in a safe API.

## Key Takeaways

✅ **Unsafe is sometimes necessary** - For FFI, performance, and low-level code

✅ **Raw pointers need careful handling** - Check null, ensure validity

✅ **FFI requires extern blocks** - And often repr(C) for structs

✅ **Bindgen automates binding generation** - But still need safe wrappers

✅ **Document safety contracts** - Make invariants explicit

✅ **Minimize unsafe scope** - Wrap in safe abstractions

---

Next: [Chapter 26: Performance & Optimization](./26_performance.md)
