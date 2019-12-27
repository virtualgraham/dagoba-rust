use std::collections::HashMap;
use std::collections::HashSet;


#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}


#[derive(Debug)]
pub struct Vertex {
    pub properties: HashMap<String, Value>,
    pub e_in: Vec<u64>,
    pub e_out: Vec<u64>,
    pub id: u64
}


#[derive(Debug)]
pub struct Edge {
    pub label: String,
    pub properties: HashMap<String, Value>,
    pub v_in: u64,
    pub v_out: u64,
    pub id: u64
}


#[derive(Debug)]
pub struct Graph {
    pub autoid: u64,
    pub vertices: HashMap<u64, Box<Vertex>>,
    pub edges: HashMap<u64, Box<Edge>>
}


impl Graph {

    pub fn new() -> Graph {
        Graph{
            autoid: 0,
            vertices: HashMap::new(),
            edges: HashMap::new()
        }
    }
    
    
    fn next_id(self: &mut Self) -> u64 {
        self.autoid += 1;
        self.autoid
    }


    pub fn add_vertex(self: &mut Self, properties: HashMap<String, Value>) -> Result<u64, String> {
        
        let id = self.next_id();
        
        self.vertices.insert(id, Box::new(Vertex {
            properties: properties,
            e_in: Vec::new(),
            e_out: Vec::new(),
            id: id
        }));
        
        Ok(id)
    }
    
    
    pub fn add_edge(self: &mut Self, v_out: u64,  v_in: u64, label: String, properties: HashMap<String, Value>) -> Result<u64, String> {
        
        let id = self.next_id();
              
        println!("{} {} {}", v_in, v_out, id);
        
        let vertex_in =  self.vertices.get_mut(&v_in).ok_or("Vertex Not Found")?;
        
        vertex_in.e_in.push(id);
        
        let vertex_out =  self.vertices.get_mut(&v_out).ok_or("Vertex Not Found")?;

        vertex_out.e_out.push(id);

        self.edges.insert(id, Box::new(Edge {
            label: label,
            properties: properties,
            v_in: v_in,
            v_out: v_out,
            id: id
        }));
        
        
        Ok(id)
    }
    
    
    pub fn remove_vertex(self: &mut Self, id:u64) -> Result<(), String> {
    
        let mut edges_to_remove = Vec::new();
        
        {
            let v = self.vertices.get(&id).ok_or("Vertex Not Found")?;
            
            for e in &v.e_in {
                edges_to_remove.push(*e);
            }
            
            for e in &v.e_out {
                edges_to_remove.push(*e);
            }
        }
        
        for e in edges_to_remove {
            self.remove_edge(e)?;
        }
        
        self.vertices.remove(&id);
        
        Ok(())
    }
    
    
    pub fn remove_edge(self: &mut Self, id:u64) -> Result<(), String> {
    
        let e = self.edges.get(&id).ok_or("Edge Not Found")?;
        
        let v_in = self.vertices.get_mut(&e.v_in).ok_or("Vertex Not Found")?;
        let e_in_idx = v_in.e_in.iter().rposition(|&x| x == id).ok_or("Edge Not Found")?;
        v_in.e_in.remove(e_in_idx);
        
        let v_out = self.vertices.get_mut(&e.v_out).ok_or("Vertex Not Found")?;
        let e_out_idx = v_out.e_out.iter().rposition(|&x| x == id).ok_or("Edge Not Found")?;
        v_out.e_out.remove(e_out_idx);
        
        self.edges.remove(&id);
        
        Ok(())
    }
    
    
    pub fn get_verticies(self: &Self, ids: &Vec<u64>) -> Vec<&Box<Vertex>> {
        ids.iter().filter_map( |id| self.vertices.get(&id) ).collect()
    }
    
    
    pub fn get_vertex(self: &Self, id:&u64) -> Option<&Box<Vertex>> {
        self.vertices.get(&id)
    }
    
    
    pub fn search_verticies(self: &Self, filter: &VertexFilter) -> Vec<u64>  {
        if let VertexFilter::Props(p) = filter {
            return self.vertices.values().filter( move |v| properties_filter(&v.properties, p) ).map(|v| v.id).collect()
        } else if let VertexFilter::Id(id) = filter {
            return vec![*id];
        } else if let VertexFilter::Ids(ids) = filter {
            return ids.clone();
        } else if let VertexFilter::Fn(f) = filter {
            return self.vertices.values().filter( |x| f(x) ).map(|v| v.id).collect()
        } else {
            return self.vertices.keys().map(|k| *k).collect();
        }
    }
    
    
    pub fn get_out_edges(self: &Self, vertex_id: &u64) -> Vec<&Box<Edge>> {
        let vertex = self.vertices.get(&vertex_id).unwrap();
        vertex.e_out.iter().filter_map( move |edge_id| self.edges.get(edge_id) ).collect()
    }
    
    
    pub fn get_in_edges(self: &Self, vertex_id: &u64) -> Vec<&Box<Edge>> {
        let vertex = self.vertices.get(&vertex_id).unwrap();
        vertex.e_in.iter().filter_map( move |edge_id| self.edges.get(edge_id) ).collect()
    }
    
