# Chapter 17: no_std Programming Introduction

## Learning Objectives
- Understand the difference between `core`, `alloc`, and `std` libraries
- Write `no_std` libraries and applications for embedded systems
- Use heapless data structures for memory-constrained environments
- Master const functions for compile-time computation
- Apply embedded programming patterns and best practices
- Handle resource constraints and real-time requirements

## Core vs Std: Understanding Rust's Standard Library

Rust's standard library is actually composed of several layers:

```rust
#![no_std]
// Using only core library - no heap allocation, no OS dependencies

// Core is always available and provides:
use core::{
    mem, ptr, slice, str,
    option::Option,
    result::Result,
    fmt::{Debug, Display},
    iter::Iterator,
    clone::Clone,
    marker::{Copy, Send, Sync},
};

// Example of core-only function
fn find_max_core_only(slice: &[i32]) -> Option<i32> {
    if slice.is_empty() {
        return None;
    }
    
    let mut max = slice[0];
    for &item in slice.iter().skip(1) {
        if item > max {
            max = item;
        }
    }
    Some(max)
}

// Working with core types
fn core_types_example() {
    // Basic types work the same
    let x: i32 = 42;
    let y: Option<i32> = Some(x);
    let z: Result<i32, &str> = Ok(x);
    
    // Iterators work (but no collect() without alloc)
    let data = [1, 2, 3, 4, 5];
    let sum: i32 = data.iter().sum();
    
    // String slices work, but no String type
    let text: &str = "Hello, embedded world!";
    let first_char = text.chars().next();
    
    // Arrays work, but no Vec without alloc
    let mut buffer = [0u8; 64];
    buffer[0] = 42;
}
```

**C/C++ Comparison:**
- **C**: Manual memory management, platform-specific libraries
- **C++**: STL available but often avoided in embedded contexts
- **Rust**: Explicit library layers with zero-cost abstractions maintained

## Using Alloc Without Std

The `alloc` crate provides heap allocation without OS dependencies:

```rust
#![no_std]
extern crate alloc;

use alloc::{
    vec::Vec,
    string::String,
    boxed::Box,
    collections::{BTreeMap, BTreeSet},
    format,
    vec,
};

// Now we can use heap-allocated types
fn alloc_examples() {
    // Vectors work
    let mut numbers = Vec::new();
    numbers.push(1);
    numbers.push(2);
    numbers.push(3);
    
    // Strings work
    let greeting = String::from("Hello");
    let formatted = format!("{}!", greeting);
    
    // Box for single heap allocation
    let boxed_value = Box::new(42i32);
    
    // Collections that don't require hashing
    let mut map = BTreeMap::new();
    map.insert("key", "value");
    
    // But HashMap requires std (uses RandomState)
    // This won't compile in no_std:
    // use std::collections::HashMap; // Error!
}

// Custom allocator example (requires global allocator)
use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

// Initialize heap in embedded context
fn init_heap() {
    use linked_list_allocator::LockedHeap;
    const HEAP_SIZE: usize = 1024;
    static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];
    
    unsafe {
        ALLOCATOR.lock().init(HEAP.as_mut_ptr(), HEAP_SIZE);
    }
}
```

## Heapless Data Structures

The `heapless` crate provides fixed-capacity collections:

