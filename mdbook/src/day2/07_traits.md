# Chapter 7: Traits - Shared Behavior and Polymorphism
## Defining, Implementing, and Using Traits in Rust

### Learning Objectives
By the end of this chapter, you'll be able to:
- Define custom traits and implement them for various types
- Use trait bounds to constrain generic types
- Work with trait objects for dynamic dispatch
- Understand the difference between static and dynamic dispatch
- Apply common standard library traits effectively
- Use associated types and default implementations
- Handle trait coherence and orphan rules

---

## What Are Traits?

Traits define shared behavior that types can implement. They're similar to interfaces in C#/Java or concepts in C++20, but with some unique features.

### Traits vs Other Languages

| Concept | C++ | C#/Java | Rust |
|---------|-----|---------|------|
| Interface | Pure virtual class | Interface | Trait |
| Multiple inheritance | Yes (complex) | No (interfaces only) | Yes (traits) |
| Default implementations | No | Yes (C# 8+, Java 8+) | Yes |
| Associated types | No | No | Yes |
| Static dispatch | Templates | Generics | Generics |
| Dynamic dispatch | Virtual functions | Virtual methods | Trait objects |

### Basic Trait Definition

```rust
// Define a trait
trait Drawable {
    fn draw(&self);
    fn area(&self) -> f64;
    
    // Default implementation
    fn description(&self) -> String {
        format!("A drawable shape with area {}", self.area())
    }
}

// Implement the trait for different types
struct Circle {
    radius: f64,
}

struct Rectangle {
    width: f64,
    height: f64,
}

impl Drawable for Circle {
    fn draw(&self) {
        println!("Drawing a circle with radius {}", self.radius);
    }
    
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }
}

impl Drawable for Rectangle {
    fn draw(&self) {
        println!("Drawing a rectangle {}x{}", self.width, self.height);
    }
    
    fn area(&self) -> f64 {
        self.width * self.height
    }
    
    // Override default implementation
    fn description(&self) -> String {
        format!("A rectangle with dimensions {}x{}", self.width, self.height)
    }
}
```

---

## Standard Library Traits You Need to Know

### Debug and Display

```rust
use std::fmt;

#[derive(Debug)]  // Automatic Debug implementation
struct Point {
    x: f64,
    y: f64,
}

// Manual Display implementation
impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

fn main() {
    let p = Point { x: 1.0, y: 2.0 };
    println!("{:?}", p);  // Debug: Point { x: 1.0, y: 2.0 }
    println!("{}", p);    // Display: (1.0, 2.0)
}
```

### Clone and Copy

```rust
#[derive(Clone, Copy, Debug)]
struct SmallData {
    value: i32,
}

#[derive(Clone, Debug)]
struct LargeData {
    data: Vec<i32>,
}

fn main() {
    let small = SmallData { value: 42 };
    let small_copy = small;     // Copy happens automatically
    println!("{:?}", small);   // Still usable after copy
    
    let large = LargeData { data: vec![1, 2, 3] };
    let large_clone = large.clone();  // Explicit clone needed
    // large moved here, but we have large_clone
}
```

---

## Generic Functions with Trait Bounds

### Basic Trait Bounds

```rust
use std::fmt::Display;

// Function that works with any type implementing Display
fn print_info<T: Display>(item: T) {
    println!("Info: {}", item);
}

// Multiple trait bounds
fn print_and_compare<T: Display + PartialEq>(item1: T, item2: T) {
    println!("Item 1: {}", item1);
    println!("Item 2: {}", item2);
    println!("Are equal: {}", item1 == item2);
}

// Where clause for complex bounds
fn complex_function<T, U>(t: T, u: U) -> String
where
    T: Display + Clone,
    U: std::fmt::Debug + Default,
{
    format!("{} and {:?}", t, u)
}
```

---

## Trait Objects and Dynamic Dispatch

### Creating Trait Objects

```rust
trait Animal {
    fn make_sound(&self);
    fn name(&self) -> &str;
}

struct Dog { name: String }
struct Cat { name: String }

impl Animal for Dog {
    fn make_sound(&self) { println!("Woof!"); }
    fn name(&self) -> &str { &self.name }
}

impl Animal for Cat {
    fn make_sound(&self) { println!("Meow!"); }
    fn name(&self) -> &str { &self.name }
}

// Using trait objects
fn main() {
    // Vec of trait objects
    let animals: Vec<Box<dyn Animal>> = vec![
        Box::new(Dog { name: "Buddy".to_string() }),
        Box::new(Cat { name: "Whiskers".to_string() }),
    ];
    
    for animal in &animals {
        println!("{} says:", animal.name());
        animal.make_sound();
    }
    
    // Function parameter as trait object
    pet_animal(&Dog { name: "Rex".to_string() });
}

fn pet_animal(animal: &dyn Animal) {
    println!("Petting {}", animal.name());
    animal.make_sound();
}
```

---

## Associated Types

### Basic Associated Types

```rust
trait Iterator {
    type Item;  // Associated type
    
    fn next(&mut self) -> Option<Self::Item>;
}

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
    type Item = u32;  // Specify the associated type
    
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
```

---

## Key Takeaways

1. **Traits define shared behavior** across different types
2. **Static dispatch** (generics) is faster but increases code size
3. **Dynamic dispatch** (trait objects) enables runtime polymorphism
4. **Associated types** provide cleaner APIs than generic parameters
5. **Coherence rules** prevent conflicting implementations
6. **Default implementations** reduce boilerplate code
7. **Trait bounds** should be as minimal as possible

**Next Up:** In Chapter 8, we'll explore generics - Rust's powerful system for writing flexible, reusable code with type parameters.
