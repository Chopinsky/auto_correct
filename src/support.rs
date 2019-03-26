pub(crate) mod en_us {
    pub(crate) static ALPHABET_EN: &'static str = "abcdefghijklmnopqrstuvwxyz";
    pub(crate) const ALPHABET: [&str;26] = ["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "x", "y", "z"];

    #[inline(always)]
    pub(crate) fn get_char_code(rune: char) -> u8 {
        (rune as u32 - 'a' as u32) as u8
    }
}