```rust
#![no_std]

use heapless::{
    Vec, String, FnvIndexMap,
    pool::{Pool, Node},
    spsc::{Producer, Consumer, Queue},
    mpmc::Q8,
};

// Fixed-capacity vector
fn heapless_vec_example() {
    // Vec with maximum 8 elements
    let mut vec: Vec<i32, 8> = Vec::new();
    
    vec.push(1).ok(); // Returns Result - can fail if full
    vec.push(2).ok();
    vec.push(3).ok();
    
    // Check capacity
    assert_eq!(vec.len(), 3);
    assert_eq!(vec.capacity(), 8);
    
    // Iterate like normal Vec
    for &item in &vec {
        // Process item
    }
    
    // Convert to slice
    let slice: &[i32] = &vec;
}

// Fixed-capacity string
fn heapless_string_example() {
    // String with maximum 32 bytes
    let mut text: String<32> = String::new();
    
    text.push_str("Hello").ok();
    text.push(' ').ok();
    text.push_str("embedded").ok();
    
    // Format into heapless string (requires ufmt crate for no_std)
    // let formatted: String<64> = ufmt::uformat!("Value: {}", 42);
}

// Hash map alternative
fn heapless_map_example() {
    // Map with maximum 16 entries
    let mut map: FnvIndexMap<&str, i32, 16> = FnvIndexMap::new();
    
    map.insert("temperature", 23).ok();
    map.insert("humidity", 45).ok();
    
    if let Some(&temp) = map.get("temperature") {
        // Use temperature value
    }
    
    // Iterate over entries
    for (key, value) in &map {
        // Process key-value pairs
    }
}

// Memory pool for dynamic allocation without heap
fn memory_pool_example() {
    // Create pool with 16 nodes
    static mut MEMORY: [Node<[u8; 32]>; 16] = [Node::new(); 16];
    static POOL: Pool<[u8; 32]> = Pool::new();
    
    // Initialize pool
    unsafe {
        POOL.grow_exact(&mut MEMORY);
    }
    
    // Allocate from pool
    if let Some(mut buffer) = POOL.alloc() {
        buffer[0] = 42;
        // Use buffer...
        // Automatically returned to pool when dropped
    }
}

// Lock-free queue for interrupt communication
fn lock_free_queue_example() {
    static mut QUEUE: Queue<u32, 8> = Queue::new();
    
    // In main thread - split queue
    let (mut producer, mut consumer) = unsafe { QUEUE.split() };
    
    // Producer side (could be in interrupt)
    producer.enqueue(42).ok();
    producer.enqueue(13).ok();
    
    // Consumer side (main loop)
    while let Some(value) = consumer.dequeue() {
        // Process value
    }
}
```

## Const Functions and Compile-time Computation

Const functions enable computation at compile time:

```rust
#![no_std]

// Simple const function
const fn add(a: i32, b: i32) -> i32 {
    a + b
}

// Const function with control flow
const fn factorial(n: u32) -> u32 {
    if n == 0 {
        1
    } else {
        n * factorial(n - 1)
    }
}

// Const function with loops (requires const fn in loops feature)
const fn sum_range(start: i32, end: i32) -> i32 {
    let mut sum = 0;
    let mut i = start;
    while i <= end {
        sum += i;
        i += 1;
    }
    sum
}

// Const generic functions
const fn create_array<const N: usize>() -> [i32; N] {
    [0; N]
}

// Using const functions
const FACTORIAL_5: u32 = factorial(5); // Computed at compile time
const SUM_1_TO_10: i32 = sum_range(1, 10); // Also compile time
const BUFFER: [i32; 100] = create_array::<100>(); // Zero-cost

// Const fn for embedded configuration
const fn calculate_baud_divisor(clock_freq: u32, baud_rate: u32) -> u32 {
    clock_freq / (16 * baud_rate)
}

const SYSTEM_CLOCK: u32 = 16_000_000; // 16 MHz
const UART_BAUD: u32 = 115_200;
const BAUD_DIVISOR: u32 = calculate_baud_divisor(SYSTEM_CLOCK, UART_BAUD);

// Const assertions (compile-time checks)
const fn check_buffer_size(size: usize) -> usize {
    assert!(size > 0 && size <= 1024);
    size
}

const BUFFER_SIZE: usize = check_buffer_size(256);

// Advanced const fn with const generics
const fn is_power_of_two(n: usize) -> bool {
    n > 0 && (n & (n - 1)) == 0
}

struct RingBuffer<T, const N: usize> {
    buffer: [Option<T>; N],
    head: usize,
    tail: usize,
}

impl<T, const N: usize> RingBuffer<T, N> {
    const fn new() -> Self {
        // This requires const Option::None
        const fn none<T>() -> Option<T> { None }
        
        // Compile-time assertion
        assert!(is_power_of_two(N), "Buffer size must be power of two");
        
        RingBuffer {
            buffer: [none(); N],
            head: 0,
            tail: 0,
        }
    }
    
    const fn mask(&self) -> usize {
        N - 1
    }
}
```

