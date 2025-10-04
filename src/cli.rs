use clap::Parser;

#[derive(Parser)]
#[command(version)]
pub struct Cli {
    /// Exit immediately when there's no match
    #[arg(long = "exit-0", short = '0')]
    pub exit_if_empty: bool,
    /// Exit when the window loses focus
    #[arg(long)]
    pub exit_lost_focus: bool,
    /// Case-insensitive match (default: smart-case match)
    #[arg(short = 'i')]
    pub case_insensitive: bool,
    /// Do not normalize latin script letters before matching
    #[arg(long)]
    pub literal: bool,
    /// Enable cyclic scroll
    #[arg(long)]
    pub cycle: bool,
    /// Offset to start scrolling
    #[arg(long, default_value_t = 2)]
    pub scroll_offset: u32,
    /// Enable multi-select with tab/shift-tab (takes optional limit to the number of matches)
    #[arg(long, short, num_args = 0..=1, default_missing_value = "999999999")]
    pub multi: Option<usize>,
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
    // /// Ellipsis to show when line is truncated
    // #[arg(long, default_value_t = '…')]
    // pub ellipsis: char,
    /// Set font size
    #[arg(long, default_value_t = 16.0)]
    pub font_size: f32,
    /// Set font family (must be monospaced)
    #[arg(long)]
    pub font: Option<String>,
    /// Set window height
    #[arg(long, default_value_t = 510.0)]
    pub window_height: f32,
    /// Set window width
    #[arg(long, default_value_t = 480.0)]
    pub window_width: f32,
}
