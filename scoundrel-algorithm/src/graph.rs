use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::Add;

use scoundrel_geometry::{Grid2D, MooreNeighbor, Point};

/// A `BaseGraph` represents a graph data structure where nodes are identified by `NodeHandle`s.
/// This trait provides a method for accessing the adjacent nodes of a given node in the graph.
pub trait BaseGraph {
    type NodeHandle: Copy + Eq + Hash;

    /// Returns a vector of all nodes that are adjacent to the given node.
    fn adjacent_nodes(&self, point: Self::NodeHandle) -> Vec<Self::NodeHandle>;
}

/// A `LabeledGraph` is a `BaseGraph` that also associates a label of type `Label` with each node.
pub trait LabeledGraph<Label: Copy>: BaseGraph {
    /// Returns the label associated with the given node, if it has one.
    fn get(&self, point: Self::NodeHandle) -> Option<Label>;
}

/// A `GraphFunctorView` is a read-only view of a graph that applies a user-provided mapping function to the labels of the graph.
///
/// This struct is useful when you want to use a graph that has labels of one type (`T`), but you need to work with a graph that has labels of another type (`Tp`), obtained by applying a mapping function `F: Fn(T) -> Tp` to the original labels.
///
/// The `GraphFunctorView` struct implements the `BaseGraph` trait, so it can be used wherever one is expected. The `get` method returns the mapped value, or `None` if the original graph did not contain the given node.
/// A view adapter that transforms graph labels through a mapping function.
///
/// This struct allows you to reinterpret a graph with one label type as a graph
/// with a different label type by applying a transformation function.
///
/// # Type Parameters
/// * `'a` - Lifetime of the source graph reference
/// * `Graph` - The original graph type
/// * `T` - The original label type
/// * `Tp` - The transformed label type
/// * `F` - The transformation function type
pub struct GraphFunctorView<'a, Graph: ?Sized, T, Tp, F> {
    /// Reference to the original graph
    graph: &'a Graph,

    /// Function that transforms labels from type T to type Tp
    functor: F,

    /// Type marker for the input and output label types
    marker: PhantomData<(T, Tp)>,
}

impl<'a, Graph: BaseGraph, T, Tp, F> BaseGraph for GraphFunctorView<'a, Graph, T, Tp, F> {
    type NodeHandle = Graph::NodeHandle;

    fn adjacent_nodes(&self, point: Self::NodeHandle) -> Vec<Self::NodeHandle> {
        self.graph.adjacent_nodes(point)
    }
}

impl<'a, Graph: LabeledGraph<T>, T: Copy, Tp: Copy, F: Fn(T) -> Tp> LabeledGraph<Tp>
    for GraphFunctorView<'a, Graph, T, Tp, F>
{
    fn get(&self, point: Self::NodeHandle) -> Option<Tp> {
        self.graph.get(point).map(&self.functor)
    }
}

/// `TransformableGraph` is a trait that provides a method for creating a `GraphFunctorView` that applies
/// a given functor to each node or edge label as it is accessed. The functor must be a function that
/// takes a label of the original type and returns a label of the new type.
pub trait TransformableGraph<T> {
    fn apply<Tp, F: Fn(T) -> Tp>(
        &self,
        functor: F,
    ) -> GraphFunctorView<'_, Self, T, Tp, F>;
}

impl<T: Copy, Graph: LabeledGraph<T>> TransformableGraph<T> for Graph {
    /// Returns a `GraphFunctorView` that applies a given functor to each node label in the graph.
    ///
    /// # Arguments
    ///
    /// * `functor`: A function that takes a label of the original type and returns a label of the new type.
    fn apply<Tp, F: Fn(T) -> Tp>(
        &self,
        functor: F,
    ) -> GraphFunctorView<'_, Self, T, Tp, F> {
        GraphFunctorView {
            graph: self,
            functor,
            marker: Default::default(),
        }
    }
}

