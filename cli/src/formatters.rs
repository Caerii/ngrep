use std::io::Write;

use colored::{Color, ColoredString, Colorize};

macro_rules! try_write {
    ($handle:expr, $($arg:tt)*) => {
        write!($handle, $($arg)*).expect("Failed to write to handle")
    };
}

pub struct MatchFormatterOptions {
    colors: Option<Vec<Color>>,
    line_number: bool,
    only_matching: bool,
}

impl MatchFormatterOptions {
    pub fn default() -> Self {
        MatchFormatterOptions {
            colors: Some(vec![Color::Red]),
            line_number: false,
            only_matching: false,
        }
    }

    #[allow(dead_code)]
    pub fn with_colors(mut self, colors: Option<Vec<Color>>) -> Self {
        self.colors = colors;
        self
    }

    pub fn with_line_number(mut self, line_number: bool) -> Self {
        self.line_number = line_number;
        self
    }

    pub fn with_only_matching(mut self, only_matching: bool) -> Self {
        self.only_matching = only_matching;
        self
    }
}

/// Simple line-buffered colored match formatter.
pub struct MatchFormatter {
    opts: MatchFormatterOptions,
}

impl MatchFormatter {
    pub fn new(options: MatchFormatterOptions) -> Self {
        MatchFormatter { opts: options }
    }

    pub fn display_line<W: Write>(
        &self,
        writer: &mut W,
        line_inx: usize,
        line: &str,
        matches: &[(usize, usize)],
    ) {
        let mut end = 0;
        for (i, match_span) in matches.iter().enumerate() {
            let (match_start, match_end) = *match_span;
            let mut match_ = ColoredString::from(&line[match_start..match_end]);

            if let Some(colors) = &self.opts.colors {
                match_.fgcolor = Some(colors[i % colors.len()]);
                match_ = match_.bold();
            }

            if i == 0 && self.opts.line_number {
                try_write!(writer, "{}:", line_inx + 1);
            }
            if !self.opts.only_matching {
                try_write!(writer, "{}{}", &line[end..match_start], match_);
            } else {
                try_write!(writer, "{}\n", match_);
            }

            end = match_end;
        }
        if !self.opts.only_matching {
            try_write!(writer, "{}\n", &line[end..]);
        }

        writer.flush().expect("Error flushing output");
    }
}

mod tests {
    use super::*;

    #[cfg(test)]
    fn setup_formatter(opts: MatchFormatterOptions) -> (Vec<u8>, MatchFormatter) {
        let out = Vec::new();
        let formatter = MatchFormatter::new(opts);

        (out, formatter)
    }

    #[test]
    fn test_display_line_shows_colored_match() {
        let (mut out, formatter) = setup_formatter(
            MatchFormatterOptions::default()
                .with_colors(Some(vec![Color::Red]))
                .with_only_matching(false),
        );

        formatter.display_line(&mut out, 0, "hello world", &[(0, 5)]);
        assert_eq!(
            String::from_utf8(out).unwrap(),
            "\x1b[1;31mhello\x1b[0m world\n"
        );
    }

    #[test]
    fn test_display_line_number_starts_from_one() {
        let (mut out, formatter) =
            setup_formatter(MatchFormatterOptions::default().with_line_number(true));

        formatter.display_line(&mut out, 0, "hello world", &[(0, 5)]);
        assert!(String::from_utf8(out).unwrap().starts_with("1:"));
    }

    #[test]
    fn test_only_match_show_just_span() {
        let (mut out, formatter) = setup_formatter(
            MatchFormatterOptions::default()
                .with_colors(None)
                .with_only_matching(true),
        );

        formatter.display_line(&mut out, 0, "hello world", &[(0, 5)]);
        assert_eq!(String::from_utf8(out).unwrap(), "hello\n");
    }
}
