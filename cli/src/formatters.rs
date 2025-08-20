use std::io::{self, Write};

use colored::{Color, ColoredString, Colorize};

macro_rules! try_write {
    ($handle:expr, $($arg:tt)*) => {
        write!($handle, $($arg)*).expect("Failed to write to handle")
    };
}

/// Simple line-buffered colored match formatter.
pub struct MatchFormatter {
    colors: Vec<Color>,
    line_number: bool,
    only_matching: bool,
}

impl MatchFormatter {
    pub fn new(line_number: bool, only_matching: bool) -> Self {
        MatchFormatter {
            colors: vec![Color::TrueColor {
                r: 255,
                g: 123,
                b: 123,
            }],
            line_number: line_number,
            only_matching: only_matching,
        }
    }

    pub fn display_line(&self, line_inx: usize, line: &str, matches: &[(usize, usize)]) {
        let stdout = io::stdout();
        let mut handle = stdout.lock();

        if self.line_number {
            try_write!(handle, "{}:", line_inx + 1);
        }

        let mut cursor = 0;
        for (i, match_) in matches.iter().enumerate() {
            let (start, end) = *match_;
            let mut match_colored = ColoredString::from(&line[start..end]);
            match_colored.fgcolor = Some(self.colors[i % self.colors.len()]);

            if !self.only_matching {
                try_write!(handle, "{}{}", &line[cursor..start], match_colored.bold());
            } else {
                try_write!(handle, "{}\n", match_colored.bold());
            }

            cursor = end;
        }
        if !self.only_matching {
            try_write!(handle, "{}\n", &line[cursor..]);
        }

        handle.flush().expect("Error flusing stdout");
    }
}