## Embedded Programming Patterns

Common patterns for embedded Rust development:

```rust
#![no_std]
#![no_main]

use core::panic::PanicInfo;
use cortex_m_rt::entry;

// Panic handler required for no_std
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // In embedded systems, might reset or enter infinite loop
    loop {}
}

// Main function for embedded
#[entry]
fn main() -> ! {
    // Initialization
    init_system();
    
    // Main loop
    loop {
        // Application logic
        handle_tasks();
        
        // Power management
        cortex_m::asm::wfi(); // Wait for interrupt
    }
}

fn init_system() {
    // Hardware initialization
    init_clocks();
    init_gpio();
    init_peripherals();
}

fn init_clocks() {
    // Clock configuration
}

fn init_gpio() {
    // GPIO pin configuration
}

fn init_peripherals() {
    // UART, SPI, I2C, etc.
}

// State machine pattern for embedded
#[derive(Clone, Copy, Debug)]
enum SystemState {
    Idle,
    Measuring,
    Transmitting,
    Error,
}

struct SystemController {
    state: SystemState,
    measurement_count: u32,
    error_count: u32,
}

impl SystemController {
    const fn new() -> Self {
        SystemController {
            state: SystemState::Idle,
            measurement_count: 0,
            error_count: 0,
        }
    }
    
    fn update(&mut self, event: SystemEvent) {
        self.state = match (self.state, event) {
            (SystemState::Idle, SystemEvent::StartMeasurement) => {
                self.start_measurement();
                SystemState::Measuring
            }
            (SystemState::Measuring, SystemEvent::MeasurementComplete) => {
                self.measurement_count += 1;
                SystemState::Transmitting
            }
            (SystemState::Transmitting, SystemEvent::TransmissionComplete) => {
                SystemState::Idle
            }
            (_, SystemEvent::Error) => {
                self.error_count += 1;
                SystemState::Error
            }
            (SystemState::Error, SystemEvent::Reset) => {
                SystemState::Idle
            }
            // Invalid transitions stay in current state
            _ => self.state,
        };
    }
    
    fn start_measurement(&self) {
        // Start ADC conversion, etc.
    }
}

#[derive(Clone, Copy, Debug)]
enum SystemEvent {
    StartMeasurement,
    MeasurementComplete,
    TransmissionComplete,
    Error,
    Reset,
}

// Interrupt-safe communication
use cortex_m::interrupt::{self, Mutex};
use core::cell::RefCell;

type SharedData = Mutex<RefCell<Option<u32>>>;
static SENSOR_DATA: SharedData = Mutex::new(RefCell::new(None));

fn read_sensor_data() -> Option<u32> {
    interrupt::free(|cs| {
        SENSOR_DATA.borrow(cs).borrow().clone()
    })
}

fn write_sensor_data(value: u32) {
    interrupt::free(|cs| {
        *SENSOR_DATA.borrow(cs).borrow_mut() = Some(value);
    });
}

// Task scheduler pattern
struct Task {
    period_ms: u32,
    last_run: u32,
    function: fn(),
}

impl Task {
    const fn new(period_ms: u32, function: fn()) -> Self {
        Task {
            period_ms,
            last_run: 0,
            function,
        }
    }
    
    fn should_run(&self, current_time: u32) -> bool {
        current_time.wrapping_sub(self.last_run) >= self.period_ms
    }
    
    fn run(&mut self, current_time: u32) {
        (self.function)();
        self.last_run = current_time;
    }
}

static mut TASKS: [Task; 3] = [
    Task::new(100, sensor_task),    // 100ms period
    Task::new(1000, heartbeat_task), // 1s period  
    Task::new(5000, status_task),   // 5s period
];

fn handle_tasks() {
    let current_time = get_system_time_ms();
    
    unsafe {
        for task in &mut TASKS {
            if task.should_run(current_time) {
                task.run(current_time);
            }
        }
    }
}

fn sensor_task() {
    // Read sensors
}

fn heartbeat_task() {
    // Toggle LED
}

fn status_task() {
    // Send status update
}

fn get_system_time_ms() -> u32 {
    // Return system time in milliseconds
    0 // Placeholder
}
```

