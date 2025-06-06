use std::path::Path;

use crate::{
    types::FontName,
    xml::{DynamicFontBuilder, DynamicFontBuilderBundle, FontStyle, VerticalOffset},
};

pub fn create_font_bundles(base_dir: &Path) -> anyhow::Result<Vec<DynamicFontBuilderBundle>> {
    let mut bundles = Vec::new();
    bundles.push(terraria_fonts(base_dir)?);
    bundles.push(noxusboss_fonts(base_dir)?);
    bundles.push(terratcg_fonts(base_dir)?);
    Ok(bundles)
}

#[allow(unused)]
fn terraria_fonts(base_dir: &Path) -> anyhow::Result<DynamicFontBuilderBundle> {
    let base_font = DynamicFontBuilder::new()
        .add_font_name(FontName::full("YOzCbBlack"))
        .use_kerning(true)
        .vertical_offset(VerticalOffset::DefaultFontAscent)
        .size(12f32)
        .spacing(0f32);

    let mut bundle = DynamicFontBuilderBundle::new(base_dir.join("terraria"));
    bundle.add_font(base_font.clone().file_name("Medium_Text")); // Combat_Text, Combat_Crit, Item_Stack, Mouse_Text
    bundle.add_font(base_font.clone().file_name("Large_Text").size(24f32)); // Death_Text
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
    Ok(bundle)
}

#[allow(unused)]
fn terratcg_fonts(base_dir: &Path) -> anyhow::Result<DynamicFontBuilderBundle> {
    let base_font = DynamicFontBuilder::new()
        .add_font_name(FontName::family("YOzCbBlack"))
        .add_font_name(FontName::family("Noto Sans JP"))
        .use_kerning(true)
        .vertical_offset(VerticalOffset::DefaultFontAscent);

    let mut bundle = DynamicFontBuilderBundle::new(base_dir.join("TerraTCG"));
    bundle.add_font(
        base_font
            .clone()
            .file_name("SmallText")
            .size(12f32)
            .spacing(0f32),
    );

    Ok(bundle)
}
