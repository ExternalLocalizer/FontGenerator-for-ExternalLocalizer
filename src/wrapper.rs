use std::{
    path::Path,
    process::{Command, Stdio},
};

const DYNAMIC_FONT_GENERATOR_EXE: &str = r"dfg/DynamicFontGenerator.exe";

pub fn generate_dynamic_font(target_dir: &Path) -> anyhow::Result<()> {
    if !Path::new(DYNAMIC_FONT_GENERATOR_EXE).is_file(){
        anyhow::bail!(
            "The DynamicFontGenerator executable could not be found at the specified path: {}\n\
            To proceed, please download the required executable from the link below and ensure it \
            is saved as 'DynamicFontGenerator.exe' inside the 'dfg' folder:\n\
            https://forums.terraria.org/index.php?threads/dynamicspritefontgenerator-0-4-generate-fonts-without-xna-game-studio.57127/",
            DYNAMIC_FONT_GENERATOR_EXE
        );
    }
    
    let output = Command::new(DYNAMIC_FONT_GENERATOR_EXE)
        .current_dir(target_dir)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()?;

    if !output.status.success() {
        anyhow::bail!(
            "DynamicFontGenerator failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}
