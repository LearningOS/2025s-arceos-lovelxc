extern crate alloc;
use alloc::vec::Vec;
use axhal::misc::random;
use core::borrow::Borrow;
use core::clone::Clone;
use core::hash::{Hash, Hasher};
use core::mem;
// Gen by Claude, prompt: "用拉链法实现一个极简的HashMap(no_std), 扩容系数设置为1.5"
const DEFAULT_CAPACITY: usize = 16;
const LOAD_FACTOR: f32 = 0.75;
const RESIZE_FACTOR: f32 = 1.5;

// 简单的哈希器实现
struct SimpleHasher {
    state: u64,
}

impl SimpleHasher {
    // 就很扯淡，但不管了
    fn new() -> Self {
        Self {
            state: random() as u64,
        }
    }
}

impl Hasher for SimpleHasher {
    fn finish(&self) -> u64 {
        self.state
    }

    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.state = self.state.wrapping_mul(31).wrapping_add(b as u64);
        }
    }
}

pub struct HashMap<K, V> {
    buckets: Vec<Vec<(K, V)>>,
    len: usize,
    capacity: usize,
}

impl<K, V> HashMap<K, V>
where
    K: Hash + Eq,
{
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_CAPACITY)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let mut buckets = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            buckets.push(Vec::new());
        }

        Self {
            buckets,
            len: 0,
            capacity,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn hash<Q: Hash + ?Sized>(&self, key: &Q) -> usize {
        let mut hasher = SimpleHasher::new();
        key.hash(&mut hasher);
        hasher.finish() as usize % self.capacity
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V>
    where
        K: Clone,
        V: Clone,
    {
        if (self.len as f32) >= (self.capacity as f32 * LOAD_FACTOR) {
            self.resize();
        }

        let hash = self.hash(&key);
        let bucket = &mut self.buckets[hash];

        // 查找键是否已存在
        for i in 0..bucket.len() {
            if bucket[i].0 == key {
                // 键已存在，更新值
                return Some(mem::replace(&mut bucket[i].1, value));
            }
        }

        // 键不存在，添加新键值对
        bucket.push((key, value));
        self.len += 1;
        None
    }

    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let hash = self.hash(key);
        let bucket = &self.buckets[hash];

        for pair in bucket {
            if pair.0.borrow() == key {
                return Some(&pair.1);
            }
        }
        None
    }

    pub fn get_mut<Q: ?Sized>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let hash = self.hash(key);
        let bucket = &mut self.buckets[hash];

        for pair in bucket.iter_mut() {
            if pair.0.borrow() == key {
                return Some(&mut pair.1);
            }
        }
        None
    }

    pub fn remove<Q: ?Sized>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let hash = self.hash(key);
        let bucket = &mut self.buckets[hash];

        let mut index = None;
        for (i, pair) in bucket.iter().enumerate() {
            if pair.0.borrow() == key {
                index = Some(i);
                break;
            }
        }

        if let Some(idx) = index {
            let (_, value) = bucket.remove(idx);
            self.len -= 1;
            Some(value)
        } else {
            None
        }
    }

    pub fn contains_key<Q: ?Sized>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.get(key).is_some()
    }

    fn resize(&mut self)
    where
        K: Clone,
        V: Clone,
    {
        let new_capacity = (self.capacity as f32 * RESIZE_FACTOR) as usize;
        let mut new_buckets = Vec::with_capacity(new_capacity);
        for _ in 0..new_capacity {
            new_buckets.push(Vec::new());
        }

        // 移动旧数据到新桶
        for bucket in &self.buckets {
            for (key, value) in bucket {
                let mut hasher = SimpleHasher::new();
                key.hash(&mut hasher);
                let hash = hasher.finish() as usize % new_capacity;
                new_buckets[hash].push((key.clone(), value.clone()));
            }
        }

        self.buckets = new_buckets;
        self.capacity = new_capacity;
    }
    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter {
            buckets: &self.buckets,
            bucket_index: 0,
            element_index: 0,
        }
    }
}

impl<K, V> Default for HashMap<K, V>
where
    K: Hash + Eq,
{
    fn default() -> Self {
        Self::new()
    }
}
pub struct Iter<'a, K, V> {
    buckets: &'a Vec<Vec<(K, V)>>,
    bucket_index: usize,
    element_index: usize,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        // 遍历所有桶
        while self.bucket_index < self.buckets.len() {
            let bucket = &self.buckets[self.bucket_index];

            // 检查当前桶中是否还有元素
            if self.element_index < bucket.len() {
                // 获取当前元素并前进到下一个位置
                let (k, v) = &bucket[self.element_index];
                self.element_index += 1;
                return Some((k, v));
            }

            // 当前桶已遍历完，移至下一个桶
            self.bucket_index += 1;
            self.element_index = 0;
        }

        // 所有桶都遍历完了
        None
    }
}
