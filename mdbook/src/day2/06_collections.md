# Chapter 6: Collections - Your Data Structure Toolkit
## Vec, HashMap, HashSet, and When to Use Each

### Learning Objectives
By the end of this chapter, you'll be able to:
- Master Vec<T> for dynamic arrays and sequences
- Use HashMap<K, V> efficiently for key-value storage
- Apply HashSet<T> for unique value collections
- Choose the right collection for different use cases
- Understand performance characteristics and trade-offs
- Work with collection iterators and common operations
- Handle edge cases and avoid common pitfalls

---

## Collection Overview

### Rust's Standard Collections vs C++/.NET

| Rust Collection | C++ Equivalent | C#/.NET Equivalent | Use Case |
|-----------------|-----------------|-------------------|----------|
| `Vec<T>` | `std::vector<T>` | `List<T>` | Dynamic arrays |
| `HashMap<K,V>` | `std::unordered_map<K,V>` | `Dictionary<K,V>` | Key-value pairs |
| `HashSet<T>` | `std::unordered_set<T>` | `HashSet<T>` | Unique values |
| `BTreeMap<K,V>` | `std::map<K,V>` | `SortedDictionary<K,V>` | Sorted key-value |
| `BTreeSet<T>` | `std::set<T>` | `SortedSet<T>` | Sorted unique values |
| `VecDeque<T>` | `std::deque<T>` | `LinkedList<T>` | Double-ended queue |

### Performance Characteristics

| Collection | Access | Search | Insert | Delete | Memory |
|------------|--------|---------|--------|--------|---------|
| `Vec<T>` | O(1) | O(n) | O(1) amortized* | O(n) | Contiguous |
| `HashMap<K,V>` | O(1) average | O(1) average | O(1) average | O(1) average | Hash table |
| `HashSet<T>` | N/A | O(1) average | O(1) average | O(1) average | Hash table |
| `BTreeMap<K,V>` | O(log n) | O(log n) | O(log n) | O(log n) | Tree |

*Insert at end is O(1) amortized, O(n) at arbitrary position

---

## Vec<T>: The Workhorse Dynamic Array

### Creation and Basic Operations

```rust
fn main() {
    // Creation methods
    let mut vec1 = Vec::new();              // Empty vector
    let mut vec2: Vec<i32> = Vec::new();    // With type annotation
    let vec3 = vec![1, 2, 3, 4, 5];        // vec! macro
    let vec4 = Vec::with_capacity(10);      // Pre-allocated capacity
    
    // Adding elements
    vec1.push(1);
    vec1.push(2);
    vec1.extend([3, 4, 5]);                 // Add multiple elements
    
    // Accessing elements
    let first = vec1[0];                    // Panics if out of bounds
    let first_safe = vec1.get(0);           // Returns Option<&T>
    let last = vec1.last();                 // Returns Option<&T>
    
    // Safe element access
    match vec1.get(10) {
        Some(value) => println!("Value: {}", value),
        None => println!("Index out of bounds"),
    }
    
    println!("Vec: {:?}", vec1);
    println!("Length: {}, Capacity: {}", vec1.len(), vec1.capacity());
}
```

### Iteration Patterns

```rust
fn iteration_examples() {
    let vec = vec![1, 2, 3, 4, 5];
    
    // Immutable iteration (borrowing)
    for item in &vec {
        println!("Item: {}", item);
    }
    // vec is still usable here
    
    // Mutable iteration (borrowing mutably)
    let mut vec_mut = vec![1, 2, 3, 4, 5];
    for item in &mut vec_mut {
        *item *= 2;  // Modify in place
    }
    println!("Doubled: {:?}", vec_mut);
    
    // Consuming iteration (takes ownership)
    for item in vec {  // vec is moved here
        println!("Owned item: {}", item);
    }
    // vec is no longer usable
    
    // Index-based iteration
    let vec2 = vec![10, 20, 30];
    for (index, value) in vec2.iter().enumerate() {
        println!("Index {}: {}", index, value);
    }
}
```

### Common Vec Operations