## Error Handling in no_std

Robust error handling without std:

```rust
#![no_std]

// Custom error types
#[derive(Debug, Clone, Copy)]
enum SensorError {
    NotInitialized,
    CommunicationFailed,
    InvalidData,
    Timeout,
}

#[derive(Debug, Clone, Copy)]
enum SystemError {
    Sensor(SensorError),
    Memory,
    Hardware,
}

impl From<SensorError> for SystemError {
    fn from(err: SensorError) -> Self {
        SystemError::Sensor(err)
    }
}

// Result type alias
type SystemResult<T> = Result<T, SystemError>;

// Error handling functions
fn read_temperature_sensor() -> Result<i16, SensorError> {
    // Simulate sensor reading
    if !is_sensor_initialized() {
        return Err(SensorError::NotInitialized);
    }
    
    if !is_communication_ok() {
        return Err(SensorError::CommunicationFailed);
    }
    
    let raw_value = read_adc();
    if raw_value > 4095 {
        return Err(SensorError::InvalidData);
    }
    
    Ok(raw_value as i16)
}

fn process_sensor_data() -> SystemResult<()> {
    let temperature = read_temperature_sensor()?; // Error propagation
    
    if temperature > 1000 {
        return Err(SystemError::Hardware);
    }
    
    // Process temperature
    store_temperature(temperature)?;
    
    Ok(())
}

fn store_temperature(temp: i16) -> SystemResult<()> {
    // Simulate memory operation
    if is_memory_full() {
        Err(SystemError::Memory)
    } else {
        // Store temperature
        Ok(())
    }
}

// Utility functions (would be implemented for real hardware)
fn is_sensor_initialized() -> bool { true }
fn is_communication_ok() -> bool { true }
fn read_adc() -> u16 { 1234 }
fn is_memory_full() -> bool { false }

// Error recovery patterns
fn safe_sensor_operation() -> SystemResult<i16> {
    const MAX_RETRIES: usize = 3;
    let mut retries = 0;
    
    loop {
        match read_temperature_sensor() {
            Ok(value) => return Ok(value),
            Err(SensorError::CommunicationFailed) if retries < MAX_RETRIES => {
                retries += 1;
                // Wait and retry
                delay_ms(10);
                continue;
            }
            Err(e) => return Err(e.into()),
        }
    }
}

fn delay_ms(_ms: u32) {
    // Platform-specific delay implementation
}
```

## Memory Management in no_std

Strategies for managing memory without heap:

