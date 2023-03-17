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

/// A `LabeledGraph` is a `BaseGraph` that also associates a label of type `Label` with each node.=
pub trait LabeledGraph<Label: Copy>: BaseGraph {
    /// Returns the label associated with the given node, if it has one.
    fn get(&self, point: Self::NodeHandle) -> Option<Label>;
}

/// A `GraphFunctorView` is a read-only view of a graph that applies a user-provided mapping function to the labels of the graph.
///
/// This struct is useful when you want to use a graph that has labels of one type (`T`), but you need to work with a graph that has labels of another type (`Tp`), obtained by applying a mapping function `F: Fn(T) -> Tp` to the original labels.
///
/// The `GraphFunctorView` struct implements the `BaseGraph` trait, so it can be used wherever one is expected. The `get` method returns the mapped value, or `None` if the original graph did not contain the given node.
pub struct GraphFunctorView<'a, Graph: ?Sized, T, Tp, F> {
    graph: &'a Graph,
    functor: F,
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
    fn apply<Tp, F: Fn(T) -> Tp>(&self, functor: F) -> GraphFunctorView<Self, T, Tp, F>;
}

impl<T: Copy, Graph: LabeledGraph<T>> TransformableGraph<T> for Graph {
    /// Returns a `GraphFunctorView` that applies a given functor to each node label in the graph.
    ///
    /// # Arguments
    ///
    /// * `functor`: A function that takes a label of the original type and returns a label of the new type.
    fn apply<Tp, F: Fn(T) -> Tp>(&self, functor: F) -> GraphFunctorView<Self, T, Tp, F> {
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
