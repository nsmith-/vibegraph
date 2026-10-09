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
