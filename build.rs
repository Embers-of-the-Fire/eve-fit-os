use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{env, fs};

use convert_case::{Case, Casing};
use serde::Deserialize;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    dotenvy::dotenv()?;

    #[cfg(feature = "protobuf")]
    {
        build_protobuf()?;
    }

    load_patch_id()?;

    Ok(())
}

#[cfg(feature = "protobuf")]
fn build_protobuf() -> Result<()> {
    println!("cargo::rerun-if-changed=efos.proto");
    prost_build::Config::new().compile_protos(&["efos.proto"], &["."])?;

    Ok(())
}

#[derive(Debug, Deserialize)]
struct GenericNameMapping {
    entries: HashMap<i32, NameObj>,
}

#[derive(Debug, Deserialize)]
struct NameObj {
    name: String,
}

fn load_patch_id() -> Result<()> {
    let dir = env::var("OUTPUT_DIR")?;
    let dir = PathBuf::from(dir).join("json");

    let out = env::var("OUT_DIR")?;
    let out = PathBuf::from(out);

    load_attrs_patches(&dir, &out)?;
    load_effects_patches(&dir, &out)?;

    Ok(())
}

fn load_attrs_patches(dir: &Path, out: &Path) -> Result<()> {
    let attr_file = dir.join("dogmaAttributes.json");
    println!("cargo::rerun-if-changed={}", attr_file.display());

    let attrs = fs::File::open(attr_file)?;
    let attrs: GenericNameMapping = serde_json::from_reader(attrs)?;
    let mut file = fs::File::create(out.join("patch_attrs.rs"))?;
    for (k, v) in attrs.entries {
        if k >= 0 {
            continue;
        }
        writeln!(file, "/// Patch item: `{}` - `{}`", k, v.name)?;
        writeln!(
            file,
            "pub const ATTR_{}: i32 = {};\n",
            v.name.to_case(Case::Constant),
            k
        )?;
    }
    file.sync_data()?;

    Ok(())
}

fn load_effects_patches(dir: &Path, out: &Path) -> Result<()> {
    let effect_file = dir.join("dogmaEffects.json");
    println!("cargo::rerun-if-changed={}", effect_file.display());

    let effects = fs::File::open(effect_file)?;
    let effects: GenericNameMapping = serde_json::from_reader(effects)?;
    let mut file = fs::File::create(out.join("patch_effects.rs"))?;
    for (k, v) in effects.entries {
        if k >= 0 {
            continue;
        }
        writeln!(file, "/// Patch item: `{}` - `{}`", k, v.name)?;
        writeln!(
            file,
            "pub const EFFECT_{}: i32 = {};\n",
            v.name.to_case(Case::Constant),
            k
        )?;
    }
    file.sync_data()?;

    Ok(())
}
