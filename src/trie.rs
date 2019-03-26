use support::en_us;

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

    pub(crate) fn insert(_word: &str) {
        if let Some(root) = dict_mut() {
            let mut curr = root;
        }
    }

    pub(crate) fn contains(word: &str) -> Option<u32> {
        if let Some(root) = dict_ref() {
            let mut curr = root;

            'outer: while let Some(rune) = word.chars().next() {
                // quick reject
                if curr.occupied == 0 || !curr.check_bit(rune) {
                    return None;
                }

                // check which child match the current rune
                for child in curr.children.as_slice() {
                    if child.rune == rune {
                        curr = child;
                        continue 'outer;
                    }
                }

                // shouldn't happen since we've checked the bits, but just be sure
                return None;
            }

            return curr.word.as_ref().and_then(|(_, score)| Some(score.clone()));
        }

        None
    }

    fn check_bit(&self, rune: char) -> bool {
        (self.occupied & (1 << en_us::get_char_code(rune))) == 1
    }

    fn add_bit(&mut self, rune: char) {
        self.occupied |= (1 << en_us::get_char_code(rune))
    }
}

impl Default for Node {
    fn default() -> Self {
        Node {
            rune: '\u{0000}',
            occupied: 0,
            children: Vec::new(),
            word: None,
        }
    }
}

#[inline]
fn dict_ref() -> Option<&'static Node> {
    unsafe { DICT.as_ref() }
}

#[inline]
fn dict_mut() -> Option<&'static mut Node> {
    unsafe {
        if DICT.is_none() {
            DICT.replace(Node::default());
        }

        DICT.as_mut()
    }
}