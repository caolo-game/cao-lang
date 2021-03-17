#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CompileOptions {
    /// Insert Breadcrumbs into the compiled program.
    /// Default: true
    pub breadcrumbs: bool,
}

impl Default for CompileOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl CompileOptions {
    pub fn new() -> Self {
        Self { breadcrumbs: true }
    }

    pub fn with_breadcrumbs(mut self, breadcrumbs: bool) -> Self {
        self.breadcrumbs = breadcrumbs;
        self
    }
}
