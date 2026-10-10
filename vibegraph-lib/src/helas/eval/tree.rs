//! The [`Tree`] trait shared by the evaluator's tree and arena structures.

/// Common access to a tree structure: children, values and the root.
///
/// `children`/`value`/`root` describe the shape; the default `fold_recursive` builds on
/// them. It assumes a genuine tree: every node is reached exactly once. A DAG (shared
/// child) will be visited — and evaluated — once per path to it.
pub(crate) trait Tree {
    type Item;
    type NodeId: Copy;

    /// Returns the children of the given node.
    fn children(&self, node: Self::NodeId) -> impl Iterator<Item = Self::NodeId>;

    /// Returns the value stored at the given node.
    fn value(&self, node: Self::NodeId) -> &Self::Item;

    /// Returns the root node of the tree.
    fn root(&self) -> Self::NodeId;

    /// Iterate over every node id, in unspecified (storage) order — NOT tree order.
    /// This is just for cheaply scanning every node (e.g. collecting couplings);
    /// resolve a value with [`Tree::value`]. For structural traversal use
    /// `children`/`root`.
    fn iter(&self) -> impl Iterator<Item = Self::NodeId>;

    /// General tree fold (catamorphism).
    ///
    /// At each node, the children's results are reduced left-to-right by `g` starting
    /// from the seed `a`, then `f` combines the node's value with that reduction to
    /// produce the node's result. Note `a` is the **identity/seed for `g`**, broadcast
    /// unchanged to every node — it is not a top-down accumulator that evolves as you
    /// descend.
    fn fold_recursive<F, G, A, R>(&self, f: &F, g: &G, a: A, node: Self::NodeId) -> R
    where
        F: Fn(&Self::Item, A) -> R,
        G: Fn(A, R) -> A,
        A: Clone,
    {
        let value = self.value(node);
        f(
            value,
            self.children(node)
                .map(|child| self.fold_recursive(f, g, a.clone(), child))
                .fold(a.clone(), g),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestTree {
        values: Vec<i32>,
        children: Vec<Vec<usize>>,
    }

    impl Tree for TestTree {
        type NodeId = usize;
        type Item = i32;

        fn value(&self, node: Self::NodeId) -> &Self::Item {
            &self.values[node]
        }
        fn children(&self, node: Self::NodeId) -> impl Iterator<Item = usize> {
            self.children[node].iter().copied()
        }
        fn root(&self) -> Self::NodeId {
            0
        }
        fn iter(&self) -> impl Iterator<Item = Self::NodeId> {
            0..self.values.len()
        }
    }

    /// `fold_recursive` directly: count nodes and sum values.
    #[test]
    fn test_fold_recursive() {
        let tree = TestTree {
            values: vec![1, 2, 3, 4],
            children: vec![vec![1, 2], vec![3], vec![], vec![]],
        };
        let count = tree.fold_recursive(&|_, a: usize| a + 1, &|a, r| a + r, 0usize, 0);
        assert_eq!(count, 4);
        let sum = tree.fold_recursive(&|v, a: i32| v + a, &|a, r| a + r, 0i32, 0);
        assert_eq!(sum, 10);
    }
}
