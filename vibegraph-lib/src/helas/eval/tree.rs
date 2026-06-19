//! Tree trait and related utilities
//!
//! We have a lot of tree structures in this module, so its worth defining a few
//! abstractions for folding, mapping, and traversing these trees

/// Tree trait defining common operations on tree structures.
///
/// This trait provides methods for accessing children, values, and the root of the tree,
/// as well as utility methods for folding and linearizing the tree.

pub trait Tree {
    type Item;
    type NodeId: Copy + Eq + From<usize>;

    /// Returns the children of the given node.
    fn children(&self, node: Self::NodeId) -> impl Iterator<Item = Self::NodeId>;

    /// Returns the value stored at the given node.
    fn value(&self, node: Self::NodeId) -> &Self::Item;

    /// Returns the root node of the tree.
    fn root(&self) -> Self::NodeId;

    /// Get the maximum depth of the tree.
    fn max_depth(&self) -> usize {
        let d = self.fold_recursive(&|_, a: usize| a + 1, &|a, d| a.max(d), 0usize, self.root());
        // The fold_recursive call returns the depth including the current node,
        // so we subtract 1 to get the maximum depth relative to the root.
        assert!(d > 0);
        d - 1
    }

    /// General tree fold helper function
    ///
    /// Folds over the tree starting at the given node, applying the function
    /// `f` to each node and combining results using the function `g`.
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

    /// Linearize the tree
    ///
    /// The linearized representation borrows references to the items in the tree.
    fn linearize<R>(&self, node: Self::NodeId) -> Linearized<'_, Self::Item, R> {
        let mut dfs_stack = vec![node];
        let mut linearized = Vec::new();
        while let Some(current) = dfs_stack.pop() {
            let num_children = self.children(current).map(|c| dfs_stack.push(c)).count();
            linearized.push((self.value(current), num_children));
        }
        linearized.reverse();
        Linearized {
            ops: linearized,
            stack: Vec::new(),
        }
    }
}

/// Linearized representation of a tree, suitable for efficient evaluation.
#[derive(Debug)]
pub struct Linearized<'a, N, R> {
    /// Depth-first post-order representation of the tree
    /// (node, number of children)
    ops: Vec<(&'a N, usize)>,
    /// Scratch space used during evaluation of the linearized tree.
    stack: Vec<R>,
}

impl<'a, N, R> Linearized<'a, N, R> {
    pub fn eval<F>(&mut self, f: F) -> R
    where
        F: Fn(&N, &[R]) -> R,
    {
        self.stack.clear();
        for (node, children_count) in &self.ops {
            let start = self.stack.len() - children_count;
            let slice = &self.stack.as_slice()[start..];
            let result = f(node, slice);
            self.stack.truncate(start);
            self.stack.push(result);
        }
        if self.stack.len() != 1 {
            panic!("Expected a single result on the stack");
        }
        self.stack.pop().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestTree {
        values: Vec<i32>,
        children: Vec<Vec<usize>>,
    }

    impl super::Tree for TestTree {
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
    }

    #[test]
    fn test_max_depth() {
        let tree = TestTree {
            values: vec![1, 2, 3, 4],
            children: vec![vec![1, 2], vec![3], vec![], vec![]],
        };
        let max_depth = tree.max_depth();
        assert_eq!(max_depth, 2);
    }

    #[test]
    fn test_linearize_and_eval() {
        let tree = TestTree {
            values: vec![1, 2, 3],
            children: vec![vec![1, 2], vec![], vec![]],
        };
        let mut linearized = tree.linearize(0);
        eprintln!("Linearized tree: {:?}", linearized);
        let result = linearized.eval(|node, children_results| {
            eprintln!("Node: {:?}, Children results: {:?}", node, children_results);
            *node + children_results.iter().sum::<i32>()
        });
        assert_eq!(result, 6);
    }
}
