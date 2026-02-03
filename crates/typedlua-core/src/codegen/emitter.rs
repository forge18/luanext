use crate::codegen::sourcemap::SourceMapBuilder;
use crate::config::OutputFormat;

/// Emitter handles all code generation output operations.
/// This separates output concerns from the CodeGenerator state,
/// making code generation more testable and allowing parallel emission.
pub struct Emitter {
    pub output: String,
    indent_level: usize,
    indent_str: String,
    source_map: Option<SourceMapBuilder>,
    output_format: OutputFormat,
}

impl Emitter {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            indent_level: 0,
            indent_str: "    ".to_string(),
            source_map: None,
            output_format: OutputFormat::Readable,
        }
    }

    pub fn with_source_map(mut self, source_file: String) -> Self {
        self.source_map = Some(SourceMapBuilder::new(source_file));
        self
    }

    pub fn with_output_format(mut self, format: OutputFormat) -> Self {
        self.output_format = format;
        self.indent_str = match format {
            OutputFormat::Minified => "".to_string(),
            OutputFormat::Compact => " ".to_string(),
            OutputFormat::Readable => "    ".to_string(),
        };
        self
    }

    pub fn with_indent_str(mut self, indent_str: String) -> Self {
        self.indent_str = indent_str;
        self
    }

    pub fn write(&mut self, s: &str) {
        self.output.push_str(s);
        if let Some(source_map) = &mut self.source_map {
            source_map.advance(s);
        }
    }

    pub fn writeln(&mut self, s: &str) {
        match self.output_format {
            OutputFormat::Minified => {
                self.output.push_str(s);
            }
            OutputFormat::Compact | OutputFormat::Readable => {
                self.output.push_str(s);
                self.output.push('\n');
            }
        }
        if let Some(source_map) = &mut self.source_map {
            source_map.advance(s);
            if !matches!(self.output_format, OutputFormat::Minified) {
                source_map.advance("\n");
            }
        }
    }

    pub fn indent(&mut self) {
        if !matches!(self.output_format, OutputFormat::Minified) {
            self.indent_level += 1;
        }
    }

    pub fn dedent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }

    pub fn write_indent(&mut self) {
        if matches!(self.output_format, OutputFormat::Minified) {
            return;
        }
        for _ in 0..self.indent_level {
            self.output.push_str(&self.indent_str);
        }
        if let Some(source_map) = &mut self.source_map {
            source_map.advance(&self.indent_str);
        }
    }

    pub fn write_indented(&mut self, s: &str) {
        self.write_indent();
        self.writeln(s);
    }

    pub fn push_str(&mut self, s: &str) {
        self.output.push_str(s);
    }

    pub fn push_char(&mut self, c: char) {
        self.output.push(c);
    }

    pub fn is_minified(&self) -> bool {
        matches!(self.output_format, OutputFormat::Minified)
    }

    pub fn is_readable(&self) -> bool {
        matches!(self.output_format, OutputFormat::Readable)
    }

    pub fn take_output(&mut self) -> String {
        std::mem::take(&mut self.output)
    }

    pub fn clone_output(&self) -> String {
        self.output.clone()
    }

    pub fn take_source_map(&mut self) -> Option<super::SourceMap> {
        self.source_map.take().map(|builder| builder.build())
    }

    pub fn source_map(&self) -> Option<&super::sourcemap::SourceMapBuilder> {
        self.source_map.as_ref()
    }

    pub fn source_map_mut(&mut self) -> Option<&mut super::sourcemap::SourceMapBuilder> {
        self.source_map.as_mut()
    }

    pub fn clone_source_map(&self) -> Option<super::sourcemap::SourceMapBuilder> {
        self.source_map.clone()
    }

    pub fn output_mut(&mut self) -> &mut String {
        &mut self.output
    }

    pub fn output_ref(&self) -> &String {
        &self.output
    }

    #[cfg(test)]
    pub fn from_output(output: String) -> Self {
        Self {
            output,
            indent_level: 0,
            indent_str: "    ".to_string(),
            source_map: None,
            output_format: OutputFormat::Readable,
        }
    }
}

impl Default for Emitter {
    fn default() -> Self {
        Self::new()
    }
}
