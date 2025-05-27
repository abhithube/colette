use std::{fs, io::Write, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=crates/api/src");

    let openapi = colette_api::create_openapi();
    let raw = openapi.to_yaml()?;

    let crate_path = std::env::var("CARGO_MANIFEST_DIR")?;
    let out_dir = PathBuf::from(format!("{}/../../", crate_path));

    fs::create_dir_all(&out_dir)?;

    let out_path = out_dir.join("openapi.yaml");

    let mut file = fs::File::create(out_path)?;
    file.write_all(raw.as_bytes())?;

    Ok(())
}
