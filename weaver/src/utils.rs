/// Inline table for storing key-value pairs.
pub struct SmallMap<K, V> {
    inner: Vec<(K, V)>,
}

impl<K, V> Default for SmallMap<K, V> {
    fn default() -> Self {
        Self { inner: Vec::new() }
    }
}

impl<K: Eq, V> SmallMap<K, V> {
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let previous = self
            .inner
            .iter()
            .position(|(k, _)| *k == key)
            .map(|i| self.inner.remove(i));
        self.inner.push((key, value));
        previous.map(|(_, value)| value)
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.inner
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, value)| value)
    }
}