    // TODO implements to/from JSON string
}


pub enum EdgeFilter {
    None,
    Label(String),
    Labels(Vec<String>),
    Props(HashMap<String, Value>)
}

pub enum VertexFilter {
    None,
    Id(u64),
    Ids(Vec<u64>),
    Props(HashMap<String, Value>),
    Fn(Box<dyn Fn(&Vertex) -> bool>)
}


#[derive(Debug, Clone, PartialEq)]
pub enum QueryResult {
    None,
    Value(Value),
    Vertex(u64),
}


pub struct Query<'a> {
    pub graph: &'a Graph,
    pub program: Vec<Box<dyn Pipe + 'a>>
}

impl<'a> Query<'a> {
    
    pub fn new(graph: &'a Graph, filter: VertexFilter) -> Query<'a> {
        Query {
            graph: graph,
            program: vec![Box::new(VertexPipe::new(graph, filter))]
        }
    }

    pub fn run(self: &mut Self) -> Vec<QueryResult> {

        // TRANSFORM PROGRAM

        let max = self.program.len() as i32 - 1;

        let mut maybe_gremlin = MaybeGremlin::False;
        let mut results = Vec::new();
        let mut done:i32 = -1;
        let mut pc = max;
    
        while done < max {
            
            let step = &mut self.program[pc as usize];
            maybe_gremlin = step.run(match maybe_gremlin { MaybeGremlin::Gremlin(g) => Some(g), _ => None });

            println!("maybe_gremlin {:?}", maybe_gremlin);

            if let MaybeGremlin::Pull = maybe_gremlin {
                maybe_gremlin = MaybeGremlin::False;
                if pc-1 > done {
                    pc -= 1;
                    continue;
                } else {
                    done = pc;
                }
            }

            if let MaybeGremlin::Done = maybe_gremlin {
                maybe_gremlin = MaybeGremlin::False;
                done = pc;
            }

            pc += 1;

            if pc > max {
                if let MaybeGremlin::Gremlin(r) = maybe_gremlin {
                    results.push(r);
                }
                maybe_gremlin = MaybeGremlin::False;
                pc -= 1;
            }
        }

        results.iter().map(|g| {
                if g.result.is_some() { 
                    QueryResult::Value(g.result.as_ref().unwrap().clone())
                } else if g.vertex.is_some() { 
                    QueryResult::Vertex(g.vertex.unwrap()) 
                } else {
                    QueryResult::None
                }
            }
        ).collect()
    }
    
    // // Pipetypes
    pub fn vertex(self: &mut Self, filter: VertexFilter) -> &mut Self {
        self.program.push(Box::new(VertexPipe::new(&self.graph, filter)));
        self
    }
    
    pub fn r#in(self: &mut Self, filter: EdgeFilter) -> &mut Self {
        self.program.push(Box::new(SimpleTraversalPipe::new(&self.graph, SimpleTraversalDir::In, filter)));
        self
    }
    
    pub fn out(self: &mut Self, filter: EdgeFilter) -> &mut Self {
        self.program.push(Box::new(SimpleTraversalPipe::new(&self.graph, SimpleTraversalDir::Out, filter)));
        self
    }
    
    pub fn both(self: &mut Self, filter: EdgeFilter) -> &mut Self {
        self.program.push(Box::new(SimpleTraversalPipe::new(&self.graph, SimpleTraversalDir::Both, filter)));
        self
    }

    pub fn property(self: &mut Self, property: String) -> &mut Self {
        self.program.push(Box::new(PropertyPipe::new(&self.graph, property)));
        self
    }

    pub fn unique(self: &mut Self) -> &mut Self {
        self.program.push(Box::new(UniquePipe::new()));
        self
    }

