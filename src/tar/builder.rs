use super::{
    command::build_tar_command,
    compression::Compression,
    exclude::ExcludeList,
    paths::PathList,
    verify::{VerifyMode, list_archive, sha256_file},
};
use std::{fmt, io, path::PathBuf};

#[derive(Debug)]
pub enum TarError {
    EmptyPathList,
    Io(io::Error),
    VerifyFailed(String),
}

impl From<io::Error> for TarError {
    fn from(e: io::Error) -> Self {
        TarError::Io(e)
    }
}

pub struct TarBuilder {
    out_file: String,
    paths: PathList,
    excludes: ExcludeList,
    compression: Compression,
    verify: Option<VerifyMode>,
}

impl TarBuilder {
    pub fn new<S: Into<String>>(out_file: S) -> Self {
        Self {
            out_file: out_file.into(),
            paths: PathList::default(),
            excludes: ExcludeList::default(),
            compression: Compression::Gzip,
            verify: None,
        }
    }

    pub fn paths<I, P>(mut self, iter: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: Into<String>,
    {
        for p in iter {
            self.paths.push_unique(p.into());
        }
        self
    }

    pub fn add_path<P: Into<PathBuf>>(mut self, p: P) -> Self {
        self.paths.push_unique(p);
        self
    }

    pub fn exclude(mut self, p: impl Into<String>) -> Self {
        self.excludes.push_unique(p);
        self
    }

    pub fn exclude_default_runtime(mut self) -> Self {
        self.excludes.add_default_runtime();
        self
    }

    pub fn compression(mut self, c: Compression) -> Self {
        self.compression = c;
        self
    }

    pub fn verify(mut self, mode: VerifyMode) -> Self {
        self.verify = Some(mode);
        self
    }

    pub fn build(&self) -> Result<String, TarError> {
        if self.paths.is_empty() {
            return Err(TarError::EmptyPathList);
        }
        Ok(build_tar_command(
            &self.out_file,
            &self.paths,
            &self.excludes,
            self.compression,
        ))
    }

    pub fn verify_archive(&self) -> Result<(), TarError> {
        if let Some(mode) = self.verify {
            match mode {
                VerifyMode::Sha256 => {
                    sha256_file(&self.out_file)?;
                }
                VerifyMode::ListAndHash => {
                    list_archive(&self.out_file)?;
                    sha256_file(&self.out_file)?;
                }
            }
        }
        Ok(())
    }
}

impl fmt::Display for TarBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.build() {
            Ok(cmd) => write!(f, "{cmd}"),
            Err(_) => write!(f, "<invalid tar builder>"),
        }
    }
}
