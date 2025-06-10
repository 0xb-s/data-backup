use super::{compression::Compression, exclude::ExcludeList, paths::PathList};

pub fn build_tar_command(
    out_file: &str,
    paths: &PathList,
    excludes: &ExcludeList,
    comp: Compression,
) -> String {
    let mut cmd = String::from("sudo tar");

    match comp {
        Compression::Gzip | Compression::Xz => {
            cmd.push_str(&format!(" -c{}pvf", comp.flag()));
            cmd.push_str(&format!(" '{}'", out_file));
        }
        Compression::Zstd => {
            cmd.push_str(" -cpvf");
            cmd.push_str(&format!(" '{}' {}", comp.flag(), out_file));
        }
    }

    cmd.push_str(" --ignore-failed-read");
    let exclude_str = excludes.to_string();
    if !exclude_str.is_empty() {
        cmd.push(' ');
        cmd.push_str(&exclude_str);
    }

    cmd.push(' ');
    cmd.push_str(&paths.to_string());

    cmd
}
