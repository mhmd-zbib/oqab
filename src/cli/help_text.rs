use console::style;

/// Returns the detailed help text for the application
pub fn get_help_text() -> String {
    let title = style("Oqab Search Utility").bold().cyan().to_string();
    let section_title = |title: &str| style(title).bold().green().to_string();
    let option = |opt: &str| style(opt).yellow().to_string();
    let example = |ex: &str| style(ex).italic().to_string();
    
    format!("{}

{}
oqab [QUERY]
oqab [OPTIONS] [QUERY]
oqab --grep PATTERN [OPTIONS]

{}
{} Display this help message
{} Directory to search in (default: root directory)
{} File extension to search for (e.g., 'rs' or '.rs')
{} Filter by file name pattern
{} Search for text pattern within files (grep-like functionality)
{} Case insensitive search
{} Show line numbers in search results
{} Show only filenames of files containing the pattern
{} Suppress progress output
{} Quiet mode (less verbose output)
{} Number of worker threads (default: CPU cores)
{} Load settings from a configuration file
{} Save current settings to a configuration file

{}
# Simple file search by name (searches from root directory)
{}

# Find all Rust files in current directory
{}
{}

# Find files with 'config' in the filename
{}

# Search for text within files (grep-like functionality)
{}

# Case-insensitive search with line numbers
{}

# Search for text only in specific file types
{}

# Show only filenames containing matches
{}

# Save search settings to a config file
{}

# Use settings from a config file
{}
",
        title,
        section_title("USAGE:"),
        section_title("OPTIONS:"),
        option("-h, --help                  "),
        option("-p, --path <DIR>            "),
        option("-e, --ext <EXT>             "),
        option("-n, --name <PATTERN>        "),
        option("-g, --grep <PATTERN>        "),
        option("-i, --ignore-case          "),
        option("--line-number               "),
        option("--files-with-matches        "),
        option("-s, --silent                "),
        option("-q, --quiet                 "),
        option("-w, --workers <NUM>         "),
        option("-c, --config <FILE>         "),
        option("--save-config <FILE>        "),
        section_title("EXAMPLES:"),
        example("oqab main.rs"),
        example("oqab --path . --ext rs"),
        example("oqab -p . -e rs"),
        example("oqab -p . -n config"),
        example("oqab --grep \"function\" -p ."),
        example("oqab --grep \"error\" --ignore-case --line-number"),
        example("oqab --grep \"import\" --ext py"),
        example("oqab --grep \"TODO\" --files-with-matches"),
        example("oqab -p . -e rs --save-config myconfig.json"),
        example("oqab -c myconfig.json")
    )
} 