//! Synchronous implementation of both, the [`Traverser`] and [`TraverserMut`].

use crate::{
    traversal::{macros, TraverseOwned},
    Asynchronous, Node, Order, Synchronous,
};
use std::marker::PhantomData;

impl<T> TraverseOwned<T, Synchronous>
where
    T: Sync + Send,
{
    /// Converts the synchronous traverse into an asynchronous one.
    pub fn into_async(self) -> TraverseOwned<T, Asynchronous> {
        TraverseOwned::<T, Asynchronous>::from(self)
    }
}

impl<T> TraverseOwned<T, Synchronous> {
    pub fn new(node: Node<T>) -> Self {
        Self {
            node,
            strategy: PhantomData,
        }
    }

    /// Calls the given closure recursivelly along the tree rooted by self.
    pub fn for_each<O, F>(mut self, mut f: F) -> Self
    where
        F: FnMut(&mut Node<T>),
        O: Order,
    {
        macros::for_each_immersion!(&mut Node<T>, get_mut);
        for_each_immersion::<O, T, F>(&mut self.node, &mut f);
        self
    }

    /// Builds a new tree by calling the given closure recursivelly along the tree rooted by self.
    pub fn map<O, F, R>(mut self, mut f: F) -> TraverseOwned<R, Synchronous>
    where
        F: FnMut(&mut Node<T>) -> R,
        O: Order,
    {
        macros::map_immersion!(&mut Node<T>, get_mut);
        TraverseOwned::new(map_immersion::<O, T, F, R>(&mut self.node, &mut f))
    }

    /// Calls the given closure recursivelly along the tree rooted by self, reducing it into a single
    /// value.
    ///
    /// This method traverses the tree in post-order, and so the second parameter of f is a vector
    /// containing the returned value of f for each child in that node given as the first parameter.
    pub fn reduce<F, R>(mut self, mut f: F) -> R
    where
        F: FnMut(&mut Node<T>, Vec<R>) -> R,
        R: Sized,
    {
        macros::reduce_immersion!(&mut Node<T>, iter_mut);
        reduce_immersion(&mut self.node, &mut f)
    }

    /// Calls the given closure recursivelly along the tree rooted by self, providing the parent's
    /// result to its children.
    ///
    /// This method traverses the tree in pre-order, and so the second parameter of f is the returned
    /// value of calling f on the parent of that node given as the first parameter.
    pub fn cascade<F, R>(mut self, base: R, mut f: F) -> Self
    where
        F: FnMut(&mut Node<T>, &R) -> R,
        R: Sized,
    {
        macros::cascade_immersion!(&mut Node<T>, iter_mut);
        cascade_immersion(&mut self.node, &base, &mut f);
        self
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
    // use crate::node;

    // #[test]
    // fn test_node_preorder() {
    //     let root = node!(2, node!(4, node!(8)), node!(6, node!(10)));
    //     let new_root = root
    //         .into_traverse()
    //         .preorder(|node| -> f32 { (*node.value() as f32) / 2. })
    //         .preorder(|node| -> char {
    //             char::from_u32(node.value().trunc().abs() as u32 + '0' as u32).unwrap_or_default()
    //         })
    //         .take_node();

    //     assert_eq!(new_root.value, '1');
    //     assert_eq!(new_root.children[0].value, '2');
    //     assert_eq!(new_root.children[1].value, '3');
    //     assert_eq!(new_root.children[0].children[0].value, '4');
    //     assert_eq!(new_root.children[1].children[0].value, '5');
    // }

    // #[test]
    // fn test_node_preorder_mut() {
    //     let mut root = node!(10_i32, node!(20, node!(40)), node!(30, node!(50)));

    //     let mut result = Vec::new();
    //     TraverseMut::new(&mut root).preorder(|n| {
    //         n.set_value(n.value().saturating_add(1));
    //         result.push(*n.value())
    //     });

    //     assert_eq!(result, vec![11, 21, 41, 31, 51]);
    // }

    // #[test]
    // fn test_node_postorder() {
    //     let root = node!(10, node!(20, node!(40)), node!(30, node!(50)));

    //     let mut result = Vec::new();
    //     TraverseOwned::new(&root).postorder(|n| result.push(*n.value()));
    //     assert_eq!(result, vec![40, 20, 50, 30, 10]);
    // }

    // #[test]
    // fn test_node_postorder_mut() {
    //     let mut root = node!(10_i32, node!(20, node!(40)), node!(30, node!(50)));

    //     let mut result = Vec::new();
    //     TraverseMut::new(&mut root).postorder(|n| {
    //         n.set_value(n.value().saturating_add(1));
    //         result.push(*n.value())
    //     });

    //     assert_eq!(result, vec![41, 21, 51, 31, 11]);
    // }

    // #[test]
    // fn test_node_reduce() {
    //     let root = node!(10, node!(20, node!(40)), node!(30, node!(50)));

    //     let sum =
    //         TraverseOwned::new(&root).reduce(|n, results| n.value() + results.iter().sum::<i32>());
    //     assert_eq!(sum, 150);
    // }

    // #[test]
    // fn test_node_reduce_mut() {
    //     let mut root = node!(10_i32, node!(20, node!(40)), node!(30, node!(50)));

    //     let sum = TraverseMut::new(&mut root).reduce(|n, results| {
    //         n.set_value(n.value().saturating_add(1));
    //         n.value() + results.iter().sum::<i32>()
    //     });

    //     assert_eq!(sum, 155);
    // }

    // #[test]
    // fn test_node_cascade() {
    //     let root = node!(10, node!(20, node!(40)), node!(30, node!(50)));

    //     let mut result = Vec::new();
    //     TraverseOwned::new(&root).cascade(0, |n, parent_value| {
    //         result.push(n.value() + parent_value);
    //         n.value() + parent_value
    //     });

    //     assert_eq!(result, vec![10, 30, 70, 40, 90]);
    // }

    // #[test]
    // fn test_node_cascade_mut() {
    //     let mut root = node!(10, node!(20, node!(40)), node!(30, node!(50)));

    //     TraverseMut::new(&mut root).cascade(0, |n, parent_value| {
    //         let next = n.value() + parent_value;
    //         n.set_value(*parent_value);
    //         next
    //     });

    //     assert_eq!(root.value, 0);
    //     assert_eq!(root.children[0].value, 10);
    //     assert_eq!(root.children[1].value, 10);
    //     assert_eq!(root.children[0].children[0].value, 30);
    //     assert_eq!(root.children[1].children[0].value, 40);
    // }
}
