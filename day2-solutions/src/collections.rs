// Chapter 6: Collections - LRU Cache Solution

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

#[derive(Clone)]
struct CacheEntry<V: Clone> {
    value: V,
    last_accessed: Instant,
    expires_at: Option<Instant>,
}

struct LRUCache<K: Clone + Eq + std::hash::Hash, V: Clone> {
    capacity: usize,
    cache: HashMap<K, CacheEntry<V>>,
    access_order: VecDeque<K>,
}

impl<K: Clone + Eq + std::hash::Hash, V: Clone> LRUCache<K, V> {
    fn new(capacity: usize) -> Self {
        LRUCache {
            capacity,
            cache: HashMap::new(),
            access_order: VecDeque::new(),
        }
    }
    
    fn get(&mut self, key: &K) -> Option<&V> {
        // Check if key exists
        if !self.cache.contains_key(key) {
            return None;
        }
        
        // Check if entry is expired
        let now = Instant::now();
        if let Some(entry) = self.cache.get(key) {
            if let Some(expires_at) = entry.expires_at {
                if now >= expires_at {
                    // Remove expired entry
                    self.cache.remove(key);
                    self.access_order.retain(|k| k != key);
                    return None;
                }
            }
        }
        
        // Update last_accessed time and move to end (most recently used)
        if let Some(entry) = self.cache.get_mut(key) {
            entry.last_accessed = now;
        }
        
        // Move key to end of access_order
        self.access_order.retain(|k| k != key);
        self.access_order.push_back(key.clone());
        
        // Return the value
        self.cache.get(key).map(|entry| &entry.value)
    }
    
    fn insert(&mut self, key: K, value: V, ttl: Option<Duration>) {
        // If at capacity and key doesn't exist, remove least recently used
        if self.cache.len() >= self.capacity && !self.cache.contains_key(&key) {
            if let Some(lru_key) = self.access_order.pop_front() {
                self.cache.remove(&lru_key);
            }
        }
        
        // Create cache entry with expiration if ttl provided
        let now = Instant::now();
        let expires_at = ttl.map(|duration| now + duration);
        
        let entry = CacheEntry {
            value,
            last_accessed: now,
            expires_at,
        };
        
        // Remove key from access_order if it exists (will add at end)
        self.access_order.retain(|k| k != &key);
        
        // Add to cache and access_order
        self.cache.insert(key.clone(), entry);
        self.access_order.push_back(key);
    }
    
    fn remove(&mut self, key: &K) -> Option<V> {
        // Remove from access_order
        self.access_order.retain(|k| k != key);
        
        // Remove from cache and return value
        self.cache.remove(key).map(|entry| entry.value)
    }
    
