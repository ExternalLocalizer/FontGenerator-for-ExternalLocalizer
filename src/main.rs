use std::{
    fs::{self, File},
    io::{BufWriter, Write as _},
    path::Path,
};

use anyhow::Context as _;

use crate::fonts::create_font_bundles;

mod fonts;
mod types;
mod wrapper;
mod xml;

#[allow(unused)]
static MOD_SOURCE: &str = r"C:\Users\eva828\Documents\My Games\Terraria\tModLoader\ModSources\";

fn main() -> anyhow::Result<()> {
    // std::env::set_var("RUST_BACKTRACE", "1");

    export_all_fonts()?;

    let dyn_font_dir = Path::new("fonts").join("dynamic");
    let xnb_font_dir = Path::new("fonts").join("xnb");
    // let xnb_font_dir = Path::new(MOD_SOURCE)
    //     .join("ExternalLocalizer")
    //     .join("Assets")
    //     .join("Fonts");

    // フォルダをリセット
    println!("Clearing directories...");
    fs::remove_dir_all(&dyn_font_dir).ok();
    fs::create_dir_all(&dyn_font_dir)?;
    // fs::remove_dir_all(&xnb_font_dir).ok();
    // fs::create_dir_all(&xnb_font_dir)?;

    // DynamicFontBuilderBundleを作成
    let bundles =
        create_font_bundles(&dyn_font_dir).with_context(|| "Failed to create font bundles")?;

    // ビルドしdynamicfontファイルを書き出し
    println!("Generating .dynamicfont files...\n");
    let bundle_directories: Vec<_> = bundles.iter().map(|b| b.directory.clone()).collect();
    for bundle in bundles {
        bundle.build()?;
    }

    // xnbファイルに変換
    println!("Executing DynamicFontGenerator.exe...");
    for dir in bundle_directories {
        wrapper::generate_dynamic_font(&dir)?;
    }

    // .xnbファイルを移動
    println!("Moving .xnb files...");
    let query = dyn_font_dir.join("**/*.xnb").to_string_lossy().to_string();
    for current_path in glob::glob(&query)? {
        let current_path = current_path?;
        let relative_path = current_path.strip_prefix(&dyn_font_dir)?;
        let new_path = xnb_font_dir.join(relative_path);
        println!("{} -> {}", current_path.display(), new_path.display());
        fs::create_dir_all(new_path.parent().with_context(|| "No parent")?)?;
        fs::rename(current_path, new_path)?;
    }

    Ok(())
}

#[allow(unused)]
fn export_all_fonts() -> anyhow::Result<()> {
    let font_system_source = font_kit::source::SystemSource::new();
    let fonts = font_system_source.all_fonts()?;

    let file = File::create("fonts/fonts.yml")?;
    let mut writer = BufWriter::new(file);

    for handle in fonts {
        if let Ok(font) = handle.load() {
            let family_name = font.family_name().to_string();
            let postscript_name = font.postscript_name().unwrap_or_default();
            let full_name = font.full_name();

            writeln!(writer, "- family_name: \"{}\"", family_name)?;
            writeln!(writer, "  postscript_name: \"{}\"", postscript_name)?;
            writeln!(writer, "  full_name: \"{}\"", full_name)?;
            writeln!(writer, "")?;
        }
    }

    Ok(())
}
