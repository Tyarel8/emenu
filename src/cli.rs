use clap::Parser;

#[derive(Parser)]
pub struct Cli {
    /// Exit immediately when there's no match
    #[arg(long = "exit-0", short = '0')]
    pub exit_if_empty: bool,
    // /// Enable multi-select with tab/shift-tab
    // #[arg(long, short)]
    // pub multi: Option<usize>,
    /// Input prompt
    #[arg(long, default_value_t = String::from(""))]
    pub prompt: String,
    /// Pointer to the current line
    #[arg(long, default_value_t = String::from("→"))]
    pub pointer: String,
    /// Multi-select marker
    #[arg(long, default_value_t = String::from(">"))]
    pub marker: String,
    // /// Reverse the order of the input
    // #[arg(long)]
    // pub tac: bool,
    /// Ellipsis to show when line is truncated
    #[arg(long, default_value_t = '…')]
    pub ellipsis: char,
}
