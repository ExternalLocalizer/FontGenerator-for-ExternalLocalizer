use std::{
    path::Path,
    process::{Command, Stdio},
};

const DYNAMIC_FONT_GENERATOR_EXE: &str =
    r"C:\Users\eva828\Downloads\DFG_0.6\DynamicFontGenerator.exe";

pub fn generate_dynamic_font(target_dir: &Path) -> anyhow::Result<()> {
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