```rust
fn vec_operations() {
    let mut numbers = vec![1, 2, 3, 4, 5];
    
    // Insertion
    numbers.insert(0, 0);               // Insert at beginning (expensive)
    numbers.insert(numbers.len(), 6);   // Insert at end (like push)
    
    // Removal
    let popped = numbers.pop();         // Remove last: Some(6)
    let removed = numbers.remove(0);    // Remove at index (expensive): 0
    
    // Slicing
    let slice = &numbers[1..4];         // Slice: [2, 3, 4]
    let first_three = &numbers[..3];    // First three elements
    let last_two = &numbers[numbers.len()-2..];  // Last two elements
    
    // Searching
    if numbers.contains(&3) {
        println!("Found 3!");
    }
    
    let position = numbers.iter().position(|&x| x == 4);
    println!("Position of 4: {:?}", position);
    
    // Sorting and deduplication
    let mut words = vec!["banana", "apple", "cherry", "apple"];
    words.sort();                       // Sort in place
    words.dedup();                      // Remove consecutive duplicates
    
    println!("Sorted unique words: {:?}", words);
}
```

### Advanced Vec Techniques

```rust
fn advanced_vec_techniques() {
    // Pre-sizing for performance
    let mut vec = Vec::with_capacity(1000);
    for i in 0..1000 {
        vec.push(i);  // No reallocations needed
    }
    
    // Splitting and joining
    let mut data = vec![1, 2, 3, 4, 5, 6];
    let (left, right) = data.split_at_mut(3);
    left[0] = 10;
    right[0] = 40;
    println!("Split and modified: {:?}", data);  // [10, 2, 3, 40, 5, 6]
    
    // Retain elements matching condition
    let mut numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    numbers.retain(|&x| x % 2 == 0);    // Keep only even numbers
    println!("Even numbers: {:?}", numbers);
    
    // Convert between Vec and other collections
    let vec_from_iter: Vec<i32> = (0..5).collect();
    let string_from_chars: String = vec!['H', 'e', 'l', 'l', 'o'].into_iter().collect();
}
```

---

## HashMap<K, V>: Key-Value Storage

### Creation and Basic Operations

```rust
use std::collections::HashMap;

fn hashmap_basics() {
    // Creation methods
    let mut scores = HashMap::new();
    scores.insert("Alice".to_string(), 100);
    scores.insert("Bob".to_string(), 85);
    scores.insert("Carol".to_string(), 92);
    
    // From iterator
    let teams = vec!["Blue", "Yellow", "Red"];
    let initial_scores = vec![10, 50, 25];
    let team_scores: HashMap<_, _> = teams
        .into_iter()
        .zip(initial_scores.into_iter())
        .collect();
    
    // Accessing values
    let alice_score = scores.get("Alice");
    match alice_score {
        Some(score) => println!("Alice's score: {}", score),
        None => println!("Alice not found"),
    }
    
    // Direct access (panics if key doesn't exist)
    // let bob_score = scores["Bob"];  // Risky!
    
    // Safe access with default
    let charlie_score = scores.get("Charlie").unwrap_or(&0);
    println!("Charlie's score: {}", charlie_score);
    
    // Check if key exists
    if scores.contains_key("Alice") {
        println!("Alice is in the map");
    }
    
    println!("All scores: {:?}", scores);
}
```

### The Entry API: Powerful Key-Value Operations

```rust
use std::collections::HashMap;

fn entry_api_examples() {
    let mut word_count = HashMap::new();
    let text = "the quick brown fox jumps over the lazy dog the";
    
    // Count words using entry API
    for word in text.split_whitespace() {
        let count = word_count.entry(word.to_string()).or_insert(0);
        *count += 1;
    }
    
    println!("Word counts: {:?}", word_count);
    
    // Entry patterns
    let mut player_scores = HashMap::new();
    
    // Insert if not present
    player_scores.entry("Alice".to_string()).or_insert(0);
    
    // Insert with computed value if not present
    player_scores.entry("Bob".to_string()).or_insert_with(|| {
        // Expensive computation here
        42
    });
    
    // Modify existing or insert new
    *player_scores.entry("Alice".to_string()).or_insert(0) += 10;
    
    // Pattern: update or set default
    let alice_entry = player_scores.entry("Alice".to_string());
    match alice_entry {
        std::collections::hash_map::Entry::Occupied(mut e) => {
            *e.get_mut() *= 2;  // Double existing score
        }
        std::collections::hash_map::Entry::Vacant(e) => {
            e.insert(100);      // New player starts with 100
        }
    }
    
    println!("Player scores: {:?}", player_scores);
}
```

