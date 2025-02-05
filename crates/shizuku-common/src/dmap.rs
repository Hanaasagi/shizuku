#![allow(clippy::disallowed_types)]

use std::collections::{HashMap, HashSet};

/// Use to make `HashMaps` and `HashSets` across the compiler deterministic.
pub type DHashMap<K, V> = HashMap<K, V, DeterministicState>;

pub type DHashSet<T> = HashSet<T, DeterministicState>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct DeterministicState;
impl std::hash::BuildHasher for DeterministicState {
    type Hasher = std::collections::hash_map::DefaultHasher;

    fn build_hasher(&self) -> Self::Hasher {
        std::collections::hash_map::DefaultHasher::new()
    }
}

pub fn new<K, V>() -> DHashMap<K, V> {
    DHashMap::with_hasher(DeterministicState)
}
pub fn with_capacity<K, V>(capacity: usize) -> DHashMap<K, V> {
    DHashMap::with_capacity_and_hasher(capacity, DeterministicState)
}

pub fn new_set<T>() -> DHashSet<T> {
    DHashSet::with_hasher(DeterministicState)
}

pub fn set_with_capacity<T>(capacity: usize) -> DHashSet<T> {
    DHashSet::with_capacity_and_hasher(capacity, DeterministicState)
}

#[test]
fn test_deterministic_hashmap() {
    let mut map1 = new::<&str, i32>();
    map1.insert("a", 1);
    map1.insert("b", 2);
    map1.insert("c", 3);

    let mut map2 = new::<&str, i32>();
    map2.insert("a", 1);
    map2.insert("b", 2);
    map2.insert("c", 3);

    // Verify same iteration order
    assert_eq!(
        map1.iter().collect::<Vec<_>>(),
        map2.iter().collect::<Vec<_>>()
    );

    // Verify capacity constructor
    let map3 = with_capacity::<&str, i32>(100);
    assert!(map3.capacity() >= 100);
}

#[test]
fn test_deterministic_hashset() {
    let mut set1 = new_set::<&str>();
    set1.insert("x");
    set1.insert("y");
    set1.insert("z");

    let mut set2 = new_set::<&str>();
    set2.insert("x");
    set2.insert("y");
    set2.insert("z");

    // Verify same iteration order
    assert_eq!(
        set1.iter().collect::<Vec<_>>(),
        set2.iter().collect::<Vec<_>>()
    );

    // Verify capacity constructor
    let set3 = set_with_capacity::<&str>(50);
    assert!(set3.capacity() >= 50);
}
