use smallvec::SmallVec;

/// Inline table for storing key-value pairs.
///
/// Faster than `HashMap` for smaller number of elements.
#[derive(Debug, Clone)]
pub struct SmallMap<K, V, const N: usize = 16> {
    inner: SmallVec<[(K, V); N]>,
}

impl<K, V, const N: usize> Default for SmallMap<K, V, N> {
    fn default() -> Self {
        Self {
            inner: SmallVec::new(),
        }
    }
}

impl<K: Eq, V, const N: usize> SmallMap<K, V, N> {
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