/// A trait representing a graph in which distances can be measured between pairs of nodes.
/// This trait is intended to support both grid-based spaces and general graphs with distance
/// metrics.
pub trait SpatialGraph: BaseGraph {
    type Distance: Copy + Ord + Add<Output = Self::Distance> + Default;

    /// Returns the distance between two nodes in the graph.
    fn distance(
        &self,
        pt0: <Self as BaseGraph>::NodeHandle,
        pt1: <Self as BaseGraph>::NodeHandle,
    ) -> Self::Distance;
}

/// A `SpatialGraph` with an associated labeling, mapping each node to a value of type `T`.
///
/// This trait combines the functionality of both `SpatialGraph` and `LabeledGraph<T>`,
/// providing a complete interface for graphs that have both spatial relationships
/// and node labels. It is used extensively in algorithms like A* pathfinding and
/// field of view calculations where both spatial properties and node attributes
/// (like passability or opacity) are important.
///
/// Types implementing this trait can represent game maps, grid-based environments,
/// or any other data structure where nodes have both positions and properties.
pub trait LabeledSpatialGraph<T: Copy>: SpatialGraph + LabeledGraph<T> {}
impl<T, G> LabeledSpatialGraph<T> for G
where
    T: Copy,
    G: SpatialGraph + LabeledGraph<T>,
{
}

impl<T> BaseGraph for Grid2D<T> {
    type NodeHandle = Point;

    fn adjacent_nodes(&self, point: Self::NodeHandle) -> Vec<Self::NodeHandle> {
        MooreNeighbor::all()
            .iter()
            .map(|n| point + n.offset())
            .filter(|pt| self.index(*pt).is_some())
            .collect()
    }
}
impl<T: Copy> SpatialGraph for Grid2D<T> {
    type Distance = i32;

    fn distance(&self, pt0: Self::NodeHandle, pt1: Self::NodeHandle) -> Self::Distance {
        (pt1 - pt0).sqr_magnitude()
    }
}

