use core::{fmt, marker::PhantomData, ptr::NonNull};

use super::{Iter, Node, OptNonNullExt};

pub(crate) struct IterNodes<K, V> {
    pub(crate) head: Option<NonNull<Node<K, V>>>,
    pub(crate) tail: Option<NonNull<Node<K, V>>>,
    pub(crate) remaining: usize,
}

impl<K, V> IterNodes<K, V> {
    #[inline]
    pub(crate) fn iter(&self) -> Iter<'_, K, V> {
        Iter {
            head: self.head.as_ptr(),
            tail: self.tail.as_ptr(),
            remaining: self.remaining,
            marker: PhantomData,
        }
    }
}

unsafe impl<K, V> Send for IterNodes<K, V>
where
    K: Send,
    V: Send,
{
}

unsafe impl<K, V> Sync for IterNodes<K, V>
where
    K: Sync,
    V: Sync,
{
}

impl<K, V> fmt::Debug for IterNodes<K, V>
where
    K: fmt::Debug,
    V: fmt::Debug,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<K, V> Iterator for IterNodes<K, V> {
    type Item = Option<NonNull<Node<K, V>>>;

    #[inline]
    fn next(&mut self) -> Option<Option<NonNull<Node<K, V>>>> {
        if self.remaining == 0 {
            None
        } else {
            self.remaining -= 1;
            unsafe {
                let head = self.head;
                self.head = Some((*head.as_ptr()).links.value.next);
                Some(head)
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<K, V> DoubleEndedIterator for IterNodes<K, V> {
    #[inline]
    fn next_back(&mut self) -> Option<Option<NonNull<Node<K, V>>>> {
        if self.remaining == 0 {
            None
        } else {
            self.remaining -= 1;
            unsafe {
                let tail = self.tail;
                self.tail = Some((*tail.as_ptr()).links.value.prev);
                Some(tail)
            }
        }
    }
}

impl<K, V> ExactSizeIterator for IterNodes<K, V> {}
