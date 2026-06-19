//! Tree trait and related utilities
//!
//! We have a lot of tree structures in this module, so its worth defining a few
//! abstractions for folding, mapping, and traversing these trees

/// Tree trait defining common operations on tree structures.
///
/// This trait provides methods for accessing children, values, and the root of the tree,
/// as well as utility methods for folding and linearizing the tree.

/// `children`/`value`/`root` describe the shape; the default methods (`fold_recursive`,
/// `max_depth`, `linearize`) build on them. All traversals assume a genuine tree: every
/// node is reached exactly once. A DAG (shared child) will be visited — and evaluated —
/// once per path to it.
pub trait Tree {
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
    /// `children`/`root`/`linearize`.
    fn iter(&self) -> impl Iterator<Item = Self::NodeId>;

    /// Get the maximum depth of the tree.
    fn max_depth(&self) -> usize {
        let d = self.fold_recursive(&|_, a: usize| a + 1, &|a, d| a.max(d), 0usize, self.root());
        // The fold_recursive call returns the depth including the current node,
        // so we subtract 1 to get the maximum depth relative to the root.
        assert!(d > 0);
        d - 1
    }

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

    /// Linearize the subtree rooted at `node` into a flat, post-order evaluation plan.
    ///
    /// The plan borrows the node values, so it stays valid only while the tree is
    /// not mutated. Build it once (e.g. at compile time) and reuse it across many
    /// evaluations. Requires a non-empty subtree.
    fn linearize(&self, node: Self::NodeId) -> Linearized<'_, Self::Item> {
        let mut dfs_stack = vec![node];
        let mut linearized = Vec::new();
        while let Some(current) = dfs_stack.pop() {
            let num_children = self.children(current).map(|c| dfs_stack.push(c)).count();
            linearized.push((self.value(current), num_children));
        }
        linearized.reverse();
        Linearized { ops: linearized }
    }
}

/// A tree flattened into a post-order evaluation plan: `(node, child count)` pairs.
///
/// Pure, immutable data — it carries no scratch and is not parameterized by the
/// evaluation result type, so one plan can be evaluated to different result types
/// (e.g. wavefunctions for the real run, strings for a debug trace) and shared
/// across threads, each supplying its own scratch buffer.
#[derive(Debug)]
pub struct Linearized<'a, N> {
    /// Depth-first post-order representation of the tree (node, number of children).
    ops: Vec<(&'a N, usize)>,
}

impl<'a, N> Linearized<'a, N> {
    /// Evaluate the plan, reducing each node from its children's results via `f`.
    ///
    /// `scratch` is the working stack; it is cleared on entry and left empty on
    /// return. The caller owns it so it can be reused across evaluations (and so
    /// `&self` can be shared concurrently, each thread passing its own buffer).
    pub fn eval<R, F>(&self, scratch: &mut Vec<R>, f: F) -> R
    where
        F: Fn(&N, &[R]) -> R,
    {
        scratch.clear();
        for (node, children_count) in &self.ops {
            let start = scratch.len() - children_count;
            let result = f(node, &scratch[start..]);
            scratch.truncate(start);
            scratch.push(result);
        }
        assert_eq!(scratch.len(), 1, "tree eval must leave exactly one result");
        scratch.pop().unwrap()
    }

    /// Convenience wrapper for callers not in a hot loop: allocates a fresh scratch
    /// buffer per call. Prefer [`Linearized::eval`] with a reused buffer on hot paths.
    pub fn eval_once<R, F>(&self, f: F) -> R
    where
        F: Fn(&N, &[R]) -> R,
    {
        self.eval(&mut Vec::new(), f)
    }

    /// Number of nodes in the plan.
    pub fn len(&self) -> usize {
        self.ops.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
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
        fn iter(&self) -> impl Iterator<Item = Self::NodeId> {
            0..self.values.len()
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
        let linearized = tree.linearize(0);
        let result = linearized.eval_once(|node, children_results: &[i32]| {
            *node + children_results.iter().sum::<i32>()
        });
        assert_eq!(result, 6);
    }

    /// Non-commutative combiner: locks left-to-right child order, which a
    /// sum-based test cannot catch. This is the invariant physics eval relies on
    /// (bra/ket order, `p_out - p_in`).
    #[test]
    fn test_eval_preserves_child_order() {
        // root 0 -> [1, 2]; node 1 -> [3, 4]; rest leaves.
        let tree = TestTree {
            values: vec![0, 1, 2, 3, 4],
            children: vec![vec![1, 2], vec![3, 4], vec![], vec![], vec![]],
        };
        let s = tree.linearize(0).eval_once(|node, kids: &[String]| {
            if kids.is_empty() {
                node.to_string()
            } else {
                format!("{node}({})", kids.join(","))
            }
        });
        assert_eq!(s, "0(1(3,4),2)");
    }

    /// The plan is not tied to the item type: evaluate an i32 tree to a String.
    #[test]
    fn test_eval_result_type_differs_from_item() {
        let tree = TestTree {
            values: vec![10, 20, 30],
            children: vec![vec![1, 2], vec![], vec![]],
        };
        let total: i64 = tree
            .linearize(0)
            .eval_once(|node, kids: &[i64]| *node as i64 + kids.iter().sum::<i64>());
        assert_eq!(total, 60);
    }

    /// One plan, one reused scratch buffer, two evaluations; buffer ends empty.
    #[test]
    fn test_eval_reuses_scratch() {
        let tree = TestTree {
            values: vec![1, 2, 3],
            children: vec![vec![1, 2], vec![], vec![]],
        };
        let plan = tree.linearize(0);
        let mut scratch: Vec<i32> = Vec::new();
        let sum = |node: &i32, kids: &[i32]| *node + kids.iter().sum::<i32>();
        assert_eq!(plan.eval(&mut scratch, sum), 6);
        assert_eq!(plan.eval(&mut scratch, sum), 6);
        assert!(scratch.is_empty());
    }

    #[test]
    fn test_single_node() {
        let tree = TestTree {
            values: vec![42],
            children: vec![vec![]],
        };
        assert_eq!(tree.max_depth(), 0);
        let plan = tree.linearize(0);
        assert_eq!(plan.len(), 1);
        assert_eq!(plan.eval_once(|node, _: &[i32]| *node), 42);
    }

    /// Left-leaning chain: 0 -> 1 -> 2 -> 3.
    #[test]
    fn test_chain_depth_and_plan_length() {
        let tree = TestTree {
            values: vec![0, 1, 2, 3],
            children: vec![vec![1], vec![2], vec![3], vec![]],
        };
        assert_eq!(tree.max_depth(), 3);
        let plan = tree.linearize(0);
        assert_eq!(plan.len(), 4);
        let sum = plan.eval_once(|node, kids: &[i32]| *node + kids.iter().sum::<i32>());
        assert_eq!(sum, 6);
    }

    /// `fold_recursive` directly, independent of `max_depth`: count nodes and sum values.
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
