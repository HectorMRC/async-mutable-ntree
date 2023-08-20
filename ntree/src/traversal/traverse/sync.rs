//! Synchronous traversal implementation.

use crate::{
    traversal::{macros, Traverse},
    Asynchronous, Node, Synchronous, TraverseOwned,
};
use std::marker::PhantomData;

impl<'a, T> From<Traverse<'a, T, Asynchronous>> for Traverse<'a, T, Synchronous> {
    fn from(value: Traverse<'a, T, Asynchronous>) -> Self {
        Traverse::new(value.node)
    }
}

impl<'a, T> Traverse<'a, T, Synchronous>
where
    T: Sync + Send,
{
    /// Converts the synchronous traverse into an asynchronous one.
    pub fn into_async(self) -> Traverse<'a, T, Asynchronous> {
        Traverse::<'a, T, Asynchronous>::from(self)
    }
}

impl<'a, T> Traverse<'a, T, Synchronous> {
    pub(crate) fn new(node: &'a Node<T>) -> Self {
        Self {
            node,
            strategy: PhantomData,
        }
    }

    /// Calls the given closure for each node in the tree rooted by self.
    pub fn for_each<F>(self, mut f: F)
    where
        F: FnMut(&Node<T>),
    {
        macros::for_each_immersion!(&Node<T>, iter);
        for_each_immersion(self.node, &mut f);
    }

    /// Builds a new tree by calling the given closure along the tree rooted by self.
    pub fn map<F, R>(self, mut f: F) -> TraverseOwned<R, Synchronous>
    where
        F: FnMut(&Node<T>) -> R,
    {
        macros::map_immersion!(&Node<T>, iter);
        TraverseOwned::new(map_immersion::<T, F, R>(self.node, &mut f))
    }

    /// Calls the given closure along the tree rooted by self, reducing it into a single
    /// value.
    pub fn reduce<F, R>(self, mut f: F) -> R
    where
        F: FnMut(&Node<T>, Vec<R>) -> R,
        R: Sized,
    {
        macros::reduce_immersion!(&Node<T>, iter);
        reduce_immersion(self.node, &mut f)
    }

    /// Calls the given closure along the tree rooted by self, providing the parent's
    /// result to its children.
    pub fn cascade<F, R>(self, base: R, mut f: F) -> Self
    where
        F: FnMut(&Node<T>, &R) -> R,
        R: Sized,
    {
        macros::cascade_immersion!(&Node<T>, iter);
        cascade_immersion(self.node, &base, &mut f);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node;

    #[test]
    fn test_for_each() {
        let root = node!(10, node!(20, node!(40)), node!(30, node!(50)));

        let mut result = Vec::new();
        Traverse::new(&root).for_each(|n| result.push(n.value));
        assert_eq!(result, vec![40, 20, 50, 30, 10]);
    }

    #[test]
    fn test_reduce() {
        let root = node!(10, node!(20, node!(40)), node!(30, node!(50)));

        let sum = root
            .traverse()
            .reduce(|n, results| n.value + results.iter().sum::<i32>());

        assert_eq!(sum, 150);
    }

    #[test]
    fn test_cascade() {
        let root = node!(10, node!(20, node!(40)), node!(30, node!(50)));

        let mut result = Vec::new();
        root.traverse().cascade(0, |n, parent_value| {
            result.push(n.value + parent_value);
            n.value + parent_value
        });

        assert_eq!(result, vec![10, 30, 70, 40, 90]);
    }
}
