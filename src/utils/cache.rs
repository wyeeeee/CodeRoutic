use std::collections::HashMap;
use std::hash::Hash;

pub struct LRUCache<K, V> {
    capacity: usize,
    cache: HashMap<K, V>,
    // 使用一个双向链表来维护访问顺序
    // 这里简化实现，使用 HashMap 和访问时间戳
}

#[derive(Debug, Clone)]
pub struct Usage {
    pub input_tokens: usize,
    pub output_tokens: usize,
}

impl<K, V> LRUCache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            cache: HashMap::new(),
        }
    }
    
    pub fn get(&self, key: &K) -> Option<&V> {
        self.cache.get(key)
    }
    
    pub fn put(&mut self, key: K, value: V) {
        if self.cache.len() >= self.capacity && !self.cache.contains_key(&key) {
            // 如果缓存已满，移除第一个元素（简化实现）
            if let Some(first_key) = self.cache.keys().next().cloned() {
                self.cache.remove(&first_key);
            }
        }
        self.cache.insert(key, value);
    }
    
    pub fn values(&self) -> Vec<&V> {
        self.cache.values().collect()
    }
}

pub static mut SESSION_USAGE_CACHE: Option<LRUCache<String, Usage>> = None;