    pub fn filter(self: &mut Self, filter:VertexFilter) -> &mut Self {
        self.program.push(Box::new(FilterPipe::new(&self.graph, filter)));
        self
    }

    pub fn take(self: &mut Self, take:i64) -> &mut Self {
        self.program.push(Box::new(TakePipe::new(take)));
        self
    }

    pub fn r#as(self: &mut Self, label:u64) -> &mut Self {
        self.program.push(Box::new(AsPipe::new(label)));
        self
    }

    pub fn back(self: &mut Self, label:u64) -> &mut Self {
        self.program.push(Box::new(BackPipe::new(label)));
        self
    }

    pub fn except(self: &mut Self, label:u64) -> &mut Self {
        self.program.push(Box::new(ExceptPipe::new(label)));
        self
    }

    pub fn merge(self: &mut Self, vertex_ids:Vec<u64>) -> &mut Self {
        self.program.push(Box::new(MergePipe::new(vertex_ids)));
        self
    }
}


pub trait Pipe {
    fn run(self: &mut Self, gremlin: Option<Gremlin>) -> MaybeGremlin;
}


pub struct VertexPipe<'a> {
    init: bool,
    vertices: Vec<u64>,
    graph: &'a Graph,
    filter: VertexFilter,
}


impl<'a> VertexPipe<'a> {
    fn new(graph: &'a Graph, filter: VertexFilter) -> VertexPipe<'a> {
        VertexPipe {
            init: false,
            vertices: Vec::new(),
            graph: graph,
            filter: filter,
        }
    }
}


impl<'a> Pipe for VertexPipe<'a> {
    
    fn run(self: &mut Self, gremlin: Option<Gremlin>) -> MaybeGremlin {
        
        if !self.init {
            self.vertices.extend(self.graph.search_verticies(&self.filter));
            self.init = true;
        } 
        
        if self.vertices.is_empty() {
            return MaybeGremlin::Done
        }
        
        let vertex = self.vertices.pop().unwrap();
            
        return MaybeGremlin::Gremlin ( Gremlin {
            vertex: Some(vertex),
            r#as: match gremlin { Some(g) => g.r#as.clone(), None => None },
            result: None
        })
    }
}



#[derive(Debug, Clone, Copy)]
pub enum SimpleTraversalDir {
    In,
    Out,
    Both
}


pub struct SimpleTraversalPipe<'a> {
    graph: &'a Graph,
    dir: SimpleTraversalDir,
    filter: EdgeFilter,
    edges: Vec<u64>,
    gremlin: Option<Gremlin>,
}


impl<'a> SimpleTraversalPipe<'a> {
    fn new(graph: &'a Graph, dir: SimpleTraversalDir, filter: EdgeFilter) -> SimpleTraversalPipe<'a> {
        SimpleTraversalPipe {
            graph: graph,
            dir: dir,
            filter: filter,
            edges: Vec::new(),
            gremlin: None,
        }
    }

    fn get_edges(gremlin: &Option<Gremlin>, graph: &Graph, filter: &EdgeFilter, dir: SimpleTraversalDir) -> Vec<u64> {
        let vertex_id = gremlin.as_ref().unwrap().vertex.unwrap();
        match dir {
            SimpleTraversalDir::Out => graph.get_out_edges(&vertex_id),
            _ => graph.get_in_edges(&vertex_id),
        }
        .iter().filter(|edge| filter_edge(edge, filter))
        .map(
            |edge| {
                match dir {
                    SimpleTraversalDir::Out => edge.v_in, 
                    _ => edge.v_out,
                }
            }
        )
        .collect()
    }
}


impl<'a> Pipe for SimpleTraversalPipe<'a> {
    
    fn run(self: &mut Self, gremlin: Option<Gremlin>) -> MaybeGremlin {
        if gremlin.is_none() && self.edges.is_empty() {
            return MaybeGremlin::Pull
        }
        
        if self.edges.is_empty() {
            self.gremlin = gremlin;

            let bar = SimpleTraversalPipe::get_edges(&self.gremlin, self.graph, &self.filter, self.dir); 
            self.edges.extend(bar);
            
            if let SimpleTraversalDir::Both = self.dir {
                let bar = SimpleTraversalPipe::get_edges(&self.gremlin, self.graph, &self.filter, SimpleTraversalDir::Out);
                self.edges.extend(bar);
            }
        }
        
        if self.edges.is_empty() {
            return MaybeGremlin::Pull
        }
        
        let vertex = self.edges.pop();
        
        return MaybeGremlin::Gremlin( Gremlin {
            vertex: vertex,
            r#as: self.gremlin.as_ref().unwrap().r#as.clone(),
            result: None
        })
    }
}


pub struct PropertyPipe<'a> {
    graph: &'a Graph,
    property: String
}


