#[macro_use] extern crate maplit;

use dagoba_rust::*;


fn main() {
    
    let mut graph = Graph::new();

    let v1 = graph.add_vertex(hashmap!{
        "name".into() => Value::String("foo".into()),
        "type".into() => Value::String("banana".into())
    }).unwrap();
    
    let v2 = graph.add_vertex(hashmap!{
        "name".into() => Value::String("bar".into()),
        "type".into() => Value::String("orange".into())
    }).unwrap();
    
    graph.add_edge(v1, v2, "fruitier".into(), hashmap!{});
    
    let mut q = Query::new(&graph, VertexFilter::Id(v1));
    let out = q.out(EdgeFilter::None).run();
    
    assert_eq!(out, vec![QueryResult::Vertex(v2)]);
    
}