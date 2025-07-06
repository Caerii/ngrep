use std::io::{self, Write};

use colored::{Color, ColoredString, Colorize};
use fancy_regex::Match;

macro_rules! try_write {
    ($handle:expr, $($arg:tt)*) => {
        write!($handle, $($arg)*).expect("Failed to write to handle")
    };
}

/// Simple line-buffered colored match formatter.
pub struct MatchFormatter {
    colors: Vec<Color>,
}

impl MatchFormatter {
    pub fn default() -> Self {
        Self::new(vec![Color::Red, Color::Blue, Color::Green, Color::Yellow])
    }

    pub fn new(colors: Vec<Color>) -> Self {
        MatchFormatter { colors }
    }

    pub fn display(&self, line_inx: usize, line: &str, matches: &Vec<Match>) {
        let stdout = io::stdout();
        let mut handle = stdout.lock();

        try_write!(handle, "{}:", line_inx);

        let mut cursor = 0;
        for (i, match_) in matches.iter().enumerate() {
            let (start, end) = (match_.start(), match_.end());
            let mut match_colored = ColoredString::from(&line[start..end]);
            match_colored.fgcolor = Some(self.colors[i % self.colors.len()]);

            try_write!(handle, "{}{}", &line[cursor..start], match_colored.bold());

            cursor = end;
        }
        try_write!(handle, "{}\n", &line[cursor..]);

        handle.flush().expect("Error flusing stdout");
    }
}