### HashMap Iteration and Operations

```rust
use std::collections::HashMap;

fn hashmap_operations() {
    let mut inventory = HashMap::new();
    inventory.insert("apples", 10);
    inventory.insert("bananas", 5);
    inventory.insert("oranges", 8);
    
    // Iterate over key-value pairs
    for (item, quantity) in &inventory {
        println!("{}: {}", item, quantity);
    }
    
    // Iterate over keys only
    for item in inventory.keys() {
        println!("Item: {}", item);
    }
    
    // Iterate over values only
    for quantity in inventory.values() {
        println!("Quantity: {}", quantity);
    }
    
    // Mutable iteration over values
    for quantity in inventory.values_mut() {
        *quantity *= 2;  // Double all quantities
    }
    
    // Remove elements
    if let Some(removed) = inventory.remove("bananas") {
        println!("Removed {} bananas", removed);
    }
    
    // Bulk operations
    let total_items: i32 = inventory.values().sum();
    println!("Total items: {}", total_items);
    
    // Retain only items matching condition
    inventory.retain(|&item, &mut quantity| quantity > 10);
    println!("High quantity items: {:?}", inventory);
}
```

### Custom Keys and Hash Implementation

```rust
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

#[derive(Debug, Eq, PartialEq)]
struct Person {
    name: String,
    age: u32,
}

// Custom hash implementation - only hash by name
impl Hash for Person {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);  // Only hash the name
    }
}

fn custom_keys() {
    let mut people_scores = HashMap::new();
    
    let alice = Person { 
        name: "Alice".to_string(), 
        age: 30 
    };
    let bob = Person { 
        name: "Bob".to_string(), 
        age: 25 
    };
    
    people_scores.insert(alice, 100);
    people_scores.insert(bob, 85);
    
    // Lookup by creating temporary Person
    let lookup_alice = Person { 
        name: "Alice".to_string(), 
        age: 999  // Age doesn't matter for hash/equality
    };
    
    if let Some(score) = people_scores.get(&lookup_alice) {
        println!("Alice's score: {}", score);
    }
    
    println!("People scores: {:?}", people_scores);
}
```

---

## HashSet<T>: Unique Value Collections

### Basic Operations

```rust
use std::collections::HashSet;

fn hashset_basics() {
    // Creation
    let mut set1 = HashSet::new();
    set1.insert("apple");
    set1.insert("banana");
    set1.insert("cherry");
    set1.insert("apple");  // Duplicate - won't be added
    
    // From iterator
    let set2: HashSet<i32> = vec![1, 2, 3, 2, 4, 3, 5].into_iter().collect();
    println!("Unique numbers: {:?}", set2);  // {1, 2, 3, 4, 5}
    
    // Check membership
    if set1.contains("apple") {
        println!("Set contains apple");
    }
    
    // Remove elements
    set1.remove("banana");
    
    // Iteration
    for item in &set1 {
        println!("Item: {}", item);
    }
    
    println!("Fruits: {:?}", set1);
    println!("Set length: {}", set1.len());
}
```

### Set Operations

```rust
use std::collections::HashSet;

fn set_operations() {
    let set_a: HashSet<i32> = vec![1, 2, 3, 4, 5].into_iter().collect();
    let set_b: HashSet<i32> = vec![4, 5, 6, 7, 8].into_iter().collect();
    
    // Union: all elements from both sets
    let union: HashSet<i32> = set_a.union(&set_b).cloned().collect();
    println!("Union: {:?}", union);  // {1, 2, 3, 4, 5, 6, 7, 8}
    
    // Intersection: common elements
    let intersection: HashSet<i32> = set_a.intersection(&set_b).cloned().collect();
    println!("Intersection: {:?}", intersection);  // {4, 5}
    
    // Difference: elements in A but not B
    let difference: HashSet<i32> = set_a.difference(&set_b).cloned().collect();
    println!("Difference (A - B): {:?}", difference);  // {1, 2, 3}
    
    // Symmetric difference: elements in A or B but not both
    let sym_diff: HashSet<i32> = set_a.symmetric_difference(&set_b).cloned().collect();
    println!("Symmetric difference: {:?}", sym_diff);  // {1, 2, 3, 6, 7, 8}
    
    // Subset/superset checks
    let subset: HashSet<i32> = vec![2, 3].into_iter().collect();
    println!("Is {2, 3} subset of A? {}", subset.is_subset(&set_a));
    println!("Is A superset of {2, 3}? {}", set_a.is_superset(&subset));
    
    // Disjoint check
    let disjoint_set: HashSet<i32> = vec![10, 11, 12].into_iter().collect();
    println!("Are A and {10, 11, 12} disjoint? {}", set_a.is_disjoint(&disjoint_set));
}
```