```rust
#![no_std]

use heapless::{Vec, String};
use heapless::pool::{Pool, Node};

// Stack-allocated buffers
const BUFFER_SIZE: usize = 1024;
static mut WORK_BUFFER: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];

fn use_stack_buffer() {
    let mut local_buffer = [0u8; 256];
    
    // Use buffer for temporary work
    fill_buffer(&mut local_buffer, 0xFF);
    
    // Buffer automatically cleaned up when function exits
}

fn fill_buffer(buffer: &mut [u8], value: u8) {
    for byte in buffer {
        *byte = value;
    }
}

// Memory pool for dynamic allocation
static mut POOL_MEMORY: [Node<[u8; 64]>; 32] = [Node::new(); 32];
static BUFFER_POOL: Pool<[u8; 64]> = Pool::new();

fn init_memory_pool() {
    unsafe {
        BUFFER_POOL.grow_exact(&mut POOL_MEMORY);
    }
}

fn use_pooled_memory() -> Option<()> {
    let buffer = BUFFER_POOL.alloc()?; // Get buffer from pool
    
    // Use buffer...
    // Buffer automatically returned to pool when dropped
    
    Some(())
}

// Ring buffer implementation
struct RingBuffer<T, const N: usize> {
    buffer: [core::mem::MaybeUninit<T>; N],
    head: usize,
    tail: usize,
    full: bool,
}

impl<T, const N: usize> RingBuffer<T, N> {
    const fn new() -> Self {
        RingBuffer {
            buffer: unsafe { core::mem::MaybeUninit::uninit().assume_init() },
            head: 0,
            tail: 0,
            full: false,
        }
    }
    
    fn push(&mut self, item: T) -> Result<(), T> {
        if self.is_full() {
            return Err(item);
        }
        
        unsafe {
            self.buffer[self.head].as_mut_ptr().write(item);
        }
        
        self.head = (self.head + 1) % N;
        self.full = self.head == self.tail;
        Ok(())
    }
    
    fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }
        
        let item = unsafe { self.buffer[self.tail].as_ptr().read() };
        
        self.tail = (self.tail + 1) % N;
        self.full = false;
        Some(item)
    }
    
    const fn is_full(&self) -> bool {
        self.full
    }
    
    const fn is_empty(&self) -> bool {
        !self.full && self.head == self.tail
    }
    
    fn len(&self) -> usize {
        if self.full {
            N
        } else if self.head >= self.tail {
            self.head - self.tail
        } else {
            N - self.tail + self.head
        }
    }
}

// Fixed-capacity string formatting
fn format_sensor_data(temp: i16, humidity: u8) -> heapless::String<64> {
    let mut output = heapless::String::new();
    
    // Simple formatting without std::format!
    output.push_str("Temp: ").ok();
    push_number(&mut output, temp as i32);
    output.push_str("C, Humidity: ").ok();
    push_number(&mut output, humidity as i32);
    output.push('%').ok();
    
    output
}

fn push_number(s: &mut heapless::String<64>, mut num: i32) {
    if num == 0 {
        s.push('0').ok();
        return;
    }
    
    if num < 0 {
        s.push('-').ok();
        num = -num;
    }
    
    // Simple number to string conversion
    let mut digits = heapless::Vec::<u8, 16>::new();
    while num > 0 {
        digits.push((num % 10) as u8).ok();
        num /= 10;
    }
    
    for &digit in digits.iter().rev() {
        s.push((b'0' + digit) as char).ok();
    }
}
```

## Common Pitfalls and Solutions

### 1. Stack Overflow

```rust
#![no_std]

// BAD: Large arrays on stack
fn bad_large_stack_usage() {
    let large_array = [0u8; 10000]; // Might overflow stack
    process_data(&large_array);
}

// GOOD: Use static storage or heap
static mut LARGE_BUFFER: [u8; 10000] = [0; 10000];

fn good_large_data_usage() {
    unsafe {
        process_data(&LARGE_BUFFER);
    }
}

// Or use memory pool
fn good_pooled_usage() {
    if let Some(buffer) = BUFFER_POOL.alloc() {
        process_small_data(&*buffer);
    }
}

fn process_data(_data: &[u8]) {}
fn process_small_data(_data: &[u8; 64]) {}
```

### 2. Integer Overflow

```rust
// BAD: Unchecked arithmetic
fn bad_arithmetic(a: u32, b: u32) -> u32 {
    a + b // Can overflow silently in release mode
}

// GOOD: Checked arithmetic
fn good_arithmetic(a: u32, b: u32) -> Option<u32> {
    a.checked_add(b)
}

// Or wrapping arithmetic when overflow is expected
fn wrapping_counter(current: u32) -> u32 {
    current.wrapping_add(1)
}
```

## Exercises

### Exercise 1: Sensor Data Logger

