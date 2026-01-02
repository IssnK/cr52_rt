use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    // Get the output directory from the environment variable
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    fs::copy("link.ld", out_dir.join("link.ld")).unwrap();
    println!("carg:rustc-link-search={}", out_dir.display());

    // Assemble boot code for cr52_rt
    cc::Build::new()
        .file("boot.s")
        .flag("-march=armv8-r")
        .flag("-mcpu=cortex-r52")
        .flag("-mfpu=vfpv3-d16")
        .flag("-mfloat-abi=hard")
        .compile("boot");

    println!("cargo:rerun-if-changed=boot.s");
    println!("cargo:rerun-if-changed=link.ld");
}