### Practical HashSet Examples

```rust
use std::collections::HashSet;

fn practical_hashset_examples() {
    // Remove duplicates from a vector
    let numbers = vec![1, 2, 2, 3, 3, 3, 4, 4, 4, 4];
    let unique: HashSet<i32> = numbers.into_iter().collect();
    let deduped: Vec<i32> = unique.into_iter().collect();
    println!("Deduplicated: {:?}", deduped);
    
    // Track visited items
    let mut visited_pages = HashSet::new();
    let pages_to_visit = vec![
        "home", "about", "contact", "home", "products", "about"
    ];
    
    for page in pages_to_visit {
        if visited_pages.insert(page) {
            println!("First visit to: {}", page);
        } else {
            println!("Already visited: {}", page);
        }
    }
    
    // Find unique words in text
    let text = "the quick brown fox jumps over the lazy dog";
    let words: HashSet<&str> = text.split_whitespace().collect();
    println!("Unique words: {:?}", words);
    println!("Unique word count: {}", words.len());
}
```

---

## Other Important Collections

### BTreeMap and BTreeSet: Sorted Collections

```rust
use std::collections::{BTreeMap, BTreeSet};

fn sorted_collections() {
    // BTreeMap keeps keys sorted
    let mut scores = BTreeMap::new();
    scores.insert("Charlie", 85);
    scores.insert("Alice", 92);
    scores.insert("Bob", 78);
    
    println!("Sorted scores:");
    for (name, score) in &scores {
        println!("{}: {}", name, score);  // Prints in alphabetical order
    }
    
    // Range queries
    let range: BTreeMap<&str, i32> = scores
        .range("Alice".."Charlie")  // From Alice to Charlie (exclusive)
        .map(|(&k, &v)| (k, v))
        .collect();
    println!("Range A-C: {:?}", range);
    
    // BTreeSet for sorted unique values
    let mut numbers = BTreeSet::new();
    numbers.insert(5);
    numbers.insert(2);
    numbers.insert(8);
    numbers.insert(2);  // Duplicate ignored
    
    println!("Sorted numbers: {:?}", numbers);  // {2, 5, 8}
}
```

### VecDeque: Double-Ended Queue

```rust
use std::collections::VecDeque;

fn vecdeque_example() {
    let mut deque = VecDeque::new();
    
    // Add to both ends efficiently
    deque.push_back(1);     // Add to back: [1]
    deque.push_front(2);    // Add to front: [2, 1]
    deque.push_back(3);     // Add to back: [2, 1, 3]
    
    println!("Deque: {:?}", deque);
    
    // Remove from both ends
    let front = deque.pop_front();  // Some(2), deque: [1, 3]
    let back = deque.pop_back();    // Some(3), deque: [1]
    
    println!("Removed front: {:?}, back: {:?}", front, back);
    println!("Remaining: {:?}", deque);
    
    // Use as a queue (FIFO)
    let mut queue = VecDeque::new();
    queue.push_back("first");
    queue.push_back("second");
    queue.push_back("third");
    
    while let Some(item) = queue.pop_front() {
        println!("Processing: {}", item);
    }
}
```

---

## Collection Performance and When to Use Each

### Performance Guide

