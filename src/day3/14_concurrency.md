# Chapter 14: Concurrency Fundamentals

## Learning Objectives
- Master Rust's thread-safe shared state management using `Arc<Mutex<T>>` and `Arc<RwLock<T>>`
- Understand message passing with channels (`mpsc`) and when to use each approach
- Learn about `Send` and `Sync` traits and their role in thread safety
- Apply deadlock prevention strategies in multi-threaded code
- Use Rayon for data parallelism and performance optimization
- Compare Rust's concurrency model to C++/C# threading approaches

## Thread Safety: Send and Sync Traits

Rust's concurrency safety is built on two key traits:

```rust
// Send: Types that can be transferred between threads
// Sync: Types that can be safely shared between threads (T is Sync if &T is Send)

use std::thread;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

// Most types are Send and Sync automatically
fn demonstrate_send_sync() {
    let data = vec![1, 2, 3, 4, 5];
    
    // This works because Vec<i32> is Send
    let handle = thread::spawn(move || {
        println!("Data in thread: {:?}", data);
    });
    
    handle.join().unwrap();
}
```

**C++/C# Comparison:**
- **C++**: No built-in thread safety guarantees; developers must manually ensure thread safety
- **C#**: Thread safety is runtime-checked; race conditions possible
- **Rust**: Thread safety is compile-time guaranteed through Send/Sync traits

## Shared State with Arc<Mutex<T>>

`Arc<Mutex<T>>` is the primary pattern for shared mutable state:

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn shared_counter_example() {
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];
    
    for i in 0..10 {
        let counter_clone = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            for _ in 0..1000 {
                let mut num = counter_clone.lock().unwrap();
                *num += 1;
                // Mutex is automatically released when `num` goes out of scope
            }
            println!("Thread {} finished", i);
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    println!("Final counter value: {}", *counter.lock().unwrap());
    // Output: Final counter value: 10000
}

// Better error handling with Mutex
fn safe_shared_counter() -> Result<i32, Box<dyn std::error::Error>> {
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];
    
    for _ in 0..10 {
        let counter_clone = Arc::clone(&counter);
        let handle = thread::spawn(move || -> Result<(), Box<dyn std::error::Error + Send>> {
            for _ in 0..1000 {
                let mut num = counter_clone.lock()
                    .map_err(|e| format!("Mutex poisoned: {}", e))?;
                *num += 1;
            }
            Ok(())
        });
        handles.push(handle);
    }
    
    // Collect results and handle errors
    for handle in handles {
        handle.join().unwrap()?;
    }
    
    let final_value = counter.lock()
        .map_err(|e| format!("Final lock failed: {}", e))?;
    Ok(*final_value)
}
```

## Reader-Writer Locks: Arc<RwLock<T>>

When you have many readers and few writers, `RwLock` can be more efficient:

```rust
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

