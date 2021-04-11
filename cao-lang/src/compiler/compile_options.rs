#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CompileOptions {}

impl Default for CompileOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl CompileOptions {
    pub fn new() -> Self {
        Self {}
    }
}