```rust
use std::collections::{HashMap, HashSet, BTreeMap, BTreeSet, VecDeque};

fn performance_examples() {
    // Vec<T> - Use when:
    // - You need indexed access
    // - You're iterating in order frequently
    // - Memory usage is critical (most compact)
    let mut data_list = Vec::new();
    for i in 0..1000 {
        data_list.push(i);  // O(1) amortized
    }
    let tenth_item = data_list[9];  // O(1) access
    
    // HashMap<K,V> - Use when:
    // - You need fast key-based lookups
    // - Order doesn't matter
    // - Keys implement Hash + Eq
    let mut user_data = HashMap::new();
    user_data.insert("user123", "Alice");  // O(1) average
    let user = user_data.get("user123");   // O(1) average
    
    // BTreeMap<K,V> - Use when:
    // - You need sorted iteration
    // - You need range queries
    // - Keys implement Ord
    let mut sorted_data = BTreeMap::new();
    sorted_data.insert(10, "ten");
    sorted_data.insert(5, "five");
    // Iteration is in sorted order
    
    // HashSet<T> - Use when:
    // - You need to track unique items
    // - Membership testing is primary operation
    // - Order doesn't matter
    let mut unique_ids = HashSet::new();
    unique_ids.insert(123);
    let is_present = unique_ids.contains(&123);  // O(1) average
    
    // VecDeque<T> - Use when:
    // - You need efficient operations at both ends
    // - Implementing queues or double-ended operations
    let mut buffer = VecDeque::new();
    buffer.push_back("data");   // O(1)
    buffer.push_front("data");  // O(1)
}
```

### Collection Selection Decision Tree

```rust
// Decision helper function
fn choose_collection_advice() {
    println!("Collection Selection Guide:");
    println!("
    Need indexed access (arr[i])? 
    ├─ Yes → Vec<T>
    └─ No
       ├─ Key-value pairs?
       │  ├─ Need sorted keys? → BTreeMap<K,V>
       │  └─ Fast lookup priority? → HashMap<K,V>
       └─ Just unique values?
          ├─ Need sorted values? → BTreeSet<T>
          ├─ Fast membership testing? → HashSet<T>
          └─ Operations at both ends? → VecDeque<T>
    ");
}
```

---

## Common Pitfalls and Best Practices

### Pitfall 1: Inefficient Vec Operations

```rust
fn vec_performance_pitfalls() {
    let mut vec = vec![1, 2, 3, 4, 5];
    
    // ❌ Inefficient: frequent insertions at beginning
    // Each insert shifts all elements
    for i in 0..1000 {
        vec.insert(0, i);  // O(n) operation
    }
    
    // ✅ Better: collect in reverse order or use VecDeque
    let mut efficient_vec = Vec::new();
    for i in 0..1000 {
        efficient_vec.push(i);  // O(1) amortized
    }
    efficient_vec.reverse();    // O(n) but only once
    
    // ✅ Or use VecDeque for frequent front insertions
    let mut deque = std::collections::VecDeque::new();
    for i in 0..1000 {
        deque.push_front(i);  // O(1) operation
    }
}
```

### Pitfall 2: HashMap Key Requirements

```rust
use std::collections::HashMap;

// ❌ This won't work - f64 doesn't implement Eq
// let mut float_map: HashMap<f64, String> = HashMap::new();

// ✅ Use ordered float wrapper or avoid floats as keys
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug)]
struct OrderedFloat(f64);

impl PartialEq for OrderedFloat {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}

impl Eq for OrderedFloat {}

impl Hash for OrderedFloat {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

fn float_key_solution() {
    let mut float_map = HashMap::new();
    float_map.insert(OrderedFloat(3.14), "pi".to_string());
    float_map.insert(OrderedFloat(2.71), "e".to_string());
    
    println!("Float map: {:?}", float_map);
}
```

### Pitfall 3: Borrowing During Iteration

```rust
use std::collections::HashMap;

fn borrowing_pitfalls() {
    let mut scores = HashMap::new();
    scores.insert("Alice", 100);
    scores.insert("Bob", 85);
    
    // ❌ Can't modify while borrowing immutably
    // for (name, score) in &scores {
    //     if *score < 90 {
    //         scores.insert("bonus_" + name, 10);  // Borrow checker error!
    //     }
    // }
    
    // ✅ Solution 1: Collect keys first
    let low_scorers: Vec<String> = scores
        .iter()
        .filter(|(_, &score)| score < 90)
        .map(|(name, _)| format!("bonus_{}", name))
        .collect();
    
    for bonus_name in low_scorers {
        scores.insert(bonus_name, 10);
    }
    
    // ✅ Solution 2: Use entry API when possible
    let name = "Charlie";
    scores.entry(name.to_string()).and_modify(|score| *score += 5).or_insert(80);
    
    println!("Updated scores: {:?}", scores);
}
```

