use std::char;
use channel::Receiver;
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

    fn new_with(rune: char, word: Option<(String, u32)>) -> Self {
        Node {
            rune,
            occupied: 0,
            children: Vec::new(),
            word,
        }
    }

    pub(crate) fn build(rx: Receiver<(String, u32)>) {
        if let Some(root) = dict_mut() {
            for (word, score) in rx.recv() {
                root.insert(word, score, 0);
            }
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

    fn insert(&mut self, word: String, score: u32, index: usize) {
        let len = word.len();
        if len == 0 || index >= len {
            return;
        }

        let (pos, rune) = find_child_pos(&self.children, &word, index);
        if pos == self.children.len() {
            self.add_bit(rune);

            if index == len - 1 {
                self.children.push(Node::new_with(rune, Some((word, score))));
                return;
            } else {
                self.children.push(Node::new_with(rune, None));
            }
        }

        if let Some(child) = self.children.get_mut(pos) {
            if index == len - 1 {
                // update child node if this is the whole word
                child.word = Some((word, score));
            } else {
                // insert to the child if not the last character
                child.insert(word, score, index + 1);
            };
        }
    }

    fn check_bit(&self, rune: char) -> bool {
        (self.occupied & (1 << en_us::get_char_code(rune))) == 1
    }

    fn add_bit(&mut self, rune: char) {
        self.occupied |= 1 << en_us::get_char_code(rune)
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

fn find_child_pos(children: &Vec<Node>, word: &str, index: usize) -> (usize, char) {
    if let Some(raw) = word.as_bytes().get(index) {
        if let Some(rune) = char::from_u32(*raw as u32) {
            let mut index = 0;

            for child in children.iter() {
                if child.rune == rune {
                    break;
                }

                index += 1;
            }

            return (index, rune);
        }
    }

    (children.len(), '\u{0000}')
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