use std::{collections::HashMap, fs, path::Path};

use anyhow::Context;
use xml::{DynamicFontBuilder, FontStyle, VerticalOffset};

mod types;
mod wrapper;
mod xml;

fn main() -> anyhow::Result<()> {
    println!("Generating .dynamicfont files...\n");

    let mut fonts = HashMap::new();

    let base_font = DynamicFontBuilder::new()
        .add_font_name("なつめもじ抑")
        .add_font_name("Noto Sans JP")
        .use_kerning(false)
        .vertical_offset(VerticalOffset::MaxAscent)
        .spacing(-12f32);

    fonts.insert("Combat_Text", base_font.clone());
    fonts.insert("Combat_Crit", base_font.clone());
    fonts.insert("Item_Stack", base_font.clone().style(FontStyle::Bold));
    fonts.insert("Mouse_Text", base_font.clone().style(FontStyle::Bold));
    fonts.insert("Death_Text", base_font.clone().style(FontStyle::Bold));

    let fonts = fonts
        .into_iter()
        .map(|(key, value)| {
            let value = value.build()?.pack().to_xml();
            Ok((key.to_string(), value))
        })
        .collect::<anyhow::Result<HashMap<String, String>>>()?;

    let dyn_font_dir = Path::new("fonts/dynamic");
    fs::create_dir_all(dyn_font_dir)?;
    for (file_name, font) in fonts {
        let file = dyn_font_dir.join(file_name).with_extension("dynamicfont");
        fs::write(file, font)?;
    }

    println!("Executing DynamicFontGenerator.exe...");
    wrapper::generate_dynamic_font(dyn_font_dir)?;

    println!("Moving .xnb files...");
    let xnb_font_dir = Path::new(
        r"C:\Users\eva828\Documents\My Games\Terraria\tModLoader\ModSources\ExternalLocalizer\Fonts",
    );

    fs::create_dir_all(xnb_font_dir)?;
    for file in dyn_font_dir.read_dir()? {
        let file = file?.path();

        if file.extension() != Some("xnb".as_ref()) {
            continue;
        }

        let new_file = xnb_font_dir.join(file.file_name().with_context(|| "No file name")?);
        fs::rename(file, new_file)?;
    }

    Ok(())
}
