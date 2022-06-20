#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CompileOptions {
    /// How deep is the submodule tree allowed to grow?
    pub recursion_limit: u32,
}

impl Default for CompileOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl CompileOptions {
    pub fn new() -> Self {
        Self {
            recursion_limit: 64,
        }
    }
}
