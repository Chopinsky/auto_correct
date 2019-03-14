use std::cmp::Ordering;

#[derive(Eq, Debug, Hash)]
pub struct Candidate {
    pub word: String,
    pub score: u32,
    pub edit: u8,
}

impl Candidate {
    pub(crate) fn new(word: String, score: u32, edit: u8) -> Self {
        Candidate { word, score, edit }
    }

    pub fn get_word(&self) -> String {
        self.word.to_owned()
    }
}

impl Clone for Candidate {
    fn clone(&self) -> Self {
        Candidate {
            word: self.word.clone(),
            score: self.score,
            edit: self.edit,
        }
    }
}

impl Ord for Candidate {
    fn cmp(&self, other: &Candidate) -> Ordering {
        //TODO: use calculations
        if self.edit == other.edit {
            self.score.cmp(&other.score)
        } else {
            other.edit.cmp(&self.edit)
        }
    }
}

impl PartialOrd for Candidate {
    fn partial_cmp(&self, other: &Candidate) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Candidate {
    fn eq(&self, other: &Candidate) -> bool {
        self.word == other.word
    }
}