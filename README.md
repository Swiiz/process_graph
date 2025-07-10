 # `process_graph`

 `process_graph` is a flexible Rust crate designed for building and executing **directed acyclic graphs (DAGs)** of operations, also known as processing pipelines or workflows. It provides a generic `GraphNode` trait that allows you to chain together any function or struct that transforms an input into an output.

 Whether you're building data processing pipelines, command sequencing, or any system where discrete steps need to be chained, `process_graph` offers an intuitive and type-safe way to define these flows.

 ## Features

 * **Generic `GraphNode` trait**: Easily make any type or function a node in your graph.
 * **Sequential Pipelining**: Chain operations together using the `.pipe()` method.
 * **Parallel Execution (with Tuples)**: Run multiple nodes in parallel by providing tuple inputs and receiving tuple outputs.
 * **Macro for Expressive Graph Definition**: The `graph!` macro simplifies the creation of complex sequential and parallel pipelines.

 ## How It Works

 At its core, `process_graph` operates on the `GraphNode<In, Out>` trait. This trait defines a single method, `run(&mut self, input: In) -> Out`, which takes an input of type `In` and produces an output of type `Out`.

 ### Chaining Nodes

 The `pipe` method allows you to connect two `GraphNode`s. The output of the first node becomes the input of the second, creating a pipeline. For example, if you have a node that takes an `i32` and returns a `String`, and another that takes a `String` and returns a `Vec<u8>`, you can chain them:

 ```rust
 # use process_graph::{GraphNode, Graph};
 # use std::marker::PhantomData;
 let node1 = |x: i32| x.to_string();
 let node2 = |s: String| s.into_bytes();

 let mut pipeline = node1.pipe(node2);
 let result = pipeline.run(123); // result will be Vec<u8> containing [49, 50, 51]
 assert_eq!(result, vec![49, 50, 51]);
 ```

 ### Tuples for Parallelism and Branching

 `process_graph` uses **tuples** to enable parallel processing and branching within your graphs.

 * **Tuple Input/Output**: A `GraphNode` can take a tuple as input and produce a tuple as output. This means you can create a node that processes multiple distinct pieces of data simultaneously.
 * **Branching**: When you chain a single node's output to a tuple of nodes, its output is effectively duplicated and sent to each node in the tuple (provided their input types match). The outputs of these parallel nodes are then collected into a new tuple.

 The `impl_graph_node_for_tuples!` macro (internally used) automatically implements `GraphNode` for tuples of `GraphNode`s, allowing them to process tuples of inputs in parallel and return tuples of outputs.

 ### The `graph!` Macro

 The `graph!` macro is your primary tool for defining complex processing flows. It provides a more declarative syntax for chaining and branching nodes:

 * `=> node`: Chains the output of the previous stage to a single `node`.
 * `=> (node1, node2, ...)`: Chains the output of the previous stage to multiple nodes in parallel. The outputs of these nodes are then collected into a tuple.

 ## Example

 Let's illustrate with a practical example. Imagine a simple data pipeline that:

 1.  Takes an integer.
 2.  Converts it to a string.
 3.  **Duplicates the string** so it can be processed by two parallel branches.
 4.  In parallel:
     * Counts the number of characters in the string.
     * Parses the string back into an integer (if possible).
 5.  Combines these results into a final output.

 ```rust
 use process_graph::{GraphNode, graph};
 use std::convert::identity; // A simple function that returns its input

 fn main() {
     // 1. Initial node: takes an i32, returns an i32 (identity for demonstration)
     let initial_processing = |x: i32| {
         println!("Initial: Received {}", x);
         x * 2 // Double the input
     };

     // 2. Convert i32 to String
     let to_string_node = |num: i32| {
         println!("ToString: Converting {} to string", num);
         num.to_string()
     };

     // 3. Duplicate the string for parallel branches
     let duplicate_string = |s: String| {
         println!("DuplicateString: Duplicating '{}'", s);
         (s.clone(), s) // Return a tuple of two identical strings
     };

     // 4. Parallel branches on the String output
     // Branch A: Count characters
     let count_chars_node = |s: String| {
         println!("CountChars: Counting chars in '{}'", s);
         s.len()
     };

     // Branch B: Parse string to i32 (handles potential errors by returning Option)
     let parse_int_node = |s: String| {
         println!("ParseInt: Parsing '{}' to int", s);
         s.parse::<i32>().ok() // .ok() converts Result to Option
     };

     // 5. Final node: takes a tuple (usize, Option<i32>) and formats it
     let final_formatter = |(len, parsed_num): (usize, Option<i32>)| {
         format!(
             "Processed: Length = {}, Parsed Integer = {}",
             len,
             parsed_num.map_or_else(|| "N/A".to_string(), |n| n.to_string())
         )
     };

     // Build the graph using the `graph!` macro
     let mut my_pipeline = graph! {
         => initial_processing
         => to_string_node
         => duplicate_string // Insert the duplication node here
         => (count_chars_node, parse_int_node) // Now this expects (String, String) input
         => final_formatter
     };

     // Run the pipeline
     let result = my_pipeline.run(5);
     println!("Final Result: {}", result);
     assert_eq!(result, "Processed: Length = 2, Parsed Integer = 10");

     // Example with a string that can't be fully parsed
     let mut my_pipeline_2 = graph! {
         => |x: i32| {
             println!("\nInitial 2: Received {}", x);
             "hello_world".to_string() // Directly output a string for this example
         }
         => duplicate_string // Use the duplication node again
         => (count_chars_node, parse_int_node)
         => final_formatter
     };

     let result_2 = my_pipeline_2.run(0); // Input doesn't matter much for this path
     println!("Final Result 2: {}", result_2);
     assert_eq!(result_2, "Processed: Length = 11, Parsed Integer = N/A");
 }
 ```