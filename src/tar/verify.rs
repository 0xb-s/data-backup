use sha2::{Digest, Sha256};
use std::{
    fmt,
    io::{self, Read},
    process::Command,
};

#[derive(Debug, Clone, Copy)]
pub enum VerifyMode {
    Sha256,
    ListAndHash,
}

impl fmt::Display for VerifyMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            VerifyMode::Sha256 => "sha256",
            VerifyMode::ListAndHash => "list+sha256",
        };
        write!(f, "{s}")
    }
}

pub fn sha256_file<P: AsRef<std::path::Path>>(path: P) -> io::Result<String> {
    let mut file = std::fs::File::open(path)?;
    let mut h = Sha256::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
    }
    Ok(hex::encode(h.finalize()))
}

pub fn list_archive(path: &str) -> io::Result<()> {
    let status = Command::new("tar").arg("-tf").arg(path).status()?;
    if !status.success() {
        return Err(io::Error::new(io::ErrorKind::Other, "tar -t failed"));
    }
    Ok(())
}
