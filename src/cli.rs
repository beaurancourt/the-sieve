use crate::renderer::pdf::PageSize;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "the-sieve",
    author = "The Sieve",
    version,
    about = "Convert TTRPG markdown to typeset PDFs (half-letter, digest, letter, A4, A5)",
    long_about = "The Sieve converts markdown documents with TTRPG-specific extensions
(stat blocks, boxed read-aloud text, layout switching) into professionally typeset
PDFs. Supports half-letter (5.5\" x 8.5\"), digest (5.5\" x 8.25\"), letter, A4, and A5 page sizes."
)]
pub struct Args {
    /// Input markdown file
    #[arg(value_name = "INPUT")]
    pub input: PathBuf,

    /// Output PDF file (defaults to input name with .pdf extension)
    #[arg(short, long, value_name = "OUTPUT")]
    pub output: Option<PathBuf>,

    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Output intermediate HTML file instead of PDF
    #[arg(long)]
    pub html_only: bool,

    /// Page size preset
    #[arg(long, value_enum, default_value_t = PageSize::HalfLetter)]
    pub page_size: PageSize,
}

impl Args {
    pub fn output_path(&self) -> PathBuf {
        if let Some(ref output) = self.output {
            output.clone()
        } else {
            let mut path = self.input.clone();
            if self.html_only {
                path.set_extension("html");
            } else {
                path.set_extension("pdf");
            }
            path
        }
    }
}

pub fn parse_args() -> Args {
    Args::parse()
}
