use std::{
    borrow::Borrow,
    collections::HashMap,
    hash::Hash,
    time::{Duration, SystemTime},
};

/// Cache implementation with a focus on expiry duration and reducing IO calls.
///
/// It is based on the [HashMap] APIs, so it can be used in almost the same way.
///
/// # Examples
///
/// ```
/// use cache_cache::Cache;
/// use std::{thread, time::Duration};
///
/// // Create a new Cache with 10ms expiry duration.
/// let mut c = Cache::with_expiry_duration(Duration::from_millis(10));
///
/// // Insert a new value in the cache
/// c.insert("present_temperature", 27.0);
///
/// // Retrieve it
/// assert_eq!(c.get(&"present_temperature"), Some(&27.0));
///
/// // Wait for the value to get expired
/// thread::sleep(Duration::from_millis(20));
/// assert_eq!(c.get(&"present_temperature"), None);
/// ```

pub struct Cache<K, V> {
    hash_map: HashMap<K, (V, SystemTime)>,
    expiry_duration: Option<Duration>,
}

impl<K, V> Cache<K, V>
where
    K: Hash + Eq,
{
    /// Creates an empty Cache where the last inserted value is kept.
    ///
    /// # Examples
    ///
    /// ```
    /// use cache_cache::Cache;
    /// let mut cache: Cache<&str, i32> = Cache::keep_last();
    /// ```
    pub fn keep_last() -> Self {
        Cache {
            hash_map: HashMap::new(),
            expiry_duration: None,
        }
    }
    /// Creates an empty Cache with an expiry duration.
    ///
    /// Each inserted value is kept until its expiration duration is reached.
    ///
    /// # Examples
    ///
    /// ```
    /// use cache_cache::Cache;
    /// use std::time::Duration;
    ///
    /// let mut cache: Cache<&str, i32> = Cache::with_expiry_duration(Duration::from_millis(10));
    /// ```
    pub fn with_expiry_duration(duration: Duration) -> Self {
        Cache {
            hash_map: HashMap::new(),
            expiry_duration: Some(duration),
        }
    }

    /// Returns a reference to the value corresponding to the key if it has not expired.
    ///
    /// # Examples
    ///
    /// ```
    /// use cache_cache::Cache;
    ///
    /// let mut cache: Cache<&str, f64> = Cache::keep_last();
    /// cache.insert("position", 0.23);
    /// assert_eq!(cache.get(&"position"), Some(&0.23));
    /// ```
    pub fn get<Q: ?Sized>(&self, k: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        match self.hash_map.get(k) {
            Some((v, t)) => match self.expiry_duration {
                Some(expiry) => {
                    if t.elapsed().unwrap() < expiry {
                        Some(v)
                    } else {
                        None
                    }
                }
                None => Some(v),
            },
            None => None,
        }
    }
    /// Inserts a key-value pair into the cache.
    ///
    /// If the cache did not have this key present, None is returned.
    /// If the cache did have this key present, the value is updated, and the old value (expired or not) is returned.
    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        self.hash_map.insert(k, (v, SystemTime::now())).map(|v| v.0)
    }
}
