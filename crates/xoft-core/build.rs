use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    for name in ["oberon2", "oberon-x"] {
        let grammar_dir = manifest_dir.join("../../grammars").join(format!("tree-sitter-{name}"));
        let gen_src = grammar_dir.join("gen-src");
        let src = grammar_dir.join("src");

        cc::Build::new()
            .include(&gen_src)
            .file(gen_src.join("parser.c"))
            .file(src.join("scanner.c"))
            .flag_if_supported("-Wno-unused-parameter")
            .compile(&format!("tree-sitter-{name}"));

        println!("cargo:rerun-if-changed={}", gen_src.join("parser.c").display());
        println!("cargo:rerun-if-changed={}", src.join("scanner.c").display());
    }
}
