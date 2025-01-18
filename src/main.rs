use std::{fs, path::Path, vec};

use anyhow::Context;
use xml::DynamicFontBuilder;

mod types;
mod wrapper;
mod xml;

fn main() -> anyhow::Result<()> {
    println!("Generating .dynamicfont files...");
    let basic_font = DynamicFontBuilder::new()
        .add_font_name("なつめもじ抑".to_string())
        .add_font_name("Noto Sans JP".to_string())
        .build()?
        .pack()
        .to_xml();

    let death_font = DynamicFontBuilder::new()
        .add_font_name("なつめもじ抑".to_string())
        .add_font_name("Noto Sans JP".to_string())
        .size(24.0)
        .build()?
        .pack()
        .to_xml();

    let files = vec![
        ("Combat_Crit", &basic_font),
        ("Combat_Text", &basic_font),
        ("Death_Text", &death_font),
        ("Item_Stack", &basic_font),
        ("Mouse_Text", &basic_font),
    ];

    let dyn_font_dir = Path::new("fonts/dynamic");
    fs::create_dir_all(dyn_font_dir)?;
    for (file_name, font) in files {
        let file = dyn_font_dir.join(file_name).with_extension("dynamicfont");
        fs::write(file, font)?;
    }

    println!("Executing DynamicFontGenerator.exe...");
    wrapper::generate_dynamic_font(dyn_font_dir)?;

    println!("Moving .xnb files...");
    let xnb_font_dir = Path::new("fonts/xnb");
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
