#[derive(Debug, Clone)]
pub struct SearchResult {
    pub file_path: String,
    pub column: u32,
    pub row: u32,
}
