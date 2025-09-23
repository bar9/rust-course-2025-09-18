// Chapter 7: Traits - All 3 Exercises Solutions

use std::cmp::Ordering;

// ==========================
// Exercise 1: Comparable Trait
// ==========================

pub trait Comparable {
    fn compare(&self, other: &Self) -> Ordering;
    
    // Provide default implementations
    fn is_greater(&self, other: &Self) -> bool {
        matches!(self.compare(other), Ordering::Greater)
    }
    
    fn is_less(&self, other: &Self) -> bool {
        matches!(self.compare(other), Ordering::Less)
    }
    
    fn is_equal(&self, other: &Self) -> bool {
        matches!(self.compare(other), Ordering::Equal)
    }
}

#[derive(Debug)]
pub struct Person {
    pub name: String,
    pub age: u32,
}

impl Comparable for Person {
    fn compare(&self, other: &Self) -> Ordering {
        // Compare by age first, then by name
        match self.age.cmp(&other.age) {
            Ordering::Equal => self.name.cmp(&other.name),
            other_order => other_order,
        }
    }
}

#[derive(Debug)]
struct Product {
    name: String,
    price: f64,
}

impl Comparable for Product {
    fn compare(&self, other: &Self) -> Ordering {
        // Compare by price (handling f64)
        if self.price < other.price {
            Ordering::Less
        } else if self.price > other.price {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

// ==========================
// Exercise 2: Plugin System
// ==========================

trait Plugin {
    fn name(&self) -> &str;
    fn execute(&self);
}

trait Configurable {
    fn configure(&mut self, config: &str);
}

struct LogPlugin {
    name: String,
    level: String,
}

impl Plugin for LogPlugin {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn execute(&self) {
        println!("[{}] Logging at level: {}", self.name, self.level);
    }
}

impl Configurable for LogPlugin {
    fn configure(&mut self, config: &str) {
        self.level = config.to_string();
        println!("LogPlugin configured with level: {}", self.level);
    }
}

struct MetricsPlugin {
    name: String,
    interval: u32,
}

impl Plugin for MetricsPlugin {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn execute(&self) {
        println!("[{}] Collecting metrics every {} seconds", self.name, self.interval);
    }
}

impl Configurable for MetricsPlugin {
    fn configure(&mut self, config: &str) {
        if let Ok(interval) = config.parse::<u32>() {
            self.interval = interval;
            println!("MetricsPlugin configured with interval: {} seconds", self.interval);
        }
    }
}

struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginManager {
    fn new() -> Self {
        PluginManager { plugins: Vec::new() }
    }
    
    fn register(&mut self, plugin: Box<dyn Plugin>) {
        println!("Registering plugin: {}", plugin.name());
        self.plugins.push(plugin);
    }
    
    fn run_all(&self) {
        println!("\nExecuting all plugins:");
        for plugin in &self.plugins {
            plugin.execute();
        }
    }
}

// ==========================
// Exercise 3: Custom Iterator
// ==========================

trait FilterMap: Iterator {
    fn filter_map_custom<B, F>(self, f: F) -> FilterMapCustom<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> Option<B>,
    {
        FilterMapCustom { iter: self, f }
    }
}

struct FilterMapCustom<I, F> {
    iter: I,
    f: F,
}

impl<B, I: Iterator, F> Iterator for FilterMapCustom<I, F>
where
    F: FnMut(I::Item) -> Option<B>,
{
    type Item = B;
    
    fn next(&mut self) -> Option<Self::Item> {
        for item in self.iter.by_ref() {
            if let Some(result) = (self.f)(item) {
                return Some(result);
            }
        }
        None
    }
}

// Implement FilterMap for all iterators
impl<I: Iterator> FilterMap for I {}

// ==========================
// Main function demonstrating all exercises
// ==========================

pub fn demonstrate_traits() {
    println!("=== Day 2, Chapter 7: Traits Solutions ===\n");
    
    // Exercise 1: Comparable Trait
    println!("--- Exercise 1: Comparable Trait ---\n");
    
    let alice = Person { 
        name: "Alice".to_string(), 
        age: 30 
    };
    let bob = Person { 
        name: "Bob".to_string(), 
        age: 25 
    };
    let charlie = Person {
        name: "Charlie".to_string(),
        age: 30
    };
    
    println!("Alice (age 30) vs Bob (age 25):");
    println!("  Alice > Bob? {}", alice.is_greater(&bob));
    println!("  Alice < Bob? {}", alice.is_less(&bob));
    
    println!("\nAlice (age 30) vs Charlie (age 30):");
    println!("  Compare result: {:?}", alice.compare(&charlie));
    println!("  Alice == Charlie? {}", alice.is_equal(&charlie));
    
    let laptop = Product {
        name: "Laptop".to_string(),
        price: 999.99,
    };
    let phone = Product {
        name: "Phone".to_string(),
        price: 599.99,
    };
    
    println!("\nProduct comparison:");
    println!("  Laptop (${}) > Phone (${}): {}", 
             laptop.price, phone.price, laptop.is_greater(&phone));
    
    // Exercise 2: Plugin System
    println!("\n--- Exercise 2: Plugin System ---\n");
    
    let mut manager = PluginManager::new();
    
    // Create and configure plugins
    let mut log_plugin = LogPlugin {
        name: "Logger".to_string(),
        level: "INFO".to_string(),
    };
    log_plugin.configure("DEBUG");
    
    let mut metrics_plugin = MetricsPlugin {
        name: "Metrics".to_string(),
        interval: 60,
    };
    metrics_plugin.configure("30");
    
    // Register plugins
    manager.register(Box::new(log_plugin));
    manager.register(Box::new(metrics_plugin));
    
    // Run all plugins
    manager.run_all();
    
    // Exercise 3: Custom Iterator
    println!("\n--- Exercise 3: Custom Iterator ---\n");
    
    let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    
    // Use the custom filter_map
    let result: Vec<i32> = numbers
        .into_iter()
        .filter_map_custom(|x| {
            if x % 2 == 0 {
                Some(x * x)  // Square even numbers
            } else {
                None  // Filter out odd numbers
            }
        })
        .collect();
    
    println!("Even numbers squared: {:?}", result);
    
    // Another example with strings
    let words = vec!["hello", "world", "rust", "is", "awesome"];
    let long_words: Vec<String> = words
        .into_iter()
        .filter_map_custom(|word| {
            if word.len() > 3 {
                Some(word.to_uppercase())
            } else {
                None
            }
        })
        .collect();
    
    println!("Long words (>3 chars) in uppercase: {:?}", long_words);
    
    println!("\n✅ All trait exercises completed successfully!");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_comparable_person() {
        let younger = Person { 
            name: "Alice".to_string(), 
            age: 25 
        };
        let older = Person { 
            name: "Bob".to_string(), 
            age: 30 
        };
        
        assert!(younger.is_less(&older));
        assert!(older.is_greater(&younger));
    }
    
    #[test]
    fn test_comparable_product() {
        let cheap = Product { 
            name: "Item1".to_string(), 
            price: 10.0 
        };
        let expensive = Product { 
            name: "Item2".to_string(), 
            price: 100.0 
        };
        
        assert!(cheap.is_less(&expensive));
        assert!(expensive.is_greater(&cheap));
    }
    
    #[test]
    fn test_filter_map_custom() {
        let numbers = vec![1, 2, 3, 4];
        let result: Vec<i32> = numbers
            .into_iter()
            .filter_map_custom(|x| {
                if x % 2 == 0 {
                    Some(x * 2)
                } else {
                    None
                }
            })
            .collect();
        
        assert_eq!(result, vec![4, 8]);
    }
    
    #[test]
    fn test_plugin_system() {
        let mut manager = PluginManager::new();
        let plugin = LogPlugin {
            name: "TestLogger".to_string(),
            level: "INFO".to_string(),
        };
        
        manager.register(Box::new(plugin));
        assert_eq!(manager.plugins.len(), 1);
    }
}