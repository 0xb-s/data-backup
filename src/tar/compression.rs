use std::fmt;
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum Compression {
    Gzip,
    Xz,
    Zstd,
}

impl Compression {
    pub fn flag(self) -> &'static str {
        match self {
            Compression::Gzip => "z", // -z
            Compression::Xz => "J",   // -J
            Compression::Zstd => "--zstd",
        }
    }
}
impl fmt::Display for Compression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Compression::Gzip => "gzip",
            Compression::Xz => "xz",
            Compression::Zstd => "zstd",
        };
        write!(f, "{s}")
    }
}
