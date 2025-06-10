use std::{fmt, path::PathBuf};

#[derive(Debug, Default, Clone)]
pub struct PathList(Vec<PathBuf>);

impl PathList {
    pub fn push_unique(&mut self, path: impl Into<PathBuf>) {
        let p = path.into();
        if !self.0.iter().any(|x| x == &p) {
            self.0.push(p);
        }
    }
    #[allow(dead_code)]
    pub fn as_slice(&self) -> &[PathBuf] {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn join_for_shell(&self) -> String {
        self.0
            .iter()
            .map(|p| p.to_string_lossy())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

impl From<Vec<String>> for PathList {
    fn from(vec: Vec<String>) -> Self {
        let mut pl = PathList::default();
        for s in vec {
            pl.push_unique(s);
        }
        pl
    }
}

impl fmt::Display for PathList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.join_for_shell())
    }
}
