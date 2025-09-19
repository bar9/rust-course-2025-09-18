// Chapter 8: Generics Solutions

use std::collections::VecDeque;
use std::marker::PhantomData;

// ==========================
// Exercise 1: Generic Queue
// ==========================

pub struct Queue<T> {
    items: VecDeque<T>,
}

impl<T> Queue<T> {
    pub fn new() -> Self {
        Queue {
            items: VecDeque::new(),
        }
    }
    
    pub fn enqueue(&mut self, item: T) {
        self.items.push_back(item);
    }
    
    pub fn dequeue(&mut self) -> Option<T> {
        self.items.pop_front()
    }
    
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
    
    pub fn len(&self) -> usize {
        self.items.len()
    }
}

// ==========================
// Exercise 2: Generic Min Function
// ==========================

pub fn min<T>(a: T, b: T) -> T 
where
    T: PartialOrd,
{
    if a < b { a } else { b }
}

// ==========================
// Exercise 3: Builder Pattern with Phantom Types
// ==========================

pub struct NoHeaders;
pub struct WithHeaders;

pub struct RequestBuilder<State> {
    url: String,
    headers: Vec<(String, String)>,
    _state: PhantomData<State>,
}

impl RequestBuilder<NoHeaders> {
    pub fn new(url: String) -> Self {
        RequestBuilder {
            url,
            headers: Vec::new(),
            _state: PhantomData,
        }
    }
    
    pub fn add_header(mut self, key: String, value: String) -> RequestBuilder<WithHeaders> {
        self.headers.push((key, value));
        RequestBuilder {
            url: self.url,
            headers: self.headers,
            _state: PhantomData,
        }
    }
}

impl RequestBuilder<WithHeaders> {
    pub fn add_header(mut self, key: String, value: String) -> Self {
        self.headers.push((key, value));
        self
    }
    
    pub fn send(self) -> Request {
        Request {
            url: self.url,
            headers: self.headers,
        }
    }
}

pub struct Request {
    pub url: String,
    pub headers: Vec<(String, String)>,
}

impl Request {
    pub fn execute(&self) -> String {
        format!("Sent request to {} with {} headers", self.url, self.headers.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_queue() {
        let mut queue = Queue::new();
        queue.enqueue(1);
        queue.enqueue(2);
        queue.enqueue(3);
        
        assert_eq!(queue.dequeue(), Some(1));
        assert_eq!(queue.dequeue(), Some(2));
        assert_eq!(queue.dequeue(), Some(3));
        assert_eq!(queue.dequeue(), None);
    }
    
    #[test]
    fn test_min() {
        assert_eq!(min(5, 3), 3);
        assert_eq!(min("apple", "banana"), "apple");
        assert_eq!(min(3.14, 2.71), 2.71);
    }
    
    #[test]
    fn test_builder() {
        let request = RequestBuilder::new("https://api.example.com".to_string())
            .add_header("Content-Type".to_string(), "application/json".to_string())
            .add_header("Authorization".to_string(), "Bearer token".to_string())
            .send();
        
        assert_eq!(request.url, "https://api.example.com");
        assert_eq!(request.headers.len(), 2);
    }
}