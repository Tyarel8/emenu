use clap::Parser;

#[derive(Parser)]
pub struct Cli {
    #[arg(long = "exit-0", short = '0')]
    /// Exit immediately when there's no match
    pub exit_if_empty: bool,
    #[arg(long, short)]
    /// Enable multi-select with tab/shift-tab
    pub multi: Option<usize>,
    #[arg(long, default_value_t = String::from(""))]
    /// Input prompt
    pub prompt: String,
    #[arg(long, default_value_t = String::from(">"))]
    /// Pointer to the current line
    pub pointer: String,
    #[arg(long, default_value_t = String::from(">"))]
    /// Multi-select marker
    pub marker: String,
    #[arg(long)]
    /// Reverse the order of the input
    pub tac: bool,
    #[arg(long, default_value_t = '…')]
    /// Ellipsis to show when line is truncated
    pub ellipsis: char,
}
