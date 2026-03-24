use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let po_dir = Path::new(&manifest_dir).join("po");

    if !po_dir.exists() {
        println!("cargo:warning=No 'po' directory found, skipping translation compilation");
        return;
    }

    let mo_dir = Path::new(&out_dir).join("locale");

    for entry in fs::read_dir(&po_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.extension().map(|e| e == "po").unwrap_or(false) {
            let lang = path.file_stem().unwrap().to_str().unwrap().to_string();

            let lang_mo_dir = mo_dir.join(&lang).join("LC_MESSAGES");
            fs::create_dir_all(&lang_mo_dir).unwrap();

            let mo_path = lang_mo_dir.join("riskie.mo");

            let status = Command::new("msgfmt")
                .arg(&path)
                .arg("-o")
                .arg(&mo_path)
                .status()
                .expect("msgfmt not found. Install gettext.");

            if !status.success() {
                panic!("Failed to compile {}", path.display());
            }

            println!("cargo:rerun-if-changed={}", path.display());
        }
    }

    println!("cargo:rerun-if-changed=po/");
}
