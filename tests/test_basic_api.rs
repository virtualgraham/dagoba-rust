#[macro_use] extern crate maplit;

use dagoba_rust::*;

#[test]
fn simple_graph() {
    // should build an empty graph
    let mut graph = Graph::new();
    assert_eq!(graph.vertices.len(), 0);
    assert_eq!(graph.edges.len(), 0);

    // should add a vertex v1
    let v1 = graph.add_vertex(hashmap!{
        "name".into() => Value::String("foo".into()),
        "type".into() => Value::String("banana".into())
    }).unwrap();

    assert_eq!(graph.vertices.len(), 1);
    assert_eq!(graph.edges.len(), 0);

    // should add another vertex v2
    let v2 = graph.add_vertex(hashmap!{
        "name".into() => Value::String("bar".into()),
        "type".into() => Value::String("orange".into())
    }).unwrap();

    assert_eq!(graph.vertices.len(), 2);
    assert_eq!(graph.edges.len(), 0);

    // should add an edge v1->v2
    graph.add_edge(v1, v2, "fruitier".into(), hashmap!{});

    assert_eq!(graph.vertices.len(), 2);
    assert_eq!(graph.edges.len(), 1);

    // g.v(1) should return v1
    let mut q = Query::new(&graph, VertexFilter::Id(v1));
    let out = q.run();

    assert_eq!(out, vec![QueryResult::Vertex(v1)]);

    // g.v(1).out() should follow out edge v1->v2 and return v2
    let mut q = Query::new(&graph, VertexFilter::Id(v1));
    let out = q.out(EdgeFilter::None).run();

    assert_eq!(out, vec![QueryResult::Vertex(v2)]);

    // g.v(2).in() should follow in edge v2<-v1 and return v1
    let mut q = Query::new(&graph, VertexFilter::Id(v2));
    let out = q.r#in(EdgeFilter::None).run();

    assert_eq!(out, vec![QueryResult::Vertex(v1)]);

    // g.v(2).out() should follow no edge and return nothing
    let mut q = Query::new(&graph, VertexFilter::Id(v2));
    let out = q.out(EdgeFilter::None).run();

    assert_eq!(out, vec![]);
}


#[test]
fn bigger_graph() {

    // should build the graph
    let mut graph = Graph::new();

    let vertices = vec![
        hashmap!{"name" => "Fred"},
        hashmap!{"name" => "Bob"},
        hashmap!{"name" => "Tom"},
        hashmap!{"name" => "Dick"},
        hashmap!{"name" => "Harry"},
        hashmap!{"name" => "Lucy"},
    ];

    let v_ids:Vec<u64> = vertices.iter().map(|h| graph.add_vertex(hashmap!{
        "name".into() => Value::String(h.get("name").unwrap().to_string())
    }).unwrap()).collect();

    graph.add_edge(v_ids[0], v_ids[1], "son".into(), hashmap!{});
    graph.add_edge(v_ids[1], v_ids[2], "son".into(), hashmap!{});
    graph.add_edge(v_ids[1], v_ids[3], "son".into(), hashmap!{});
    graph.add_edge(v_ids[1], v_ids[4], "son".into(), hashmap!{});
    graph.add_edge(v_ids[1], v_ids[5], "daughter".into(), hashmap!{});
    graph.add_edge(v_ids[2], v_ids[3], "brother".into(), hashmap!{});
    graph.add_edge(v_ids[3], v_ids[4], "brother".into(), hashmap!{});
    graph.add_edge(v_ids[4], v_ids[2], "brother".into(), hashmap!{});
    graph.add_edge(v_ids[2], v_ids[4], "brother".into(), hashmap!{});
    graph.add_edge(v_ids[3], v_ids[2], "brother".into(), hashmap!{});
    graph.add_edge(v_ids[4], v_ids[3], "brother".into(), hashmap!{});
    graph.add_edge(v_ids[2], v_ids[5], "sister".into(), hashmap!{});
    graph.add_edge(v_ids[3], v_ids[5], "sister".into(), hashmap!{});
    graph.add_edge(v_ids[4], v_ids[5], "sister".into(), hashmap!{});
    graph.add_edge(v_ids[5], v_ids[2], "brother".into(), hashmap!{});
    graph.add_edge(v_ids[5], v_ids[3], "brother".into(), hashmap!{});
    graph.add_edge(v_ids[5], v_ids[4], "brother".into(), hashmap!{});

    assert_eq!(graph.vertices.len(), 6);
    assert_eq!(graph.edges.len(), 17);

    // g.v(1).out().out() should get all grandkids
    let mut q = Query::new(&graph, VertexFilter::Id(v_ids[0]));
    let out = q.out(EdgeFilter::None).out(EdgeFilter::None).run();

    assert_eq!(out, vec![
        QueryResult::Vertex(v_ids[5]),
        QueryResult::Vertex(v_ids[4]),
        QueryResult::Vertex(v_ids[3]),
        QueryResult::Vertex(v_ids[2]),
    ]);

    // g.v(1).out().in().out() means 'fred is his son's father'
    let mut q = Query::new(&graph, VertexFilter::Id(v_ids[0]));
    let out = q.out(EdgeFilter::None).r#in(EdgeFilter::None).out(EdgeFilter::None).run();

    assert_eq!(out, vec![
        QueryResult::Vertex(v_ids[1])
    ]);

    // g.v(1).out().out('daughter') should get the granddaughters
    let mut q = Query::new(&graph, VertexFilter::Id(v_ids[0]));
    let out = q.out(EdgeFilter::None).out(EdgeFilter::Label("daughter".into())).run();

    assert_eq!(out, vec![
        QueryResult::Vertex(v_ids[5])
    ]);

    // g.v(3).out('sister') means 'who is tom's sister?'
    let mut q = Query::new(&graph, VertexFilter::Id(v_ids[2]));
    let out = q.out(EdgeFilter::Label("sister".into())).run();

    assert_eq!(out, vec![
        QueryResult::Vertex(v_ids[5])
    ]);

    // g.v(3).out().in('son').in('son') means 'who is tom's brother's grandfather?'
    let mut q = Query::new(&graph, VertexFilter::Id(v_ids[2]));
    let out = q.out(EdgeFilter::None).r#in(EdgeFilter::Label("son".into())).r#in(EdgeFilter::Label("son".into())).run();

    assert_eq!(out, vec![
        QueryResult::Vertex(v_ids[0]),
        QueryResult::Vertex(v_ids[0])
    ]);

    // g.v(3).out().in('son').in('son').unique() should return the unique grandfather
    let mut q = Query::new(&graph, VertexFilter::Id(v_ids[2]));
    let out = q.out(EdgeFilter::None).r#in(EdgeFilter::Label("son".into())).r#in(EdgeFilter::Label("son".into())).unique().run();

    assert_eq!(out, vec![
        QueryResult::Vertex(v_ids[0])
    ]);
}