impl<T: Copy> LabeledGraph<T> for Grid2D<T> {
    fn get(&self, point: Self::NodeHandle) -> Option<T> {
        self.get(point).cloned()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    // A simple graph implementation for testing
    struct TestGraph {
        adjacency: HashMap<u32, Vec<u32>>,
        labels: HashMap<u32, i32>,
    }

    impl TestGraph {
        fn new() -> Self {
            let mut adjacency = HashMap::new();
            let mut labels = HashMap::new();

            // Create simple graph: 0 -- 1 -- 2
            //                      |    |
            //                      3 -- 4
            adjacency.insert(0, vec![1, 3]);
            adjacency.insert(1, vec![0, 2, 4]);
            adjacency.insert(2, vec![1]);
            adjacency.insert(3, vec![0, 4]);
            adjacency.insert(4, vec![1, 3]);

            // Add labels
            labels.insert(0, 10);
            labels.insert(1, 20);
            labels.insert(2, 30);
            labels.insert(3, 40);
            labels.insert(4, 50);

            Self { adjacency, labels }
        }
    }

    impl BaseGraph for TestGraph {
        type NodeHandle = u32;

        fn adjacent_nodes(&self, node: Self::NodeHandle) -> Vec<Self::NodeHandle> {
            self.adjacency.get(&node).cloned().unwrap_or_default()
        }
    }

    impl LabeledGraph<i32> for TestGraph {
        fn get(&self, node: Self::NodeHandle) -> Option<i32> {
            self.labels.get(&node).copied()
        }
    }

    impl SpatialGraph for TestGraph {
        type Distance = u32;

        fn distance(
            &self,
            pt0: Self::NodeHandle,
            pt1: Self::NodeHandle,
        ) -> Self::Distance {
            // Simple implementation: just absolute difference for testing
            (pt0 as i64 - pt1 as i64).unsigned_abs() as u32
        }
    }

    #[test]
    fn test_base_graph_grid2d() {
        let grid = Grid2D::new(3, 3, 0);

        // Test center point adjacency
        let center = Point::new(1, 1);
        let adj = grid.adjacent_nodes(center);

        // Should have 8 neighbors (Moore neighborhood)
        assert_eq!(adj.len(), 8);

        // Test specific adjacency relationships
        assert!(adj.contains(&Point::new(0, 0))); // top-left
        assert!(adj.contains(&Point::new(1, 0))); // top
        assert!(adj.contains(&Point::new(2, 0))); // top-right
        assert!(adj.contains(&Point::new(0, 1))); // left
        assert!(adj.contains(&Point::new(2, 1))); // right
        assert!(adj.contains(&Point::new(0, 2))); // bottom-left
        assert!(adj.contains(&Point::new(1, 2))); // bottom
        assert!(adj.contains(&Point::new(2, 2))); // bottom-right

        // Test edge adjacency (should have fewer neighbors)
        let edge = Point::new(0, 1);
        let edge_adj = grid.adjacent_nodes(edge);
        assert_eq!(edge_adj.len(), 5); // Only 5 neighbors for edge point
    }

    #[test]
    fn test_labeled_graph_grid2d() {
        let mut grid = Grid2D::new(3, 3, 0);

        // Set some values
        grid.set(Point::new(1, 1), 42);
        grid.set(Point::new(0, 0), 10);

        // Test label retrieval
        assert_eq!(grid.get(Point::new(1, 1)), Some(&42));
        assert_eq!(grid.get(Point::new(0, 0)), Some(&10));
        assert_eq!(grid.get(Point::new(2, 2)), Some(&0)); // Default value
        assert_eq!(grid.get(Point::new(3, 3)), None); // Out of bounds
    }

    #[test]
    fn test_spatial_graph_grid2d() {
        let grid = Grid2D::new(5, 5, 0);

        // Test distance calculation
        let p1 = Point::new(1, 1);
        let p2 = Point::new(4, 5); // Out of bounds, but that's fine for distance calc

        // Expected: (4-1)² + (5-1)² = 9 + 16 = 25
        assert_eq!(grid.distance(p1, p2), 25);

        // Test symmetry
        assert_eq!(grid.distance(p1, p2), grid.distance(p2, p1));

        // Test distance to self is zero
        assert_eq!(grid.distance(p1, p1), 0);
    }

    #[test]
    fn test_custom_graph_implementation() {
        let graph = TestGraph::new();

        // Test adjacency
        assert_eq!(graph.adjacent_nodes(0), vec![1, 3]);
        assert_eq!(graph.adjacent_nodes(1), vec![0, 2, 4]);
        assert_eq!(graph.adjacent_nodes(5), Vec::<u32>::new()); // Non-existent node

        // Test labels
        assert_eq!(graph.get(0), Some(10));
        assert_eq!(graph.get(4), Some(50));
        assert_eq!(graph.get(5), None); // Non-existent node

        // Test distance
        assert_eq!(graph.distance(0, 3), 3);
        assert_eq!(graph.distance(1, 4), 3);
    }

    #[test]
    fn test_graph_functor_view() {
        let graph = TestGraph::new();

        // Create a view that doubles each label
        let view = graph.apply(|x| x * 2);

        // Adjacency should be unchanged
        assert_eq!(view.adjacent_nodes(0), vec![1, 3]);
        assert_eq!(view.adjacent_nodes(1), vec![0, 2, 4]);

        // Labels should be doubled
        assert_eq!(view.get(0), Some(20)); // 10 * 2
        assert_eq!(view.get(2), Some(60)); // 30 * 2
        assert_eq!(view.get(5), None); // Non-existent node
    }

    #[test]
    fn test_labeled_spatial_graph() {
        // Grid2D implements both SpatialGraph and LabeledGraph, so it should
        // automatically implement LabeledSpatialGraph
        let mut grid = Grid2D::new(3, 3, 0);
        grid.set(Point::new(1, 1), 42);

        // Test distance calculation
        assert_eq!(grid.distance(Point::new(0, 0), Point::new(2, 2)), 8);
    }
}
