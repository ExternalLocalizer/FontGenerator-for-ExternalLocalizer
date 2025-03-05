use std::{fs, path::Path};

use anyhow::Context as _;
use xml::{DynamicFontBuilder, DynamicFontBuilderBundle, FontStyle, VerticalOffset};

mod types;
mod wrapper;
mod xml;

static MOD_SOURCE: &str = r"C:\Users\eva828\Documents\My Games\Terraria\tModLoader\ModSources\";

fn main() -> anyhow::Result<()> {
    // std::env::set_var("RUST_BACKTRACE", "1");

    let dyn_font_dir = Path::new("fonts").join("dynamic");
    // let xnb_font_dir = Path::new("fonts").join("xnb");
    let xnb_font_dir = Path::new(MOD_SOURCE)
        .join("ExternalLocalizer")
        .join("Assets")
        .join("Fonts");

    // フォルダをリセット
    println!("Clearing directories...");
    fs::remove_dir_all(&dyn_font_dir).ok();
    fs::create_dir_all(&dyn_font_dir)?;
    // fs::remove_dir_all(&xnb_font_dir).ok();
    // fs::create_dir_all(&xnb_font_dir)?;

    // DynamicFontBuilderBundleを作成
    let mut bundles = Vec::new();
    // bundles.push(terraria_fonts(&dyn_font_dir)?);
    bundles.push(noxusboss_fonts(&dyn_font_dir)?);

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

fn terraria_fonts(base_dir: &Path) -> anyhow::Result<DynamicFontBuilderBundle> {
    let base_font = DynamicFontBuilder::new()
        .add_font_name("なつめもじ抑")
        .add_font_name("Noto Sans JP")
        .use_kerning(false)
        .vertical_offset(VerticalOffset::MaxAscent)
        .spacing(-12f32);

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
            .style(FontStyle::Bold),
    );
    Ok(bundle)
}

fn noxusboss_fonts(base_dir: &Path) -> anyhow::Result<DynamicFontBuilderBundle> {
    let base_font = DynamicFontBuilder::new()
        .use_kerning(true)
        .vertical_offset(VerticalOffset::DefaultFontAscent)
        .spacing(0f32);

    let solyn_font = base_font
        .clone()
        .add_font_name("Noto Serif CJK JP")
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
            .add_font_name("07にくまるフォント")
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
