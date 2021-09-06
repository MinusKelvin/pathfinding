use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::hash::{BuildHasher, Hash};

use bumpalo::Bump;

use crate::util::{GridDomain, IndexDomain};
use crate::{Cell, Owner, SearchNode};

use super::NodePool;

pub struct HashPool<V, S = RandomState> {
    map: Cell<HashMap<V, *const Cell<SearchNode<V>>, S>>,
    arena: Bump,
}

// require V: Copy so that we don't have any drop glue, otherwise we might leak memory.
impl<V: Hash + Eq + Copy, S: BuildHasher> NodePool<V> for HashPool<V, S> {
    fn reset(&mut self, owner: &mut Owner) {
        owner.rw(&self.map).clear();
        self.arena.reset();
    }

    fn generate(&self, id: V, owner: &mut Owner) -> &Cell<SearchNode<V>> {
        let map = owner.rw(&self.map);
        let &mut node_ptr = map.entry(id).or_insert_with(|| {
            self.arena.alloc(Cell::new(SearchNode {
                search_num: 0,
                pqueue_location: 0,
                expansions: 0,
                id,
                parent: None,
                g: f64::INFINITY,
                lb: f64::INFINITY,
            }))
        });
        unsafe {
            // SAFETY: The pointer points into our arena. The pointer can only dangle if the arena
            //         is reset, which requires a mutable reference to self. This means that, since
            //         we return data living as long as &self, any references must be gone when this
            //         happens. Additionally, the pointers in this map cannot be stale as the map is
            //         emptied whenever the arena is reset.
            &*node_ptr
        }
    }
}

impl<V, S> HashPool<V, S> {
    pub fn with_hasher(hash_builder: S) -> Self {
        HashPool {
            map: Cell::new(HashMap::with_hasher(hash_builder)),
            arena: Bump::new(),
        }
    }

    pub fn with_capacity_and_hasher(capacity: usize, hash_builder: S) -> Self {
        HashPool {
            map: Cell::new(HashMap::with_capacity_and_hasher(capacity, hash_builder)),
            arena: Bump::with_capacity(capacity * std::mem::size_of::<Cell<SearchNode<V>>>()),
        }
    }
}

impl<V> HashPool<V, RandomState> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_and_hasher(capacity, Default::default())
    }
}

impl<V, S: Default> Default for HashPool<V, S> {
    fn default() -> Self {
        HashPool {
            map: Cell::new(HashMap::default()),
            arena: Bump::new(),
        }
    }
}

// SAFETY: the pointers we hold that prevent this type from auto-implementing Send are basically
//         just self-references, so we don't care what thread we're on.
unsafe impl<V: Send, S: Send> Send for HashPool<V, S> {}

// SAFETY: all ids are in-bounds, so obviously the required invariant holds.
unsafe impl<S> GridDomain for HashPool<(i32, i32), S> {
    fn width(&self) -> i32 {
        i32::MAX
    }

    fn height(&self) -> i32 {
        i32::MAX
    }
}

// SAFETY: all ids are in-bounds, so obviously the required invariant holds.
unsafe impl<S> IndexDomain for HashPool<usize, S> {
    fn len(&self) -> usize {
        usize::MAX
    }
}
