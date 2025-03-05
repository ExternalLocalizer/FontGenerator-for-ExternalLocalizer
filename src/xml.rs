use std::{
    borrow::Cow,
    fs,
    path::{self, Path, PathBuf},
};

use font_kit::{handle::Handle, source::SystemSource};
use serde::{Serialize, Serializer};

use crate::types::{CharRange, CharRangeList};

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct XnaContent {
    #[serde(skip)]
    pub file_name: String,
    #[serde(rename = "@xmlns:Graphics")]
    pipeline: String,
    pub asset: XnaAsset,
}

impl XnaContent {
    fn to_xml(&self) -> String {
        let mut buffer = String::from("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");

        let mut ser = quick_xml::se::Serializer::new(&mut buffer);
        ser.indent(' ', 2);
        self.serialize(ser).unwrap();

        buffer = buffer.replace("&amp;", "&");

        buffer
    }

    fn write(&self, directory: &Path) -> anyhow::Result<PathBuf> {
        let content = self.to_xml();
        fs::create_dir_all(directory)?;
        let path = directory
            .join(&self.file_name)
            .with_extension("dynamicfont");
        std::fs::write(&path, &content)?;
        Ok(path)
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct XnaAsset {
    #[serde(rename = "@Type")]
    r#type: String,
    #[serde(flatten)]
    pub font: DynamicFont,
}

pub struct DynamicFontBuilderBundle {
    pub directory: PathBuf,
    pub fonts: Vec<DynamicFontBuilder>,
}

impl DynamicFontBuilderBundle {
    pub fn new<T: Into<PathBuf>>(directory: T) -> Self {
        Self {
            directory: directory.into(),
            fonts: Vec::new(),
        }
    }

    pub fn add_font(&mut self, font: DynamicFontBuilder) {
        self.fonts.push(font);
    }

    pub fn build(self) -> anyhow::Result<Vec<PathBuf>> {
        let fonts = self
            .fonts
            .into_iter()
            .map(|builder| builder.build()?.pack().write(&self.directory))
            .collect::<anyhow::Result<Vec<PathBuf>>>()?;
        Ok(fonts)
    }
}

#[derive(Clone)]
pub struct DynamicFontBuilder {
    file_name: String,
    font_name_list: Vec<String>,
    size: f32,
    spacing: f32,
    use_kerning: bool,
    style: FontStyle,
    default_character: char,
    vertical_offset: VerticalOffset,
}

impl DynamicFontBuilder {
    pub fn new() -> Self {
        Self {
            file_name: String::new(),
            font_name_list: Vec::new(),
            size: 16.0,
            spacing: 0.0,
            use_kerning: true,
            style: FontStyle::Regular,
            default_character: '*',
            vertical_offset: VerticalOffset::DefaultFontAscent,
        }
    }

    pub fn add_font_name<'a, T: Into<Cow<'a, str>>>(mut self, font_name: T) -> Self {
        self.font_name_list.push(font_name.into().to_string());
        self
    }

    #[allow(unused)]
    pub fn file_name<T: Into<Cow<'static, str>>>(mut self, file_name: T) -> Self {
        self.file_name = file_name.into().to_string();
        self
    }

    #[allow(unused)]
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    #[allow(unused)]
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    #[allow(unused)]
    pub fn use_kerning(mut self, use_kerning: bool) -> Self {
        self.use_kerning = use_kerning;
        self
    }

    #[allow(unused)]
    pub fn style(mut self, style: FontStyle) -> Self {
        self.style = style;
        self
    }

    #[allow(unused)]
    pub fn default_character(mut self, default_character: char) -> Self {
        self.default_character = default_character;
        self
    }

    #[allow(unused)]
    pub fn vertical_offset(mut self, vertical_offset: VerticalOffset) -> Self {
        self.vertical_offset = vertical_offset;
        self
    }

    pub fn build(self) -> anyhow::Result<DynamicFont> {
        if self.font_name_list.is_empty() {
            anyhow::bail!("No font names specified");
        }

        let base_font_name = self.font_name_list[0].to_string();

        let font_system_source = SystemSource::new();

        let font_path_list = self
            .font_name_list
            .iter()
            .map(|name| {
                let Ok(font) = font_system_source.select_family_by_name(name) else {
                    anyhow::bail!("Font not found: {}", name);
                };

                let Some(font) = font.fonts().first() else {
                    anyhow::bail!("Failed to load font: {}", name);
                };

                let Handle::Path {
                    path,
                    font_index: _,
                } = font
                else {
                    anyhow::bail!("Failed to load font: {}", name);
                };

                let path = path.to_path_buf();
                println!("Found font: {}", path.display());
                Ok(path.to_string_lossy().to_string())
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        let mut include_chars: Vec<(usize, CharRangeList)> =
            Vec::with_capacity(font_path_list.len());

        for (font_idx, font_path) in font_path_list.iter().enumerate() {
            let font = font::File::open(font_path)?;

            let mut supported_chars = Vec::new();

            for mut f in font.fonts {
                supported_chars.extend(f.characters()?);
            }

            let mut supported_chars = CharRangeList::from(supported_chars);

            for (_, chars) in include_chars.iter() {
                supported_chars.subtract_range_list(&chars);
            }
            // null文字等を除外
            supported_chars.subtract_range(&CharRange::new(0, 32));

            include_chars.push((font_idx, supported_chars));
        }

        // TODO: ビルド結果にdefault_characterが含まれない時にエラーを出すべき
        if include_chars
            .iter()
            .all(|(_, chars)| !chars.contains(self.default_character as u32))
        {
            anyhow::bail!("Default character not found in any font.\nYou must include the default character in at least one font.");
        }

        let character_regions = include_chars
            .into_iter()
            .flat_map(|(font_idx, chars)| {
                let mut chars: Vec<CharacterRegion> = chars.into();
                if font_idx != 0 {
                    chars.iter_mut().for_each(|region| {
                        region.font_name = Some(self.font_name_list[font_idx].to_string());
                    });
                }
                chars
            })
            .map(|region| region.into())
            .collect::<Vec<_>>()
            .into();

        Ok(DynamicFont {
            file_name: self.file_name,
            font_name: base_font_name,
            size: self.size,
            spacing: self.spacing,
            use_kerning: self.use_kerning,
            style: self.style,
            default_character: self.default_character,
            vertical_offset: self.vertical_offset,
            character_regions,
        })
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct DynamicFont {
    #[serde(skip)]
    pub file_name: String,
    pub font_name: String,
    pub size: f32,
    pub spacing: f32,
    pub use_kerning: bool,
    pub style: FontStyle,
    pub default_character: char,
    pub vertical_offset: VerticalOffset,
    pub character_regions: CharacterRegions,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CharacterRegions {
    pub character_region: Vec<CharacterRegion>,
}

impl From<Vec<CharacterRegion>> for CharacterRegions {
    fn from(vec: Vec<CharacterRegion>) -> Self {
        Self {
            character_region: vec,
        }
    }
}

impl DynamicFont {
    pub fn pack(self) -> XnaContent {
        XnaContent {
            file_name: self.file_name.clone(),
            pipeline: "ReLogic.Content.Pipeline".to_string(),
            asset: XnaAsset {
                r#type: "Graphics:DynamicFontDescription".to_string(),
                font: self,
            },
        }
    }
}

#[derive(Debug, Serialize, PartialEq, Clone, Copy)]
#[allow(unused)]
pub enum FontStyle {
    Regular,
    Bold,
    Italic,
    BoldItalic,
}

#[derive(Debug, Serialize, PartialEq, Clone, Copy)]
#[allow(unused)]
pub enum VerticalOffset {
    DefaultFontAscent,
    MaxAscent,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CharacterRegion {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<FontStyle>,
    #[serde(serialize_with = "serialize_char_as_xml_reference")]
    pub start: char,
    #[serde(serialize_with = "serialize_char_as_xml_reference")]
    pub end: char,
}

impl CharacterRegion {
    #[allow(unused)]
    pub fn new(start: char, end: char) -> Self {
        Self {
            font_name: None,
            size: None,
            style: None,
            start,
            end,
        }
    }
}

fn serialize_char_as_xml_reference<S>(c: &char, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let xml_reference = format!("&#x{:X};", *c as u32);
    // let xml_reference = c.escape_unicode().to_string();
    // let xml_reference = c.to_string();

    serializer.serialize_str(&xml_reference)
}

impl From<CharRange> for CharacterRegion {
    fn from(range: CharRange) -> Self {
        Self {
            font_name: None,
            size: None,
            style: None,
            start: char::from_u32(range.start).unwrap_or_default(),
            end: char::from_u32(range.end).unwrap_or_default(),
        }
    }
}

impl Into<Vec<CharacterRegion>> for CharRangeList {
    fn into(self) -> Vec<CharacterRegion> {
        self.into_iter().map(|range| range.into()).collect()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_xml() {
        let test_region1 = CharacterRegion::new('a', 'z');
        let test_region2 = CharacterRegion::new('A', 'Z');
        let test_vec = vec![test_region1, test_region2];

        let test_dynamic_font = DynamicFont {
            file_name: "test".to_string(),
            font_name: "Arial".to_string(),
            size: 16.0,
            spacing: 0.0,
            use_kerning: true,
            style: FontStyle::Regular,
            default_character: '*',
            vertical_offset: VerticalOffset::DefaultFontAscent,
            character_regions: test_vec.into(),
        };

        let mut buffer = String::from("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");
        let mut ser = quick_xml::se::Serializer::new(&mut buffer);
        ser.indent(' ', 2);
        test_dynamic_font.pack().serialize(ser).unwrap();
        println!("{}", buffer);
    }
}
