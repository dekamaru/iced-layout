pub enum Node {
    Container { id: Option<String>, children: Vec<Node> },
    Text(String),
}
