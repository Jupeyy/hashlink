use core::{borrow::Borrow, hash::Hash};

use super::{LinkedHashMap, Node, OptNonNullExt, RawEntryMut};

pub struct LinkedHashMapExceptView<'a, K, V, S> {
    hash_map: &'a mut LinkedHashMap<K, V, S>,
    ignore_key: K,
    ignore_node: *mut Node<K, V>,
}

impl<'a, K: Eq + Hash, V, S> LinkedHashMapExceptView<'a, K, V, S> {
    #[inline]
    pub(crate) fn new(
        hash_map: &'a mut LinkedHashMap<K, V, S>,
        ignore_key: K,
        ignore_node: *mut Node<K, V>,
    ) -> Self {
        Self {
            hash_map,
            ignore_key,
            ignore_node,
        }
    }
}

impl<K: Copy + Clone + Eq + Hash, V, S: core::hash::BuildHasher>
    LinkedHashMapExceptView<'_, K, V, S>
{
    #[inline]
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        if self.ignore_key.borrow().eq(key) {
            None
        } else {
            self.hash_map.get(key)
        }
    }

    #[inline]
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        if self.ignore_key.borrow().eq(key) {
            None
        } else {
            self.hash_map.get_mut(key)
        }
    }

    #[inline]
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = (&K, &V)> {
        self.hash_map
            .iter_nodes()
            .filter(move |node| node.as_ptr() != self.ignore_node)
            .map(|node| {
                let (k, v) = unsafe { (*node.as_ptr()).entry_ref() };
                (k, v)
            })
    }

    #[inline]
    pub fn iter_mut(&mut self) -> impl DoubleEndedIterator<Item = (&K, &mut V)> {
        let ignore_key = self.ignore_node;
        self.hash_map
            .iter_nodes()
            .filter(move |node| node.as_ptr() != ignore_key)
            .map(|node| {
                let (k, v) = unsafe { (*node.as_ptr()).entry_mut() };
                (&*k, v)
            })
    }
}

pub struct LinkedHashMapIterExt<'a, K, V, S> {
    hash_map: &'a mut LinkedHashMap<K, V, S>,
    rev: bool,
}

impl<'a, K, V, S> LinkedHashMapIterExt<'a, K, V, S> {
    #[inline]
    pub fn new(hash_map: &'a mut LinkedHashMap<K, V, S>) -> Self {
        Self {
            hash_map,
            rev: false,
        }
    }
}

impl<'a, K: Eq + Hash + Copy + Clone + 'static, V: 'static, S: core::hash::BuildHasher>
    LinkedHashMapIterExt<'a, K, V, S>
{
    #[inline]
    pub fn rev(self) -> Self {
        Self {
            hash_map: self.hash_map,
            rev: !self.rev,
        }
    }

    #[inline]
    pub fn for_each<F>(&'a mut self, mut f: F)
    where
        for<'b> F: FnMut((&'b K, (&'b mut V, LinkedHashMapExceptView<'b, K, V, S>))),
    {
        let it = self.hash_map.iter_nodes();
        if self.rev {
            let it = it.rev();
            for node in it {
                let node_ptr = node.as_ptr();
                let (k, v) = unsafe { (*node_ptr).entry_mut() };
                let k = *k;
                f((
                    &k,
                    (v, LinkedHashMapExceptView::new(self.hash_map, k, node_ptr)),
                ));
            }
        } else {
            for node in it {
                let node_ptr = node.as_ptr();
                let (k, v) = unsafe { (*node_ptr).entry_mut() };
                let k = *k;
                f((
                    &k,
                    (v, LinkedHashMapExceptView::new(self.hash_map, k, node_ptr)),
                ));
            }
        }
    }
}

pub struct LinkedHashMapEntryAndRes {}

impl LinkedHashMapEntryAndRes {
    #[inline]
    pub fn get<
        'a,
        K: Eq + Hash + Copy + Clone + 'static,
        V: 'static,
        S: core::hash::BuildHasher,
    >(
        hash_map: &'a mut LinkedHashMap<K, V, S>,
        key: &K,
    ) -> (&'a mut V, LinkedHashMapExceptView<'a, K, V, S>) {
        let RawEntryMut::Occupied(entry) = hash_map.raw_entry_mut().from_key(key) else {
            panic!("No value found for given key");
        };

        let node = entry.entry.get();
        let node_ptr = node.as_ptr();
        let res = &mut unsafe { (*node_ptr).entry_mut() }.1;

        (res, LinkedHashMapExceptView::new(hash_map, *key, node_ptr))
    }
}
