use flapigen::{JavaConfig, LanguageConfig};
use std::{env, path::Path};

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let in_src = Path::new("src").join("java_glue.rs.in");
    let out_src = Path::new(&out_dir).join("java_glue.rs");

    let swig_gen = flapigen::Generator::new(LanguageConfig::JavaConfig(JavaConfig::new(
        Path::new("java")
            .join("src")
            .join("main")
            .join("java")
            .join("army")
            .join("warfare")
            .join("skiter"),
        "army.warfare.skiter".into(),
    )))
    .rustfmt_bindings(true);

    swig_gen.expand("jni bindings", &in_src, &out_src);
    println!("cargo:rerun-if-changed={}", in_src.display());
}
