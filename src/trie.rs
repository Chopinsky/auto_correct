use std::char;
use channel::Receiver;

use crate::common;
use crate::support::en_us;

static mut DICT: Option<Node> = None;

#[derive(Debug)]
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

    pub(crate) fn build(rx: Receiver<String>) {
        if let Some(root) = dict_mut() {
            for received in rx {
                let temp: Vec<&str> = received.splitn(2, common::DELIM).collect();
                if temp[0].is_empty() {
                    continue;
                }

                if let Ok(score) = temp[1].parse::<u32>() {
                    let word = temp[0].to_owned();

                    let clone = word.clone();
                    let mut chars = clone.chars();
                    let mut vec: Vec<char> = Vec::with_capacity(word.len());

                    while let Some(rune) = chars.next() {
                        vec.push(rune.clone());
                    }

                    root.insert((word, score), vec.as_slice(), 0);
                }
            }
        }
    }

    pub(crate) fn check(word: &str) -> Option<u32> {
        if let Some(root) = dict_ref() {
            let mut curr = root;
            let mut chars = word.chars();

            while let Some(rune) = chars.next() {
                // quick reject
                if curr.occupied == 0 || !curr.check_bit(rune) {
                    return None;
                }

                // check which child match the current rune
                for child in curr.children.iter() {
                    if child.rune == rune {
                        curr = child;
                        break;
                    }
                }
            }

            return curr.word.as_ref().and_then(|(_, score)| Some(score.clone()));
        }

        None
    }

    fn insert(&mut self, content: (String, u32), arr: &[char], index: usize) {
        let len = arr.len();
        if len == 0 || index >= len {
            eprintln!("Failed to insert: {} ({:?} @ {}), len: {}", content.0, arr, index, len);
            return;
        }

        let (pos, rune) = find_child_pos(&self.children, arr[index]);
        if pos == self.children.len() {
            self.add_bit(rune);

            if index == len - 1 {
                self.children.push(Node::new_with(rune, Some(content)));
                return;
            } else {
                self.children.push(Node::new_with(rune, None));
            }
        }

        if let Some(child) = self.children.get_mut(pos) {
            if index == len - 1 {
                // update child node if this is the whole word
                child.word = Some(content);
            } else {
                // insert to the child if not the last character
                child.insert(content, arr, index + 1);
            };
        }
    }

    fn check_bit(&self, rune: char) -> bool {
        (self.occupied >> en_us::get_char_code(rune)) & 1 == 1
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

fn find_child_pos(children: &Vec<Node>, rune: char) -> (usize, char) {
    let mut index = 0;

    for child in children.iter() {
        if child.rune == rune {
            break;
        }

        index += 1;
    }

    return (index, rune);
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