//! Asynchronous traversal implementation.

use crate::{traversal::TraverseMut, Asynchronous, Node, Synchronous};
use async_recursion::async_recursion;
use futures::future::join_all;
use std::marker::PhantomData;

impl<'a, T> From<TraverseMut<'a, T, Synchronous>> for TraverseMut<'a, T, Asynchronous>
where
    T: Sync + Send,
{
    fn from(value: TraverseMut<'a, T, Synchronous>) -> Self {
        TraverseMut::new_async(value.node)
    }
}

impl<'a, T> TraverseMut<'a, T, Asynchronous> {
    /// Converts the asynchronous traverse into a synchronous one.
    pub fn into_sync(self) -> TraverseMut<'a, T, Synchronous> {
        self.into()
    }
}

impl<'a, T: Sync + Send> TraverseMut<'a, T, Asynchronous> {
    pub fn new_async(node: &'a mut Node<T>) -> Self {
        Self {
            node,
            strategy: PhantomData,
        }
    }

    /// Calls the given closure for each node in the tree rooted by self following the pre-order traversal.
    #[async_recursion]
    pub async fn preorder<F>(&mut self, f: F) -> &mut Self
    where
        F: Fn(&mut Node<T>) + Sync + Send,
    {
        #[async_recursion]
        pub async fn immersion_mut<T, F>(root: &mut Node<T>, f: &F)
        where
            T: Sync + Send,
            F: Fn(&mut Node<T>) + Sync + Send,
        {
            f(root);

            let futures: Vec<_> = root
                .children_mut()
                .iter_mut()
                .map(|child| immersion_mut(child, f))
                .collect();

            join_all(futures).await;
        }

        immersion_mut(self.node, &f).await;

        self
    }

    /// Calls the given closure for each node in the tree rooted by self following the post-order traversal.
    #[async_recursion]
    pub async fn postorder<F>(&mut self, f: F) -> &mut Self
    where
        F: Fn(&mut Node<T>) + Sync + Send,
    {
        #[async_recursion]
        pub async fn immersion_mut<T, F>(root: &mut Node<T>, f: &F)
        where
            T: Sync + Send,
            F: Fn(&mut Node<T>) + Sync + Send,
        {
            let futures: Vec<_> = root
                .children_mut()
                .iter_mut()
                .map(|child| immersion_mut(child, f))
                .collect();

            join_all(futures).await;
            f(root);
        }

        immersion_mut(self.node, &f).await;

        self
    }

    /// Calls the given closure recursivelly along the tree rooted by self.
    /// This method traverses the tree in post-order, and so the second parameter of f is a vector
    /// containing the returned value of f for each child in that node given as the first parameter.
    #[async_recursion]
    pub async fn reduce<F, R>(&mut self, f: F) -> R
    where
        F: Fn(&mut Node<T>, Vec<R>) -> R + Sync + Send,
        R: Sized + Sync + Send,
    {
        #[async_recursion]
        async fn immersion_mut<T, F, R>(root: &mut Node<T>, f: &F) -> R
        where
            T: Sync + Send,
            F: Fn(&mut Node<T>, Vec<R>) -> R + Sync + Send,
            R: Sized + Sync + Send,
        {
            let futures: Vec<_> = root
                .children_mut()
                .iter_mut()
                .map(|child| immersion_mut(child, f))
                .collect();

            let results = join_all(futures).await;
            f(root, results)
        }

        immersion_mut(self.node, &f).await
    }

    /// Calls the given closure recursivelly along the tree rooted by self.
    /// This method traverses the tree in pre-order, and so the second parameter of f is the returned
    /// value of calling f on the parent of that node given as the first parameter.
    #[async_recursion]
    pub async fn cascade<F, R>(&mut self, base: R, f: F)
    where
        F: Fn(&mut Node<T>, &R) -> R + Sync + Send,
        R: Sized + Sync + Send,
    {
        #[async_recursion]
        async fn immersion_mut<T, F, R>(root: &mut Node<T>, base: &R, f: &F)
        where
            T: Sync + Send,
            F: Fn(&mut Node<T>, &R) -> R + Sync + Send,
            R: Sized + Sync + Send,
        {
            let base = f(root, base);
            let futures = root
                .children_mut()
                .iter_mut()
                .map(|child| immersion_mut(child, &base, f));

            join_all(futures).await;
        }

        immersion_mut(self.node, &base, &f).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node;
    use std::sync::{Arc, Mutex};

    #[tokio::test]
    async fn test_node_preorder_mut() {
        let mut root = node!(10_i32, node!(20, node!(40)), node!(30, node!(50)));

        let result = Arc::new(Mutex::new(Vec::new()));
        root.traverse_mut()
            .into_async()
            .preorder(|n| {
                n.set_value(n.value().saturating_add(1));
                result.clone().lock().unwrap().push(*n.value())
            })
            .await;

        let got = result.lock().unwrap();
        assert_eq!(got[0], 11);
        assert!(got.contains(&21));
        assert!(got.contains(&31));
        assert!(got.contains(&41));
        assert!(got.contains(&51));
    }

    #[tokio::test]
    async fn test_node_postorder_mut() {
        let mut root = node!(10_i32, node!(20, node!(40)), node!(30, node!(50)));

        let result = Arc::new(Mutex::new(Vec::new()));
        root.traverse_mut()
            .into_async()
            .postorder(|n| {
                n.set_value(n.value().saturating_add(1));
                result.clone().lock().unwrap().push(*n.value());
            })
            .await;

        let got = result.lock().unwrap();
        assert!(got.contains(&41));
        assert!(got.contains(&21));
        assert!(got.contains(&51));
        assert!(got.contains(&31));
        assert_eq!(got[got.len() - 1], 11);
    }

    #[tokio::test]
    async fn test_node_reduce_mut() {
        let mut root = node!(10_i32, node!(20, node!(40)), node!(30, node!(50)));

        let result = Arc::new(Mutex::new(Vec::new()));
        let sum = root
            .traverse_mut()
            .into_async()
            .reduce(|n, results| {
                n.set_value(n.value().saturating_add(1));
                result.clone().lock().unwrap().push(*n.value());
                n.value() + results.iter().sum::<i32>()
            })
            .await;

        assert_eq!(sum, 155);

        let got = result.lock().unwrap();
        assert!(got.contains(&41));
        assert!(got.contains(&21));
        assert!(got.contains(&51));
        assert!(got.contains(&31));
        assert_eq!(got[got.len() - 1], 11);
    }

    #[tokio::test]
    async fn test_node_cascade_mut() {
        let mut root = node!(10, node!(20, node!(40)), node!(30, node!(50)));

        let result = Arc::new(Mutex::new(Vec::new()));
        root.traverse_mut()
            .into_async()
            .cascade(0, |n, parent_value| {
                let next = n.value() + parent_value;
                result.clone().lock().unwrap().push(next);
                n.set_value(*parent_value);
                next
            })
            .await;

        assert_eq!(root.value, 0);
        assert_eq!(root.children[0].value, 10);
        assert_eq!(root.children[1].value, 10);
        assert_eq!(root.children[0].children[0].value, 30);
        assert_eq!(root.children[1].children[0].value, 40);

        let got = result.lock().unwrap();
        assert_eq!(got[0], 10);
        assert!(got.contains(&30));
        assert!(got.contains(&40));
        assert!(got.contains(&70));
        assert!(got.contains(&90));
    }
}
