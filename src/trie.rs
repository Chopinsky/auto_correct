static mut DICT: Option<Node> = None;

pub(crate) struct Node {
    rune: char,
    occupied: u32,
    children: Vec<Node>,
    word: Option<(String, u32)>,
}

impl Node {
    fn new() -> Self {
        Node::default()
    }

    pub(crate) fn contains(word: &str) -> bool {

    }
}

impl Default for Node {
    fn default() -> Self {
        Node {
            rune: '\u{0000}',
            occupied: 0,
            children: Vec::with_capacity(26),
            word: None,
        }
    }
}