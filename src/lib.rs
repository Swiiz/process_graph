#![doc = include_str!("../README.md")]

/// The core trait for any node in a process graph.
/// Each node takes an input `In` and produces an output `Out`.
pub trait GraphNode<In, Out> {
    /// Executes the processing logic for this node.
    fn run(&mut self, input: In) -> Out;

    /// Chains this node with another `next` node.
    /// The output of `self` becomes the input of `next`.
    fn pipe<Next, NextOut>(self, next: Next) -> Graph<In, Out, NextOut, Self, Next>
    where
        Self: Sized,
        Next: GraphNode<Out, NextOut>,
    {
        Graph::new(self, next)
    }
}

impl<In, Out, F: FnMut(In) -> Out> GraphNode<In, Out> for F {
    fn run(&mut self, input: In) -> Out {
        self(input)
    }
}

/// A composite node representing a sequential execution of two `GraphNode`s.
/// `src` is the source node, `sink` is the destination node.
pub struct Graph<In, Mid, Out, T, U>
where
    T: GraphNode<In, Mid>,
    U: GraphNode<Mid, Out>,
{
    pub src: T,
    pub sink: U,
    _marker: std::marker::PhantomData<(In, Mid, Out)>,
}

impl<In, Mid, Out, T, U> Graph<In, Mid, Out, T, U>
where
    T: GraphNode<In, Mid>,
    U: GraphNode<Mid, Out>,
{
    pub fn new(src: T, sink: U) -> Self {
        Self {
            sink,
            src,
            _marker: std::marker::PhantomData,
        }
    }
}

/// Implementation of `GraphNode` for `Graph` struct.
/// This allows chaining `Graph` instances, forming longer processing sequences.
impl<In, Mid, Out, T, U> GraphNode<In, Out> for Graph<In, Mid, Out, T, U>
where
    T: GraphNode<In, Mid>,
    U: GraphNode<Mid, Out>,
{
    fn run(&mut self, input: In) -> Out {
        self.sink.run(self.src.run(input))
    }
}

macro_rules! impl_graph_node_for_tuples {
    ($T_first:ident $In_first:ident $Out_first:ident) => {
        impl<
            $In_first, $Out_first,
            $T_first: GraphNode<$In_first, $Out_first>
        > GraphNode<($In_first,), ($Out_first,)> for ($T_first,)
        {
            #[allow(non_snake_case)]
            fn run(&mut self, input: ($In_first,)) -> ($Out_first,) {
                let ($T_first,) = self; // Deconstruct self (mutable reference)
                let ($In_first,) = input;       // Deconstruct input
                ($T_first.run($In_first),)
            }
        }
    };
    ($T_first:ident $In_first:ident $Out_first:ident, $($T_rest:ident $In_rest:ident $Out_rest:ident),*) => {
        impl<
            $In_first, $Out_first,
            $($In_rest, $Out_rest,)*
            $T_first: GraphNode<$In_first, $Out_first>,
            $($T_rest: GraphNode<$In_rest, $Out_rest>),*
        > GraphNode<($In_first, $($In_rest),*), ($Out_first, $($Out_rest),*)> for ($T_first, $($T_rest),*)
        {
            #[allow(non_snake_case)]
            fn run(&mut self, input: ($In_first, $($In_rest),*)) -> ($Out_first, $($Out_rest),*) {
                let ($T_first, $($T_rest),*) = self;
                let ($In_first, $($In_rest),*) = input;
                (
                    $T_first.run($In_first),
                    $($T_rest.run($In_rest)),*
                )
            }
        }

        impl_graph_node_for_tuples!($($T_rest $In_rest $Out_rest),*);
    };
}
impl_graph_node_for_tuples!(
    A InA OutA, B InB OutB, C InC OutC, D InD OutD,
    E InE OutE, F InF OutF, G InG OutG, H InH OutH
);

/// Macro to easily build a Directed Acyclic Graph (DAG) of GraphNodes.
///
/// Syntax:
/// `graph! { => initial_node => next_node => (branch_node1, branch_node2) => final_node }`
///
/// - `=> expr`: Chains the previous node's output to the next single node's input using `pipe`.
/// - `=> (expr1 expr2 ...)`: Branches the previous node's output to multiple nodes
///   (provided as a tuple), collecting their outputs into a tuple.
#[macro_export]
macro_rules! graph {
    (=> $first_node:expr $(=> $($rest:tt)*)?) => {
        graph!(@build $first_node $(=> $($rest)*)?)
    };

    (@build $current_pipeline:expr) => {
        $current_pipeline
    };

    (@build $current_pipeline:expr => $next_node_expr:expr $(=> $($rest:tt)*)?) => {
        {
            let next_pipeline = $current_pipeline.pipe($next_node_expr);
            graph!(@build next_pipeline $(=> $($rest)*)?)
        }
    };
}
