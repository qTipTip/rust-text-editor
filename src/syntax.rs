use crossterm::style::{Color, ContentStyle, Stylize};
use ropey::Rope;
use std::path::Path;
use tree_sitter::{Language, Parser, Query, QueryCursor, StreamingIterator, Tree};
pub enum HighlightType {
    Keyword,
    String,
    Comment,
    Number,
    Function,
    FunctionMethod,
}

impl HighlightType {
    pub fn to_style(self) -> ContentStyle {
        match self {
            HighlightType::Keyword => ContentStyle::new().with(Color::Blue),
            HighlightType::Comment => ContentStyle::new().with(Color::Green),
            HighlightType::String => ContentStyle::new().with(Color::Red),
            HighlightType::Number => ContentStyle::new().with(Color::Yellow),
            HighlightType::Function => ContentStyle::new().with(Color::DarkCyan),
            HighlightType::FunctionMethod => ContentStyle::new().with(Color::Blue),
        }
    }
}
pub struct SyntaxHighlighter {
    parser: Parser,
    current_language: Option<Language>,
    query: Option<Query>,
    highlight_names: Vec<String>,
}

impl SyntaxHighlighter {
    pub fn new() -> Self {
        Self {
            parser: Parser::new(),
            current_language: None,
            query: None,
            highlight_names: Vec::new(),
        }
    }

    pub fn set_language_from_path(
        &mut self,
        path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Get the extension, map it to string, or default to "".
        let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        // Map extension to a specific language, and a set of tree-sitter-queries.
        let (language, query_source) = match extension {
            "rs" => (
                tree_sitter_rust::LANGUAGE.into(),
                include_str!("../queries/rust.scm"),
            ),
            _ => return Ok(()), // No highlighting for unknown files.
        };

        self.parser.set_language(&language)?;
        let query = Query::new(&language, query_source)?;
        self.highlight_names = query
            .capture_names()
            .iter()
            .map(|s| s.to_string())
            .collect();
        self.query = Some(query);

        self.current_language = Some(language);

        Ok(())
    }

    pub fn highlight_line(
        &self,
        rope: &Rope,
        line_idx: usize,
        tree: &Tree,
    ) -> Vec<(usize, usize, HighlightType)> {
        // If we have no query, return empty highlights
        let Some(query) = &self.query else {
            return Vec::new();
        };

        // Get byte offset for line start, and line end. If end of file, return number of bytes in file as end.
        let line_start_byte = rope.line_to_byte(line_idx);
        let line_end_byte = if line_idx + 1 < rope.len_lines() {
            rope.line_to_byte(line_idx + 1)
        } else {
            rope.len_bytes()
        };

        let mut cursor = QueryCursor::new();
        let mut highlights = Vec::new();

        let line_start_char = rope.byte_to_char(line_start_byte);
        let text_bytes = rope.to_string().into_bytes();

        cursor.set_byte_range(line_start_byte..line_end_byte);

        let mut matches = cursor.matches(query, tree.root_node(), text_bytes.as_slice());

        while let Some(match_) = matches.next() {
            for capture in match_.captures {
                let start_byte = capture.node.start_byte();
                let end_byte = capture.node.end_byte();

                // Skip captures outside our line
                if end_byte <= line_start_byte || start_byte >= line_end_byte {
                    continue;
                }

                let capture_name = &self.highlight_names[capture.index as usize];
                let highlight_type = match capture_name.as_str() {
                    "keyword" => HighlightType::Keyword,
                    "string" => HighlightType::String,
                    "comment" => HighlightType::Comment,
                    "number" => HighlightType::Number,
                    "function" => HighlightType::Function,
                    "function.method" => HighlightType::FunctionMethod,
                    &_ => todo!(),
                };

                // Convert byte positions to character positions relative to line start
                let start_char = rope.byte_to_char(start_byte);
                let end_char = rope.byte_to_char(end_byte);

                let relative_start = start_char - line_start_char;
                let relative_end = end_char - line_start_char;

                highlights.push((relative_start, relative_end, highlight_type));
            }
        }
        highlights.sort_by_key(|&(start, _, _)| start);
        highlights
    }

    pub fn parse(&mut self, text: &str) -> Option<Tree> {
        self.parser.parse(text, None)
    }
}