impl<'a> PropertyPipe<'a> {
    fn new(graph: &'a Graph, property: String) -> PropertyPipe {
        PropertyPipe {
            graph: graph,
            property: property
        }
    }
}


impl<'a> Pipe for PropertyPipe<'a> {
    
    fn run(self: &mut Self, gremlin: Option<Gremlin>) -> MaybeGremlin {
        if let Option::None = gremlin {
            return MaybeGremlin::Pull
        } 

        let g = gremlin.unwrap();

        if let Option::None = g.vertex {
            return MaybeGremlin::False
        }

        let v_id = g.vertex.unwrap();
        let v = self.graph.get_vertex(&v_id).unwrap();

        let r = v.properties.get(&self.property);
        
        if let Option::None = r {
            return MaybeGremlin::False
        } else {
            let mut g2 = g.clone();
            g2.result = Some(r.unwrap().clone());
            return MaybeGremlin::Gremlin ( g2 )
        }
    }
}


pub struct UniquePipe {
    seen: HashSet<u64>
}


impl UniquePipe {
    fn new() -> UniquePipe {
        UniquePipe {
            seen: HashSet::new()
        }
    }
}


impl Pipe for UniquePipe {
    
    fn run(self: &mut Self, gremlin: Option<Gremlin>) -> MaybeGremlin {
        if let Option::None = gremlin {
            return MaybeGremlin::Pull
        } 

        let g = gremlin.unwrap();

        if let Option::None = g.vertex {
            return MaybeGremlin::Pull
        }

        let v = g.vertex.unwrap();

        if !self.seen.insert(v) {
            return MaybeGremlin::Pull
        } 
        
        return MaybeGremlin::Gremlin ( g.clone() )
    }
}


// // TODO implement something like this
// pub enum FilterPipeArg {
//     VertexFilter(VertexFilter),
//     Closure(Box<dyn Fn(&Vertex) -> bool>)
// }


pub struct FilterPipe<'a> {
    graph: &'a Graph,
    filter: VertexFilter
}


impl<'a> FilterPipe<'a> {
    fn new(graph: &'a Graph, filter: VertexFilter) -> FilterPipe {
        FilterPipe {
            graph: graph,
            filter: filter
        }
    }
}


impl<'a> Pipe for FilterPipe<'a> {
    
    fn run(self: &mut Self, gremlin: Option<Gremlin>) -> MaybeGremlin {
        if let Option::None = gremlin {
            return MaybeGremlin::Pull
        } 

        let g = gremlin.unwrap();

        if let Option::None = g.vertex {
            return MaybeGremlin::Pull
        }

        let v_id = g.vertex.unwrap();
        let v = self.graph.get_vertex(&v_id).unwrap();

        if filter_vertex(v, &self.filter) {
            return MaybeGremlin::Gremlin ( g.clone() ) 
        } else {
            return MaybeGremlin::Pull
        }
    }
}


pub struct TakePipe {
    taken: i64,
    take: i64,
}


impl TakePipe {
    fn new(take: i64) -> TakePipe {
        TakePipe {
            taken: 0,
            take: take
        }
    }
}


impl<'a> Pipe for TakePipe {
    
    fn run(self: &mut Self, gremlin: Option<Gremlin>) -> MaybeGremlin {
        if self.taken == self.take {
            self.taken = 0;
            return  MaybeGremlin::Done
        }

        if let Option::None = gremlin {
            return MaybeGremlin::Pull
        } 

        let g = gremlin.unwrap();

        self.taken += 1;

        return MaybeGremlin::Gremlin ( g.clone() )  
    }
}


pub struct AsPipe {
    label: u64
}


impl AsPipe {
    fn new(label: u64) -> AsPipe {
        AsPipe {
            label: label
        }
    }
}


impl<'a> Pipe for AsPipe {
    
    fn run(self: &mut Self, gremlin: Option<Gremlin>) -> MaybeGremlin {
        if let Option::None = gremlin {
            return MaybeGremlin::Pull
        } 

        let g = gremlin.unwrap();

        if let Option::None = g.vertex {
            return MaybeGremlin::Pull
        }

        let v = g.vertex.unwrap();

        let mut p = HashMap::new();
        p.insert(self.label, v);

        let mut g = g.clone();
        g.r#as = Some(p);

        return MaybeGremlin::Gremlin ( g )  
    }
}


