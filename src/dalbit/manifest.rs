use indexmap::IndexMap;
use super::polyfill::Polyfill;

/// Manifest for dalbit transpiler. This is a writable manifest.
#[derive(Debug, Clone)]
pub struct Manifest {
    pub minify: bool,
    pub modifiers: IndexMap<String, bool>,
    pub polyfill: Option<Polyfill>,
    pub bundle: bool,
    pub hmr : bool
}

impl Default for Manifest {
    fn default() -> Self {
        Self {
            minify: true,
            modifiers: IndexMap::new(),
            polyfill: Some(Polyfill::default()),
            bundle: false,
            hmr: false
        }
    }
}

impl Manifest {
    #[inline]
    pub fn modifiers(&self) -> &IndexMap<String, bool> {
        &self.modifiers
    }

    #[inline]
    pub fn polyfill(&self) -> &Option<Polyfill> {
        &self.polyfill
    }
}