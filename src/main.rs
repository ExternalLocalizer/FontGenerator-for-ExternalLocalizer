use std::{
    fs::{self, File},
    io::{BufWriter, Write as _},
    path::Path,
};

use anyhow::Context as _;
use types::FontName;
use xml::{DynamicFontBuilder, DynamicFontBuilderBundle, FontStyle, VerticalOffset};

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
    let mut bundles = Vec::new();
    bundles.push(terraria_fonts(&dyn_font_dir)?);
    // bundles.push(noxusboss_fonts(&dyn_font_dir)?);
    // bundles.push(terratcg_fonts(&dyn_font_dir)?);

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

            // writeln!(writer, "Font: {font_name}")?;
            // writeln!(writer, "\tPostscript Name: {postscript_name}")?;
            // writeln!(writer, "\tFull Name: {full_name}")?;
            writeln!(writer, "- family_name: \"{}\"", family_name)?;
            writeln!(writer, "  postscript_name: \"{}\"", postscript_name)?;
            writeln!(writer, "  full_name: \"{}\"", full_name)?;
            writeln!(writer, "")?;
        }
    }

    Ok(())
}

#[allow(unused)]
fn terraria_fonts(base_dir: &Path) -> anyhow::Result<DynamicFontBuilderBundle> {
    let base_font = DynamicFontBuilder::new()
        .add_font_name(FontName::postscript("YOzCb"))
        .add_font_name(FontName::postscript("NotoSansJP-Regular"))
        .use_kerning(true)
        .vertical_offset(VerticalOffset::DefaultFontAscent)
        .size(12f32)
        .style(FontStyle::Bold)
        .spacing(0f32);

    let mut bundle = DynamicFontBuilderBundle::new(base_dir.join("terraria"));
    bundle.add_font(base_font.clone().file_name("Combat_Text"));
    bundle.add_font(base_font.clone().file_name("Combat_Crit"));
    bundle.add_font(base_font.clone().file_name("Item_Stack"));
    bundle.add_font(
        base_font
            .clone()
            .file_name("Mouse_Text")
            .style(FontStyle::Bold),
    );
    bundle.add_font(
        base_font
            .clone()
            .file_name("Death_Text")
            .size(24f32)
            .style(FontStyle::Bold),
    );
    Ok(bundle)
}

#[allow(unused)]
fn noxusboss_fonts(base_dir: &Path) -> anyhow::Result<DynamicFontBuilderBundle> {
    let base_font = DynamicFontBuilder::new()
        .use_kerning(true)
        .vertical_offset(VerticalOffset::DefaultFontAscent)
        .spacing(0f32);

    let solyn_font = base_font
        .clone()
        .add_font_name(FontName::family("Noto Serif CJK JP"))
        .size(28f32);

    let mut bundle = DynamicFontBuilderBundle::new(base_dir.join("WrathOfTheGods"));
    bundle.add_font(
        solyn_font
            .clone()
            .file_name("SolynText")
            .style(FontStyle::Regular),
    );
    bundle.add_font(
        solyn_font
            .clone()
            .file_name("SolynTextItalics")
            .style(FontStyle::Italic),
    );
    bundle.add_font(
        base_font
            .clone()
            .file_name("SolynFightDialogue")
            .add_font_name(FontName::family("07にくまるフォント"))
            .size(32f32),
    );
    // bundle.add_font(
    //     base_font
    //         .clone()
    //         .file_name("DraedonText")
    //         .add_font_name("ノスタルドット（M+）")
    //         // .add_font_name("x12y12pxMaruMinya")
    //         // .vertical_offset(VerticalOffset::MaxAscent)
    //         .use_kerning(false)
    //         .size(20f32),
    // );

    Ok(bundle)
}

#[allow(unused)]
fn terratcg_fonts(base_dir: &Path) -> anyhow::Result<DynamicFontBuilderBundle> {
    let base_font = DynamicFontBuilder::new()
        .add_font_name(FontName::family("なつめもじ抑"))
        .add_font_name(FontName::family("Noto Sans JP"))
        .use_kerning(false)
        // .vertical_offset(VerticalOffset::MaxAscent);
        .vertical_offset(VerticalOffset::DefaultFontAscent);

    let mut bundle = DynamicFontBuilderBundle::new(base_dir.join("TerraTCG"));
    bundle.add_font(
        base_font
            .clone()
            .file_name("SmallText")
            .size(15f32)
            .spacing(0.25f32),
    );

    Ok(bundle)
}
