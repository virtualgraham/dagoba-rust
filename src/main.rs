#[macro_use] extern crate maplit;

use dagoba_rust::*;


fn main() {
    
    
    let mut graph = Graph::new();

    let v1 = graph.add_vertex(hashmap!{
        "name".into() => Value::String("alice".into())
    }).unwrap();
    
    let v2 = graph.add_vertex(hashmap!{
        "name".into() => Value::String("Bob".into()),
        "hobbies".into() => Value::Array(vec![ Value::String("asdf".into()), Value::Object(hashmap!{"x".into() => Value::Int(3)})]),
    }).unwrap();

    graph.add_edge(v1, v2, "knows".into(), hashmap!{});

    let v3 = graph.add_vertex(hashmap!{
        "name".into() => Value::String("charlie".into())
    }).unwrap();

    let v4 = graph.add_vertex(hashmap!{
        "name".into() => Value::String("delta".into())
    }).unwrap();

    graph.add_edge(v2, v4, "parent".into(), hashmap!{});

    graph.add_edge(v2, v3, "knows".into(), hashmap!{});


    
    let mut q = Query::new(&graph, VertexFilter::Id(v1));
    let r = q.out(EdgeFilter::Label("knows".into())).out(EdgeFilter::None).run();
    

    println!("{:?}", r);
}