    fn clear_expired(&mut self) {
        let now = Instant::now();
        
        // Find all expired keys
        let expired_keys: Vec<K> = self.cache
            .iter()
            .filter_map(|(key, entry)| {
                if let Some(expires_at) = entry.expires_at {
                    if now >= expires_at {
                        Some(key.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();
        
        // Remove expired entries
        for key in expired_keys {
            self.cache.remove(&key);
            self.access_order.retain(|k| k != &key);
        }
    }
    
    fn stats(&self) -> (usize, usize) {
        (self.cache.len(), self.capacity)
    }
}

pub fn demonstrate_lru_cache() {
    let mut cache = LRUCache::new(3);
    
    // Test basic operations
    println!("LRU Cache demonstration:");
    cache.insert("user:1", "Alice", Some(Duration::from_secs(60)));
    cache.insert("user:2", "Bob", None);
    cache.insert("user:3", "Charlie", Some(Duration::from_secs(5)));
    
    let (size, capacity) = cache.stats();
    println!("  Cache stats - Size: {}/{}", size, capacity);
    
    // Access user:1 to make it most recently used  
    if let Some(name) = cache.get(&"user:1") {
        println!("  Accessed user:1: {}", name);
    }
    
    // Add one more - should evict user:2 (least recently used)
    cache.insert("user:4", "David", None);
    
    // Try to get user:2 - should be None (evicted)
    match cache.get(&"user:2") {
        Some(name) => println!("  Found user:2: {} (unexpected!)", name),
        None => println!("  ✅ User:2 correctly evicted (LRU)"),
    }
}

// Original main function kept for standalone testing
#[cfg(test)]
fn main_original() {
    println!("=== LRU Cache Demo ===\n");
    
    let mut cache = LRUCache::new(3);
    
    // Test basic operations
    println!("Inserting 3 users...");
    cache.insert("user:1", "Alice", Some(Duration::from_secs(60)));
    cache.insert("user:2", "Bob", None);  // No expiration
    cache.insert("user:3", "Charlie", Some(Duration::from_secs(5)));
    
    let (size, capacity) = cache.stats();
    println!("Cache stats after insert - Size: {}/{}\n", size, capacity);
    
    // Access user:1 to make it most recently used
    println!("Accessing user:1...");
    if let Some(name) = cache.get(&"user:1") {
        println!("Got: {}", name);
    }
    
    // Add one more - should evict user:2 (least recently used)
    println!("\nAdding user:4 (should evict user:2)...");
    cache.insert("user:4", "David", None);
    
    // Try to get user:2 - should be None (evicted)
    println!("Trying to access user:2 (should be evicted):");
    match cache.get(&"user:2") {
        Some(name) => println!("Found: {} (unexpected!)", name),
        None => println!("User 2 not found (correctly evicted)"),
    }
    
    let (size, capacity) = cache.stats();
    println!("\nFinal cache stats - Size: {}/{}", size, capacity);
    
    // Test expiration
    println!("\n=== Testing Expiration ===\n");
    
    let mut cache2 = LRUCache::new(5);
    cache2.insert("temp", "Temporary", Some(Duration::from_millis(100)));
    cache2.insert("permanent", "Permanent", None);
    
    println!("Before expiration:");
    println!("  temp: {:?}", cache2.get(&"temp"));
    println!("  permanent: {:?}", cache2.get(&"permanent"));
    
    // Wait for expiration
    std::thread::sleep(Duration::from_millis(150));
    
    println!("\nAfter expiration:");
    println!("  temp: {:?}", cache2.get(&"temp"));
    println!("  permanent: {:?}", cache2.get(&"permanent"));
    
    // Test clear_expired
    println!("\n=== Testing Clear Expired ===\n");
    
    let mut cache3 = LRUCache::new(10);
    for i in 0..5 {
        cache3.insert(
            format!("key:{}", i),
            format!("value:{}", i),
            Some(Duration::from_millis(50)),
        );
    }
    
    let (size, _) = cache3.stats();
    println!("Before clear_expired: {} entries", size);
    
    std::thread::sleep(Duration::from_millis(100));
    cache3.clear_expired();
    
    let (size, _) = cache3.stats();
    println!("After clear_expired: {} entries", size);
    
    println!("\n✅ All cache operations completed successfully!");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_operations() {
        let mut cache = LRUCache::new(2);
        
        cache.insert("key1", "value1", None);
        cache.insert("key2", "value2", None);
        
        assert_eq!(cache.get(&"key1"), Some(&"value1"));
        assert_eq!(cache.get(&"key2"), Some(&"value2"));
    }
    
    #[test]
    fn test_lru_eviction() {
        let mut cache = LRUCache::new(2);
        
        cache.insert("key1", "value1", None);
        cache.insert("key2", "value2", None);
        cache.insert("key3", "value3", None); // Should evict key1
        
        assert_eq!(cache.get(&"key1"), None);
        assert_eq!(cache.get(&"key2"), Some(&"value2"));
        assert_eq!(cache.get(&"key3"), Some(&"value3"));
    }
    
    #[test]
    fn test_expiration() {
        let mut cache = LRUCache::new(5);
        
        cache.insert("temp", "value", Some(Duration::from_millis(50)));
        assert_eq!(cache.get(&"temp"), Some(&"value"));
        
        std::thread::sleep(Duration::from_millis(100));
        assert_eq!(cache.get(&"temp"), None);
    }
    
    #[test]
    fn test_remove() {
        let mut cache = LRUCache::new(5);
        
        cache.insert("key", "value", None);
        assert_eq!(cache.remove(&"key"), Some("value"));
        assert_eq!(cache.get(&"key"), None);
    }
}