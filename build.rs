use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("const_build.rs");
    let now = match Command::new("date")
        .arg("-u")
        .arg("+%Y-%m-%d %H:%M:%S")
        .output()
    {
        Ok(n) => String::from_utf8(n.stdout).unwrap(),
        Err(_) => String::from("undefined"),
    };

    let git = match Command::new("git")
        .arg("rev-parse")
        .arg("--short")
        .arg("HEAD")
        .output()
    {
        Ok(u) => String::from_utf8(u.stdout).unwrap(),
        Err(_) => env::var("GIT_REV").unwrap_or(String::from("HEAD")),
    };

    fs::write(
        &dest_path,
        format!(
            "pub const BUILDTIME :&str = \"{now}\";
         pub const GIT_REV :&str = \"{git}\";
        "
        ),
    )
    .unwrap_or_else(|_| println!("cargo::error=Cannot write const_build.rs"));
    println!("cargo::rerun-if-changed=build.rs");
}