fn rwlock_example() {
    let data = Arc::new(RwLock::new(vec![1, 2, 3, 4, 5]));
    let mut handles = vec![];
    
    // Spawn reader threads
    for i in 0..5 {
        let data_clone = Arc::clone(&data);
        let handle = thread::spawn(move || {
            for _ in 0..10 {
                let data_guard = data_clone.read().unwrap();
                println!("Reader {} sees: {:?}", i, *data_guard);
                thread::sleep(Duration::from_millis(10));
            }
        });
        handles.push(handle);
    }
    
    // Spawn writer threads
    for i in 0..2 {
        let data_clone = Arc::clone(&data);
        let handle = thread::spawn(move || {
            for j in 0..5 {
                let mut data_guard = data_clone.write().unwrap();
                data_guard.push(i * 10 + j);
                println!("Writer {} added: {}", i, i * 10 + j);
                thread::sleep(Duration::from_millis(50));
            }
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    println!("Final data: {:?}", *data.read().unwrap());
}
```

## Message Passing with Channels

Channels provide a safer alternative to shared state for many use cases:

```rust
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn basic_channel_example() {
    let (tx, rx) = mpsc::channel();
    
    // Spawn producer thread
    thread::spawn(move || {
        let messages = vec![
            "Hello",
            "from",
            "the",
            "producer",
            "thread"
        ];
        
        for msg in messages {
            tx.send(msg).unwrap();
            thread::sleep(Duration::from_millis(100));
        }
        // tx is dropped here, which closes the channel
    });
    
    // Receive messages in main thread
    for received in rx {
        println!("Received: {}", received);
    }
}

// Multiple producer, single consumer
fn mpsc_example() {
    let (tx, rx) = mpsc::channel();
    
    // Clone sender for multiple producers
    for i in 0..3 {
        let tx_clone = tx.clone();
        thread::spawn(move || {
            for j in 0..5 {
                let msg = format!("Message {}-{}", i, j);
                tx_clone.send(msg).unwrap();
                thread::sleep(Duration::from_millis(10));
            }
        });
    }
    
    // Drop the original sender
    drop(tx);
    
    // Collect all messages
    let mut messages = Vec::new();
    for received in rx {
        messages.push(received);
    }
    
    messages.sort(); // Messages may arrive out of order
    println!("All messages: {:?}", messages);
}

// Synchronous channel for backpressure
fn sync_channel_example() {
    let (tx, rx) = mpsc::sync_channel(2); // Buffer size of 2
    
    thread::spawn(move || {
        for i in 0..5 {
            println!("Sending {}", i);
            tx.send(i).unwrap(); // This will block when buffer is full
            println!("Sent {}", i);
        }
    });
    
    thread::sleep(Duration::from_secs(1));
    
    for received in rx {
        println!("Received: {}", received);
        thread::sleep(Duration::from_millis(500)); // Slow consumer
    }
}
```

## Producer-Consumer Pattern

A common concurrency pattern combining channels and shared state:

```rust
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone)]
struct WorkItem {
    id: u32,
    data: String,
}

#[derive(Debug)]
struct WorkResult {
    item_id: u32,
    result: String,
    processing_time_ms: u64,
}

fn producer_consumer_example() {
    let (work_tx, work_rx) = mpsc::channel::<WorkItem>();
    let (result_tx, result_rx) = mpsc::channel::<WorkResult>();
    let work_rx = Arc::new(Mutex::new(work_rx));
    
    // Spawn multiple worker threads
    let num_workers = 3;
    for worker_id in 0..num_workers {
        let work_rx_clone = Arc::clone(&work_rx);
        let result_tx_clone = result_tx.clone();
        
        thread::spawn(move || {
            loop {
                let work_item = {
                    let rx = work_rx_clone.lock().unwrap();
                    rx.recv()
                };
                
                match work_item {
                    Ok(item) => {
                        let start = std::time::Instant::now();
                        
                        // Simulate work
                        let processed_data = item.data.to_uppercase();
                        thread::sleep(Duration::from_millis(100 + (item.id % 3) * 50));
                        
                        let result = WorkResult {
                            item_id: item.id,
                            result: processed_data,
                            processing_time_ms: start.elapsed().as_millis() as u64,
                        };
                        
                        result_tx_clone.send(result).unwrap();
                        println!("Worker {} processed item {}", worker_id, item.id);
                    }
                    Err(_) => {
                        println!("Worker {} shutting down", worker_id);
                        break;
                    }
                }
            }
        });
    }
    
    // Producer thread
    thread::spawn(move || {
        for i in 0..10 {
            let item = WorkItem {
                id: i,
                data: format!("task-{}", i),
            };
            work_tx.send(item).unwrap();
            thread::sleep(Duration::from_millis(50));
        }
        // Channel closes when work_tx is dropped
    });
    
    // Drop our copy of result_tx so the channel closes when workers are done
    drop(result_tx);
    
    // Collect results
    let mut results = Vec::new();
    for result in result_rx {
        results.push(result);
    }
    
    results.sort_by_key(|r| r.item_id);
    for result in results {
        println!("Result: {:?}", result);
    }
}
```

## Deadlock Prevention

Common strategies to avoid deadlocks:

```rust
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// BAD: Potential deadlock
fn deadlock_example() {
    let resource1 = Arc::new(Mutex::new(1));
    let resource2 = Arc::new(Mutex::new(2));
    
    let r1_clone = Arc::clone(&resource1);
    let r2_clone = Arc::clone(&resource2);
    
    let handle1 = thread::spawn(move || {
        let _guard1 = r1_clone.lock().unwrap();
        thread::sleep(Duration::from_millis(10)); // Simulate work
        let _guard2 = r2_clone.lock().unwrap(); // Potential deadlock here
        println!("Thread 1 completed");
    });
    
    let handle2 = thread::spawn(move || {
        let _guard2 = resource2.lock().unwrap();
        thread::sleep(Duration::from_millis(10)); // Simulate work
        let _guard1 = resource1.lock().unwrap(); // Potential deadlock here
        println!("Thread 2 completed");
    });
    
    handle1.join().unwrap();
    handle2.join().unwrap();
}

// GOOD: Ordered lock acquisition prevents deadlock
fn deadlock_prevention() {
    let resource1 = Arc::new(Mutex::new(1));
    let resource2 = Arc::new(Mutex::new(2));
    
    let r1_clone = Arc::clone(&resource1);
    let r2_clone = Arc::clone(&resource2);
    
    let handle1 = thread::spawn(move || {
        // Always lock resource1 first, then resource2
        let _guard1 = r1_clone.lock().unwrap();
        thread::sleep(Duration::from_millis(10));
        let _guard2 = r2_clone.lock().unwrap();
        println!("Thread 1 completed safely");
    });
    
    let handle2 = thread::spawn(move || {
        // Same order: resource1 first, then resource2
        let _guard1 = resource1.lock().unwrap();
        thread::sleep(Duration::from_millis(10));
        let _guard2 = resource2.lock().unwrap();
        println!("Thread 2 completed safely");
    });
    
    handle1.join().unwrap();
    handle2.join().unwrap();
}

// Using try_lock to avoid blocking
fn try_lock_pattern() {
    let resource = Arc::new(Mutex::new(42));
    let resource_clone = Arc::clone(&resource);
    
    thread::spawn(move || {
        let _guard = resource_clone.lock().unwrap();
        thread::sleep(Duration::from_millis(100)); // Hold lock for a while
    });
    
    thread::sleep(Duration::from_millis(10)); // Let other thread acquire lock
    
    match resource.try_lock() {
        Ok(guard) => println!("Got lock: {}", *guard),
        Err(_) => println!("Lock is busy, doing something else instead"),
    }
}
```

## Data Parallelism with Rayon

Rayon provides easy data parallelism without explicit thread management:

```rust
use rayon::prelude::*;

fn rayon_examples() {
    // Parallel iterator operations
    let numbers: Vec<i32> = (0..1_000_000).collect();
    
    // Parallel map
    let squares: Vec<i32> = numbers
        .par_iter()
        .map(|&x| x * x)
        .collect();
    
    println!("First 10 squares: {:?}", &squares[..10]);
    
    // Parallel filtering and reduction
    let sum_of_even_squares: i32 = numbers
        .par_iter()
        .filter(|&&x| x % 2 == 0)
        .map(|&x| x * x)
        .sum();
    
    println!("Sum of even squares: {}", sum_of_even_squares);
    
    // Parallel sorting
    let mut data: Vec<i32> = (0..100_000).rev().collect();
    data.par_sort_unstable();
    println!("Data is sorted: {}", is_sorted(&data));
}

fn is_sorted<T: Ord>(slice: &[T]) -> bool {
    slice.windows(2).all(|w| w[0] <= w[1])
}

// Custom parallel work
fn parallel_file_processing() {
    let filenames: Vec<String> = (0..100)
        .map(|i| format!("file_{}.txt", i))
        .collect();
    
    let results: Vec<_> = filenames
        .par_iter()
        .map(|filename| {
            // Simulate file processing
            let size = filename.len() * 1024; // Mock file size
            (filename.clone(), size)
        })
        .collect();
    
    let total_size: usize = results.iter().map(|(_, size)| size).sum();
    println!("Processed {} files, total size: {} bytes", results.len(), total_size);
}

// Parallel fold with custom operations
fn parallel_fold_example() {
    let numbers: Vec<f64> = (1..=1_000_000).map(|x| x as f64).collect();
    
    // Calculate mean using parallel fold
    let (sum, count) = numbers
        .par_iter()
        .fold(|| (0.0, 0), |acc, &x| (acc.0 + x, acc.1 + 1))
        .reduce(|| (0.0, 0), |a, b| (a.0 + b.0, a.1 + b.1));
    
    let mean = sum / count as f64;
    println!("Mean: {}", mean);
    
    // Calculate standard deviation
    let variance = numbers
        .par_iter()
        .map(|&x| (x - mean).powi(2))
        .sum::<f64>() / count as f64;
    
    println!("Standard deviation: {}", variance.sqrt());
}
```

## Common Pitfalls and Solutions

### 1. Mutex Poisoning

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn handle_poisoned_mutex() {
    let data = Arc::new(Mutex::new(vec![1, 2, 3]));
    let data_clone = Arc::clone(&data);
    
    // Thread that panics while holding the mutex
    let handle = thread::spawn(move || {
        let mut guard = data_clone.lock().unwrap();
        guard.push(4);
        panic!("Simulated panic!"); // This poisons the mutex
    });
    
    // This will fail
    let _ = handle.join();
    
    // Handle poisoned mutex properly
    match data.lock() {
        Ok(guard) => println!("Data: {:?}", *guard),
        Err(poisoned) => {
            println!("Mutex was poisoned, but we can recover the data");
            let guard = poisoned.into_inner();
            println!("Recovered data: {:?}", *guard);
        }
    }
}
```

### 2. Avoiding Arc<Mutex<T>> When Possible

```rust
use std::sync::mpsc;
use std::thread;

// Instead of shared mutable state, use message passing
fn prefer_message_passing() {
    let (tx, rx) = mpsc::channel();
    
    // Spawn data processor thread
    thread::spawn(move || {
        let mut data = vec![1, 2, 3];
        
        for command in rx {
            match command {
                Command::Add(value) => data.push(value),
                Command::Get(response_tx) => {
                    response_tx.send(data.clone()).unwrap();
                }
                Command::Stop => break,
            }
        }
    });
    
    // Use the data processor
    tx.send(Command::Add(4)).unwrap();
    tx.send(Command::Add(5)).unwrap();
    
    let (response_tx, response_rx) = mpsc::channel();
    tx.send(Command::Get(response_tx)).unwrap();
    let data = response_rx.recv().unwrap();
    
    println!("Data: {:?}", data);
    tx.send(Command::Stop).unwrap();
}

#[derive(Debug)]
enum Command {
    Add(i32),
    Get(mpsc::Sender<Vec<i32>>),
    Stop,
}
```

## Exercises

### Exercise 1: Thread Pool Implementation

Create a simple thread pool that can execute closures:

```rust
use std::sync::{Arc, Mutex, mpsc};
use std::thread;

struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        // TODO: Implement thread pool creation
        unimplemented!()
    }
    
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        // TODO: Send job to worker thread
        unimplemented!()
    }
}

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        // TODO: Create worker that processes jobs from receiver
        unimplemented!()
    }
}

// Test your implementation
fn test_thread_pool() {
    let pool = ThreadPool::new(4);
    
    for i in 0..8 {
        pool.execute(move || {
            println!("Executing task {}", i);
            thread::sleep(Duration::from_millis(100));
        });
    }
    
    thread::sleep(Duration::from_secs(1));
}
```

### Exercise 2: Concurrent Cache

Implement a thread-safe cache with expiration:

```rust
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

struct CacheEntry<T> {
    value: T,
    expires_at: Instant,
}

struct Cache<K, V> {
    data: Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
    default_ttl: Duration,
}

impl<K, V> Cache<K, V>
where
    K: Clone + Eq + std::hash::Hash,
    V: Clone,
{
    pub fn new(default_ttl: Duration) -> Self {
        // TODO: Implement cache creation
        unimplemented!()
    }
    
    pub fn get(&self, key: &K) -> Option<V> {
        // TODO: Get value if not expired, clean up expired entries
        unimplemented!()
    }
    
    pub fn set(&self, key: K, value: V) {
        // TODO: Insert value with expiration time
        unimplemented!()
    }
    
    pub fn set_with_ttl(&self, key: K, value: V, ttl: Duration) {
        // TODO: Insert value with custom TTL
        unimplemented!()
    }
    
    pub fn cleanup_expired(&self) {
        // TODO: Remove all expired entries
        unimplemented!()
    }
}

// Test concurrent access
fn test_concurrent_cache() {
    let cache = Arc::new(Cache::new(Duration::from_millis(100)));
    let mut handles = vec![];
    
    // Writer threads
    for i in 0..5 {
        let cache_clone = Arc::clone(&cache);
        let handle = thread::spawn(move || {
            for j in 0..10 {
                cache_clone.set(format!("key-{}-{}", i, j), j);
                thread::sleep(Duration::from_millis(10));
            }
        });
        handles.push(handle);
    }
    
    // Reader threads
    for i in 0..3 {
        let cache_clone = Arc::clone(&cache);
        let handle = thread::spawn(move || {
            for j in 0..20 {
                if let Some(value) = cache_clone.get(&format!("key-0-{}", j % 10)) {
                    println!("Reader {} got value: {}", i, value);
                }
                thread::sleep(Duration::from_millis(5));
            }
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
}
```

### Exercise 3: Pipeline Processing

Create a multi-stage processing pipeline using channels:

```rust
use std::sync::mpsc;
use std::thread;

#[derive(Debug, Clone)]
struct DataItem {
    id: u32,
    content: String,
}

// Create a processing pipeline: Input -> Transform -> Filter -> Output
fn create_processing_pipeline() {
    let (input_tx, input_rx) = mpsc::channel();
    let (transform_tx, transform_rx) = mpsc::channel();
    let (filter_tx, filter_rx) = mpsc::channel();
    let (output_tx, output_rx) = mpsc::channel();
    
    // Stage 1: Transform (uppercase content)
    thread::spawn(move || {
        for item in input_rx {
            let transformed = DataItem {
                id: item.id,
                content: item.content.to_uppercase(),
            };
            transform_tx.send(transformed).unwrap();
        }
    });
    
    // Stage 2: Filter (only items with even IDs)
    thread::spawn(move || {
        // TODO: Implement filter stage
        unimplemented!()
    });
    
    // Stage 3: Output processing
    thread::spawn(move || {
        // TODO: Process filtered items
        unimplemented!()
    });
    
    // Generate input data
    for i in 0..20 {
        let item = DataItem {
            id: i,
            content: format!("item-{}", i),
        };
        input_tx.send(item).unwrap();
    }
    drop(input_tx);
    
    // Collect results
    for result in output_rx {
        println!("Final result: {:?}", result);
    }
}
```

## Key Takeaways

1. **Thread Safety is Guaranteed**: Rust's `Send` and `Sync` traits ensure thread safety at compile time
2. **Choose the Right Pattern**: Use `Arc<Mutex<T>>` for shared state, channels for message passing
3. **RwLock for Read-Heavy Workloads**: `Arc<RwLock<T>>` allows multiple concurrent readers
4. **Prevent Deadlocks**: Use consistent lock ordering and consider `try_lock()` for non-blocking attempts
5. **Rayon for Data Parallelism**: Easy parallel processing of collections with minimal code changes
6. **Handle Poisoned Mutexes**: Always handle the case where a thread panics while holding a mutex
7. **Prefer Message Passing**: Often cleaner and safer than shared mutable state
8. **Performance Considerations**: Measure before optimizing; sometimes single-threaded code is faster

**Next**: In Chapter 14, we'll explore async programming, which provides concurrency without the overhead of OS threads.