---

## Key Takeaways

1. **Vec<T>** is your default choice for sequences and lists
2. **HashMap<K,V>** for fast key-value lookups when order doesn't matter
3. **HashSet<T>** for unique value collections and membership testing
4. **BTreeMap/BTreeSet** when you need sorted iteration or range queries
5. **VecDeque<T>** for efficient operations at both ends
6. **Consider performance characteristics** when choosing collections
7. **Use iterators** instead of index-based loops for better performance and safety

---

## Exercises

### Exercise 1: Word Frequency Counter

Build a comprehensive text analysis tool:

```rust
use std::collections::HashMap;

struct TextAnalyzer {
    word_counts: HashMap<String, usize>,
    total_words: usize,
}

impl TextAnalyzer {
    fn new() -> Self {
        // Implement
    }
    
    fn add_text(&mut self, text: &str) {
        // Implement: add words from text to counter
        // Handle punctuation and case normalization
    }
    
    fn most_common(&self, n: usize) -> Vec<(String, usize)> {
        // Implement: return n most common words with counts
    }
    
    fn unique_words(&self) -> usize {
        // Implement: return number of unique words
    }
    
    fn word_frequency(&self, word: &str) -> f64 {
        // Implement: return frequency of word as percentage
    }
    
    fn words_starting_with(&self, prefix: &str) -> Vec<String> {
        // Implement: return words starting with prefix
    }
}

fn main() {
    let mut analyzer = TextAnalyzer::new();
    
    analyzer.add_text("The quick brown fox jumps over the lazy dog");
    analyzer.add_text("The dog was really lazy, but the fox was quick");
    
    println!("Most common words: {:?}", analyzer.most_common(3));
    println!("Unique words: {}", analyzer.unique_words());
    println!("Frequency of 'the': {:.2}%", analyzer.word_frequency("the"));
    println!("Words starting with 'th': {:?}", analyzer.words_starting_with("th"));
}
```

### Exercise 2: Set-Based Permission System

Implement a role-based permission system:

```rust
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Permission {
    Read,
    Write,
    Delete,
    Admin,
    Execute,
}

struct User {
    name: String,
    permissions: HashSet<Permission>,
}

struct PermissionManager {
    users: HashMap<String, User>,
    roles: HashMap<String, HashSet<Permission>>,
}

impl PermissionManager {
    fn new() -> Self {
        // Implement
    }
    
    fn create_role(&mut self, role_name: String, permissions: HashSet<Permission>) {
        // Implement: create a role with given permissions
    }
    
    fn add_user(&mut self, username: String) {
        // Implement: add user with no permissions
    }
    
    fn assign_role(&mut self, username: &str, role_name: &str) -> Result<(), String> {
        // Implement: assign role permissions to user
    }
    
    fn grant_permission(&mut self, username: &str, permission: Permission) -> Result<(), String> {
        // Implement: grant specific permission to user
    }
    
    fn revoke_permission(&mut self, username: &str, permission: Permission) -> Result<(), String> {
        // Implement: revoke specific permission from user
    }
    
    fn has_permission(&self, username: &str, permission: &Permission) -> bool {
        // Implement: check if user has specific permission
    }
    
    fn get_user_permissions(&self, username: &str) -> Option<&HashSet<Permission>> {
        // Implement: get all permissions for user
    }
    
    fn users_with_permission(&self, permission: &Permission) -> Vec<String> {
        // Implement: get all users with specific permission
    }
}

fn main() {
    let mut pm = PermissionManager::new();
    
    // Create roles
    pm.create_role("admin".to_string(), 
                   [Permission::Read, Permission::Write, Permission::Delete, Permission::Admin]
                   .into_iter().collect());
    
    pm.create_role("editor".to_string(),
                   [Permission::Read, Permission::Write].into_iter().collect());
    
    // Add users
    pm.add_user("alice".to_string());
    pm.add_user("bob".to_string());
    
    // Assign roles
    pm.assign_role("alice", "admin").unwrap();
    pm.assign_role("bob", "editor").unwrap();
    
    // Grant additional permission
    pm.grant_permission("bob", Permission::Execute).unwrap();
    
    // Test permissions
    println!("Alice has admin? {}", pm.has_permission("alice", &Permission::Admin));
    println!("Bob has admin? {}", pm.has_permission("bob", &Permission::Admin));
    println!("Bob's permissions: {:?}", pm.get_user_permissions("bob"));
    println!("Users with write permission: {:?}", pm.users_with_permission(&Permission::Write));
}
```

