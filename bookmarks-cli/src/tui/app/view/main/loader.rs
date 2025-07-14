use std::fmt::{self, Display};

const START_CHAR: u16 = 0xEE06;
const END_CHAR: u16 = 0xEE0B;
const RANGE: u16 = END_CHAR - START_CHAR;

#[derive(Debug, Default)]
pub struct Loader(u16);

impl Loader {
    pub fn next(&mut self) {
        self.0 = (self.0 + 0x1) % (RANGE + 1);
    }
}

impl Display for Loader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&String::from_utf16_lossy(&[START_CHAR + self.0]))
    }
}
