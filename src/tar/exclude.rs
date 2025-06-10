use std::fmt;

#[derive(Debug, Default, Clone)]
pub struct ExcludeList(Vec<String>);

impl ExcludeList {
    pub fn push_unique(&mut self, rule: impl Into<String>) {
        let r = rule.into();
        if !self.0.contains(&r) {
            self.0.push(r);
        }
    }

    pub fn add_default_runtime(&mut self) {
        for r in [
            "/proc",
            "/sys",
            "/dev",
            "/run",
            "/tmp",
            "/mnt",
            "/media",
            "/var/run",
            "/var/lock",
            "/lost+found",
        ] {
            self.push_unique(r);
        }
    }

    pub fn join_for_shell(&self) -> String {
        self.0
            .iter()
            .map(|e| format!("--exclude={}", e))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

impl fmt::Display for ExcludeList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.join_for_shell())
    }
}