### Exercise 3: Multi-Collection Data Structure

Build a library catalog that efficiently supports multiple types of queries:

```rust
use std::collections::{HashMap, HashSet, BTreeMap, VecDeque};

#[derive(Debug, Clone)]
struct Book {
    id: u32,
    title: String,
    author: String,
    year: u16,
    genres: HashSet<String>,
    available: bool,
}

struct LibraryCatalog {
    books: HashMap<u32, Book>,                    // Fast lookup by ID
    by_author: HashMap<String, Vec<u32>>,         // Books by author
    by_year: BTreeMap<u16, Vec<u32>>,            // Books by year (sorted)
    by_genre: HashMap<String, HashSet<u32>>,      // Books by genre
    recently_added: VecDeque<u32>,                // Recently added books
    available_books: HashSet<u32>,                // Currently available
}

impl LibraryCatalog {
    fn new() -> Self {
        // Implement
    }
    
    fn add_book(&mut self, book: Book) {
        // Implement: add book to all appropriate collections
    }
    
    fn remove_book(&mut self, book_id: u32) -> Option<Book> {
        // Implement: remove book from all collections
    }
    
    fn find_by_id(&self, id: u32) -> Option<&Book> {
        // Implement
    }
    
    fn find_by_author(&self, author: &str) -> Vec<&Book> {
        // Implement: return all books by author
    }
    
    fn find_by_genre(&self, genre: &str) -> Vec<&Book> {
        // Implement: return all books in genre
    }
    
    fn find_by_year_range(&self, start_year: u16, end_year: u16) -> Vec<&Book> {
        // Implement: return books published in year range
    }
    
    fn recently_added(&self, count: usize) -> Vec<&Book> {
        // Implement: return most recently added books
    }
    
    fn available_in_genre(&self, genre: &str) -> Vec<&Book> {
        // Implement: return available books in genre
    }
    
    fn checkout_book(&mut self, book_id: u32) -> Result<(), String> {
        // Implement: mark book as unavailable
    }
    
    fn return_book(&mut self, book_id: u32) -> Result<(), String> {
        // Implement: mark book as available
    }
    
    fn statistics(&self) -> (usize, usize, Vec<String>) {
        // Implement: return (total_books, available_books, top_genres)
    }
}

fn main() {
    let mut catalog = LibraryCatalog::new();
    
    // Add some books
    catalog.add_book(Book {
        id: 1,
        title: "The Rust Programming Language".to_string(),
        author: "Steve Klabnik".to_string(),
        year: 2018,
        genres: ["Programming", "Technology"].iter().map(|s| s.to_string()).collect(),
        available: true,
    });
    
    catalog.add_book(Book {
        id: 2,
        title: "Dune".to_string(),
        author: "Frank Herbert".to_string(),
        year: 1965,
        genres: ["Science Fiction", "Adventure"].iter().map(|s| s.to_string()).collect(),
        available: true,
    });
    
    // Test various queries
    println!("Books by Frank Herbert: {:?}", 
             catalog.find_by_author("Frank Herbert").len());
    println!("Programming books: {:?}", 
             catalog.find_by_genre("Programming").len());
    println!("Books from 1960-1970: {:?}", 
             catalog.find_by_year_range(1960, 1970).len());
    
    catalog.checkout_book(1).unwrap();
    println!("Available programming books after checkout: {:?}", 
             catalog.available_in_genre("Programming").len());
    
    let (total, available, top_genres) = catalog.statistics();
    println!("Stats - Total: {}, Available: {}, Top genres: {:?}", 
             total, available, top_genres);
}
```

**Next Up:** In Chapter 7, we'll explore traits - Rust's powerful system for defining shared behavior and enabling polymorphism without inheritance.
