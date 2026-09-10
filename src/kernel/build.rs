fn main() {
    let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    println!("cargo:rustc-link-arg=-T{dir}/src/arch/x86_64/linker.ld");
    println!("cargo:rustc-link-arg=-no-pie");
    println!("cargo:rerun-if-changed=src/arch/x86_64/linker.ld");
    println!("cargo:rustc-link-arg=--orphan-handling=error");
}
