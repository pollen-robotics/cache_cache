//! This caching library has been designed for specific use-cases where:
//!
//! * getting a "fresh" value can be time consuming and can fail (eg. IOs with hardware)
//! * getting multiple values at once can be more efficient than getting each value independantly.
//!
//! Typically, its primary use was to retrieve position/speed/temperature/etc from multiple motors using serial communication. In this setup, the motors are daisy chained, and in the protocol used to communicate with them, a specific message can be used to retrieve a register value for multiple motors at once.
//!

use std::{
    borrow::Borrow,
    collections::HashMap,
    hash::Hash,
    ops::Index,
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

    /// Gets the given key’s corresponding entry in the cache for in-place manipulation.
    ///
    /// Examples
    /// ```
    /// use cache_cache::Cache;
    /// use std::time::Duration;
    ///
    /// let mut motors_temperature = Cache::with_expiry_duration(Duration::from_millis(100));
    ///
    /// fn get_motor_temperature(motor_id: u8) -> f64 {
    ///     // Should actually retrieve the real value from the motor
    ///     42.0
    /// }
    ///
    /// let motor_id = 11;
    /// let temp = motors_temperature.entry(motor_id).or_insert_with(|| get_motor_temperature(motor_id));
    /// assert_eq!(motors_temperature.get(&motor_id), Some(&42.0));
    pub fn entry(&mut self, key: K) -> Entry<'_, K, V> {
        if self.get(&key).is_some() {
            let v = self.get_mut(&key).unwrap();
            Entry::Occupied(OccupiedEntry { k: key, v })
        } else {
            Entry::Vacant(VacantEntry {
                k: key,
                cache: self,
            })
        }
    }
    fn has_expired(expiry_duration: Option<Duration>, t: &SystemTime) -> bool {
        match expiry_duration {
            Some(expiry) => t.elapsed().unwrap() > expiry,
            None => false,
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
            Some((v, t)) => match Self::has_expired(self.expiry_duration, t) {
                true => None,
                false => Some(v),
            },
            None => None,
        }
    }
    /// Returns a mutable reference to the value corresponding to the key if it has not expired.
    ///
    /// # Examples
    ///
    /// ```
    /// use cache_cache::Cache;
    ///
    /// let mut cache: Cache<&str, f64> = Cache::keep_last();
    /// cache.insert("target_position", 90.0);
    ///
    /// if let Some(pos) = cache.get_mut("target_position") {
    ///     *pos += 10.0;
    /// }
    /// assert_eq!(cache.get(&"target_position"), Some(&100.0));
    /// ```
    pub fn get_mut<Q: ?Sized>(&mut self, k: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        match self.hash_map.get_mut(k) {
            Some((v, t)) => match Self::has_expired(self.expiry_duration, t) {
                true => None,
                false => Some(v),
            },
            None => None,
        }
    }
    /// Inserts a key-value pair into the cache.
    ///
    /// If the cache did not have this key present, None is returned.
    /// If the cache did have this key present, the value is updated, and the old value (expired or not) is returned.
    ///
    /// Examples
    /// ```
    /// use cache_cache::Cache;
    ///
    /// let mut cache = Cache::keep_last();
    /// assert_eq!(cache.insert(10, "a"), None);
    /// assert_eq!(cache.insert(10, "b"), Some("a"));
    /// assert_eq!(cache[&10], "b");
    /// ```
    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        self.hash_map.insert(k, (v, SystemTime::now())).map(|v| v.0)
    }
}

impl<K, Q: ?Sized, V> Index<&Q> for Cache<K, V>
where
    K: Eq + Hash + Borrow<Q>,
    Q: Eq + Hash,
{
    type Output = V;

    /// Returns a reference to the value corresponding to the supplied key.
    ///
    /// # Panics
    ///
    /// Panics if the key is not present in the `Cache`.
    fn index(&self, index: &Q) -> &Self::Output {
        self.get(index).expect("no entry found for key")
    }
}

/// A view into a single entry in a cache, which may either be vacant or occupied.
///
/// This enum is constructed from the entry method on [Cache].
pub enum Entry<'a, K: 'a, V: 'a> {
    /// An occupied entry.
    Occupied(OccupiedEntry<'a, K, V>),
    /// A vacant entry.
    Vacant(VacantEntry<'a, K, V>),
}

impl<'a, K, V> Entry<'a, K, V>
where
    K: Hash + Eq + Clone,
{
    /// Ensures a value is in the entry by inserting the default if empty, and returns a mutable reference to the value in the entry.
    ///
    /// Examples
    /// ```
    /// use cache_cache::Cache;
    ///
    /// let mut target_positions = Cache::keep_last();
    ///
    /// target_positions.entry(10).or_insert(0);
    /// assert_eq!(target_positions[&10], 0);
    ///
    /// *target_positions.entry(10).or_insert(10) += 20;
    /// assert_eq!(target_positions[&10], 20);
    pub fn or_insert(self, default: V) -> &'a mut V {
        match self {
            Entry::Occupied(entry) => entry.v,
            Entry::Vacant(entry) => entry.insert(default),
        }
    }
    /// Ensures a value is in the entry by inserting the result of the default function if empty, and returns a mutable reference to the value in the entry.
    ///
    /// Examples
    /// ```
    /// use cache_cache::Cache;
    ///
    /// let mut torque_enable: Cache<u8, bool> = Cache::keep_last();
    ///
    /// torque_enable.entry(20).or_insert_with(|| false);
    /// assert_eq!(torque_enable[&20], false);
    /// ```
    pub fn or_insert_with<F: FnOnce() -> V>(self, default: F) -> &'a mut V {
        match self {
            Entry::Occupied(entry) => entry.v,
            Entry::Vacant(entry) => entry.insert(default()),
        }
    }

    /// Returns a reference to this entry’s key.
    ///
    /// Examples
    /// ```
    /// use cache_cache::Cache;
    ///
    /// let mut cache: Cache<&str, u32> = Cache::keep_last();
    /// assert_eq!(cache.entry("speed").key(), &"speed");
    /// ```
    pub fn key(&self) -> &K {
        match self {
            Entry::Occupied(entry) => &entry.k,
            Entry::Vacant(entry) => &entry.k,
        }
    }
}

/// A view into an occupied entry in a [Cache]. It is part of the [Entry] enum.
pub struct OccupiedEntry<'a, K: 'a, V: 'a> {
    k: K,
    v: &'a mut V,
}

/// A view into a vacant entry in a [Cache]. It is part of the [Entry] enum.
pub struct VacantEntry<'a, K, V> {
    k: K,
    cache: &'a mut Cache<K, V>,
}

impl<'a, K, V> VacantEntry<'a, K, V>
where
    K: Hash + Eq + Clone,
{
    fn insert(self, v: V) -> &'a mut V {
        self.cache.insert(self.k.clone(), v);
        self.cache.get_mut(&self.k).unwrap()
    }
}