Create a no_std sensor data logger with fixed-capacity storage:

```rust
#![no_std]

use heapless::{Vec, String};

#[derive(Clone, Copy, Debug)]
struct SensorReading {
    timestamp: u32,
    temperature: i16,
    humidity: u8,
    pressure: u16,
}

struct DataLogger<const N: usize> {
    readings: Vec<SensorReading, N>,
    total_readings: u32,
}

impl<const N: usize> DataLogger<N> {
    const fn new() -> Self {
        // TODO: Initialize data logger
        unimplemented!()
    }
    
    fn log_reading(&mut self, reading: SensorReading) -> Result<(), &'static str> {
        // TODO: Add reading to storage
        // If storage is full, remove oldest reading (circular buffer behavior)
        unimplemented!()
    }
    
    fn get_latest(&self) -> Option<SensorReading> {
        // TODO: Return most recent reading
        unimplemented!()
    }
    
    fn get_average_temperature(&self) -> Option<i16> {
        // TODO: Calculate average temperature from stored readings
        unimplemented!()
    }
    
    fn format_summary(&self) -> heapless::String<256> {
        // TODO: Format summary string without std::format!
        // Include: count, latest reading, average temperature
        unimplemented!()
    }
    
    fn clear(&mut self) {
        // TODO: Clear all stored readings
        unimplemented!()
    }
}

// Test your implementation
fn test_data_logger() {
    let mut logger: DataLogger<10> = DataLogger::new();
    
    // Log some readings
    for i in 0..15 {
        let reading = SensorReading {
            timestamp: i * 1000,
            temperature: 20 + (i as i16),
            humidity: 50 + (i as u8 % 20),
            pressure: 1013 + (i as u16),
        };
        logger.log_reading(reading).ok();
    }
    
    let summary = logger.format_summary();
    // Print summary (would use RTT or UART in real embedded system)
}
```

### Exercise 2: State Machine Controller

Implement a state machine for controlling an embedded device:

```rust
#![no_std]

#[derive(Clone, Copy, Debug, PartialEq)]
enum DeviceState {
    PowerOff,
    Initializing,
    Ready,
    Measuring,
    Transmitting,
    Error(ErrorCode),
    Shutdown,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum ErrorCode {
    SensorFault,
    CommunicationError,
    OverTemperature,
    LowBattery,
}

#[derive(Clone, Copy, Debug)]
enum Event {
    PowerOn,
    InitComplete,
    InitFailed(ErrorCode),
    StartMeasurement,
    MeasurementComplete,
    MeasurementFailed,
    TransmitData,
    TransmissionComplete,
    TransmissionFailed,
    ErrorRecovered,
    Shutdown,
}

struct StateMachine {
    current_state: DeviceState,
    measurement_count: u32,
    error_count: u32,
}

impl StateMachine {
    const fn new() -> Self {
        // TODO: Initialize state machine
        unimplemented!()
    }
    
    fn handle_event(&mut self, event: Event) -> DeviceState {
        // TODO: Implement state transitions based on current state and event
        // Return new state after transition
        unimplemented!()
    }
    
    fn can_handle_event(&self, event: Event) -> bool {
        // TODO: Check if current state can handle the given event
        unimplemented!()
    }
    
    fn is_operational(&self) -> bool {
        // TODO: Return true if device can perform measurements
        unimplemented!()
    }
    
    fn get_status_string(&self) -> &'static str {
        // TODO: Return human-readable status string
        unimplemented!()
    }
    
    fn reset(&mut self) {
        // TODO: Reset to initial state
        unimplemented!()
    }
}

// Test your implementation
fn test_state_machine() {
    let mut sm = StateMachine::new();
    
    assert_eq!(sm.current_state, DeviceState::PowerOff);
    
    // Power on sequence
    sm.handle_event(Event::PowerOn);
    assert_eq!(sm.current_state, DeviceState::Initializing);
    
    sm.handle_event(Event::InitComplete);
    assert_eq!(sm.current_state, DeviceState::Ready);
    
    // Measurement cycle
    sm.handle_event(Event::StartMeasurement);
    sm.handle_event(Event::MeasurementComplete);
    
    // Test error handling
    sm.handle_event(Event::InitFailed(ErrorCode::SensorFault));
    // Should handle error appropriately
}
```

