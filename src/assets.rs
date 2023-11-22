use std::path::Path;

use crate::directories::PROJECT_DIRS;
use anyhow::Result;
use syntect::dumps;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

const DEFAULT_SYNTAXSET: &[u8] = include_bytes!("../assets/syntaxes.bin");
const DEFAULT_THEMESET: &[u8] = include_bytes!("../assets/themes.bin");

pub struct HighlightingAssets {
    pub syntax_set: SyntaxSet,
    pub theme_set: ThemeSet,
}

impl Default for HighlightingAssets {
    fn default() -> Self {
        Self::new()
    }
}

impl HighlightingAssets {
    pub fn new() -> Self {
        Self::from_dump_file().unwrap_or_else(|_| Self {
            syntax_set: dumps::from_binary(DEFAULT_SYNTAXSET),
            theme_set: dumps::from_binary(DEFAULT_THEMESET),
        })
    }

    pub fn from_dump_file() -> Result<Self> {
        let cache_dir = PROJECT_DIRS.cache_dir();
        Ok(Self {
            syntax_set: dumps::from_dump_file(cache_dir.join("syntaxes.bin"))?,
            theme_set: dumps::from_dump_file(cache_dir.join("themes.bin"))?,
        })
    }

    pub fn add_from_folder<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref();
        self.theme_set.add_from_folder(path.join("themes"))?;
        let mut builder = self.syntax_set.clone().into_builder();
        builder.add_from_folder(path.join("syntaxes"), true)?;
        self.syntax_set = builder.build();
        Ok(())
    }

    pub fn dump_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        dumps::dump_to_file(&self.syntax_set, path.as_ref().join("syntaxes.bin"))?;
        dumps::dump_to_file(&self.theme_set, path.as_ref().join("themes.bin"))?;
        Ok(())
    }
}