pub struct BackPipe {
    label: u64
}


impl BackPipe {
    fn new(label: u64) -> BackPipe {
        BackPipe {
            label: label
        }
    }
}


impl Pipe for BackPipe {
    
    fn run(self: &mut Self, gremlin: Option<Gremlin>) -> MaybeGremlin {
        if let Option::None = gremlin {
            return MaybeGremlin::Pull
        } 

        let g = gremlin.unwrap();

        let a = g.r#as.as_ref().unwrap();

        let v = a.get(&self.label);

        return MaybeGremlin::Gremlin( Gremlin {
            vertex: match v { None => None, Some(v) => Some(*v) },
            r#as: None,
            result: None
        })
    }
}


pub struct ExceptPipe {
    label: u64
}


impl ExceptPipe {
    fn new(label: u64) -> ExceptPipe {
        ExceptPipe {
            label: label
        }
    }
}


impl<'a> Pipe for ExceptPipe {
    
    fn run(self: &mut Self, gremlin: Option<Gremlin>) -> MaybeGremlin {
        if let Option::None = gremlin {
            return MaybeGremlin::Pull
        } 

        let g = gremlin.unwrap();

        let a = g.r#as.as_ref().unwrap();

        let a_v = *a.get(&self.label).unwrap();

        let v = g.vertex.unwrap();

        if v == a_v {
            return MaybeGremlin::Pull
        }

        return MaybeGremlin::Gremlin ( g.clone() )
    }
}


pub struct MergePipe {
    vertex_ids:Vec<u64>,
    mapped_ids:Option<Vec<u64>>
}


impl MergePipe {
    fn new(vertex_ids:Vec<u64>) -> MergePipe {
        MergePipe {
            vertex_ids: vertex_ids,
            mapped_ids: None
        }
    }
}


impl Pipe for MergePipe {
    
    fn run(self: &mut Self, gremlin: Option<Gremlin>) -> MaybeGremlin {
        if self.mapped_ids.is_none() && gremlin.is_none() {
            return MaybeGremlin::Pull
        }

        if self.mapped_ids.is_none() || self.mapped_ids.as_ref().unwrap().is_empty() {

            let a = match gremlin.as_ref().unwrap().r#as.as_ref() { None => HashMap::new(), Some(s) => s.clone() };
            
            self.mapped_ids = Some(self.vertex_ids.iter().filter_map( |id| match a.get(id) { None => None, Some(v) => Some(*v) }).collect());
   
        }

        if self.mapped_ids.as_ref().unwrap().is_empty() {
            return MaybeGremlin::Pull
        }

        let v = self.mapped_ids.as_mut().unwrap().pop().unwrap();

        return MaybeGremlin::Gremlin( Gremlin {
            vertex: Some(v),
            r#as: gremlin.as_ref().unwrap().r#as.clone(),
            result: None
        })
    }
}


#[derive(Debug, Clone)]
pub struct Gremlin {
    result: Option<Value>,
    vertex: Option<u64>,
    r#as: Option<HashMap<u64, u64>>,
}

#[derive(Debug)]
pub enum MaybeGremlin {
    Pull,
    Done,
    False,
    Gremlin(Gremlin)
}

fn filter_vertex(vertex:&Vertex, filter:&VertexFilter) -> bool {
    match filter {
        VertexFilter::None => true,
        VertexFilter::Id(id) => vertex.id == *id,
        VertexFilter::Ids(ids) => ids.contains(&vertex.id),
        VertexFilter::Props(p) => properties_filter(p, &vertex.properties),
        VertexFilter::Fn(f) => f(vertex)
    }    
}

fn filter_edge(edge:&Edge, filter: &EdgeFilter) -> bool {
    match filter {
        EdgeFilter::None => true,
        EdgeFilter::Label(l) => &edge.label == l,
        EdgeFilter::Labels(v) => v.contains(&edge.label),
        EdgeFilter::Props(p) => properties_filter(p, &edge.properties)
    }
}

fn properties_filter(p: &HashMap<String, Value>, f: &HashMap<String, Value>) -> bool {
    for k in f.keys() {
        if !p.contains_key(k) || p.get(k) != f.get(k) {
            return false
        }
    }
    
    true
}