### Exercise 3: Memory Pool Allocator

Create a custom memory pool for managing buffers:

```rust
#![no_std]

use core::mem::{MaybeUninit, size_of, align_of};
use core::ptr::{self, NonNull};

struct MemoryPool<T, const N: usize> {
    // TODO: Define pool structure
    // Hint: Use an array for storage and a free list
}

struct PoolHandle<T> {
    // TODO: Handle that manages allocated memory
    // Should automatically return memory to pool when dropped
}

impl<T, const N: usize> MemoryPool<T, N> {
    const fn new() -> Self {
        // TODO: Initialize empty pool
        unimplemented!()
    }
    
    fn init(&mut self) {
        // TODO: Set up free list linking all blocks
        unimplemented!()
    }
    
    fn alloc(&mut self) -> Option<PoolHandle<T>> {
        // TODO: Allocate block from free list
        unimplemented!()
    }
    
    fn free_count(&self) -> usize {
        // TODO: Return number of available blocks
        unimplemented!()
    }
    
    fn total_count(&self) -> usize {
        N
    }
    
    unsafe fn free(&mut self, ptr: NonNull<T>) {
        // TODO: Return block to free list
        // This should be called by PoolHandle::drop
        unimplemented!()
    }
}

impl<T> PoolHandle<T> {
    unsafe fn new(ptr: NonNull<T>, pool: *mut dyn PoolFree<T>) -> Self {
        // TODO: Create new handle
        unimplemented!()
    }
    
    fn as_ptr(&self) -> *mut T {
        // TODO: Get raw pointer to allocated memory
        unimplemented!()
    }
}

impl<T> Drop for PoolHandle<T> {
    fn drop(&mut self) {
        // TODO: Return memory to pool
        unimplemented!()
    }
}

// Trait for returning memory to pool (needed for Handle to work with any pool)
trait PoolFree<T> {
    unsafe fn free(&mut self, ptr: NonNull<T>);
}

impl<T, const N: usize> PoolFree<T> for MemoryPool<T, N> {
    unsafe fn free(&mut self, ptr: NonNull<T>) {
        self.free(ptr);
    }
}

// Test your implementation
fn test_memory_pool() {
    let mut pool: MemoryPool<[u8; 64], 8> = MemoryPool::new();
    pool.init();
    
    assert_eq!(pool.free_count(), 8);
    
    // Allocate some blocks
    let block1 = pool.alloc().unwrap();
    let block2 = pool.alloc().unwrap();
    
    assert_eq!(pool.free_count(), 6);
    
    // Use blocks
    unsafe {
        let ptr1 = block1.as_ptr();
        (*ptr1)[0] = 42;
    }
    
    // Blocks automatically freed when dropped
    drop(block1);
    drop(block2);
    
    assert_eq!(pool.free_count(), 8);
}
```

## Key Takeaways

1. **Understand Library Layers**: `core` is always available, `alloc` adds heap allocation, `std` adds OS features
2. **Use Heapless Collections**: Fixed-capacity alternatives prevent memory allocation failures
3. **Leverage Const Functions**: Compute values at compile time to reduce runtime overhead
4. **Memory Management**: Use pools, ring buffers, and static allocation instead of heap when possible
5. **Error Handling**: Custom error types with `Result` provide type-safe error handling
6. **State Machines**: Explicit state management prevents invalid operations
7. **Interrupt Safety**: Use `Mutex<RefCell<T>>` for interrupt-safe shared data
8. **Resource Constraints**: Always consider memory, power, and timing constraints in embedded contexts

**Next**: In Chapter 17, we'll explore build systems, deployment strategies, and CI/CD for Rust projects.