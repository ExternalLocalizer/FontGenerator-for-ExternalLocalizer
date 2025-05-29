use std::{
    borrow::Cow,
    fs,
    path::{Path, PathBuf},
};

use anyhow::Context;
use font_kit::source::SystemSource;
use serde::{Serialize, Serializer};

use crate::types::{CharRange, CharRangeList, FontName, FontNameBundle};

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
    font_name_list: Vec<FontName<'static>>,
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

    pub fn add_font_name<'a>(mut self, font_name: FontName<'static>) -> Self {
        self.font_name_list.push(font_name);
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

        let font_system_source = SystemSource::new();

        // 全てのFontNameをFullNameに変換
        let font_name_bundle_list = self
            .font_name_list
            .iter()
            .map(|name| -> anyhow::Result<_> { name.into_bundle(&font_system_source) })
            .collect::<anyhow::Result<Vec<FontNameBundle>>>()?;

        // // FontNameからフォントパスを取得
        // let font_path_list = font_name_bundle_list
        //     .iter()
        //     .map(|name| {
        //         let Ok(handle) = name.get_font_handle(&font_system_source) else {
        //             anyhow::bail!("Font not found: {}", name);
        //         };

        //         let Handle::Path {
        //             path,
        //             font_index: _,
        //         } = handle
        //         else {
        //             anyhow::bail!("Failed to load font: {}", name);
        //         };

        //         let path = path.to_path_buf();
        //         println!("Found font: {}", path.display());
        //         Ok(path)
        //     })
        //     .collect::<anyhow::Result<Vec<_>>>()?;

        // フォントを読み込み、サポートされている文字を取得
        let mut include_chars: Vec<CharRangeList> = Vec::with_capacity(font_name_bundle_list.len());
        for (font_idx, font_name) in font_name_bundle_list.iter().enumerate() {
            let font_path = font_name.path(&font_system_source)?;
            let font_file = font::File::open(font_path)?;

            let mut supported_chars = Vec::new();

            for mut f in font_file.fonts {
                supported_chars.extend(f.characters()?);
            }

            let mut supported_chars = CharRangeList::from(supported_chars);

            for chars in include_chars.iter() {
                supported_chars.subtract_range_list(&chars);
            }
            // null文字等を除外
            supported_chars.subtract_range(&CharRange::new(0, 31));

            include_chars.push(supported_chars);
        }

        // ビルド結果にdefault_characterが含まれない場合に例外
        if include_chars
            .iter()
            .all(|chars| !chars.contains(self.default_character as u32))
        {
            anyhow::bail!("Default character not found in any font.\nYou must include the default character in at least one font.");
        }

        let base_font = font_name_bundle_list
            .first()
            .context("At least one font must be specified")?;

        // CharRangeListをCharacterRegionに変換
        let mut character_regions: CharacterRegions = include_chars
            .into_iter()
            .zip(font_name_bundle_list.iter())
            // FontNameをコピーしながらflatten
            .flat_map(|(vec, count)| vec.into_iter().map(move |item| (item, count)))
            // CharacterRegionに変換
            .map(|(range, font)| {
                CharacterRegion::from_range(range, Some(font.full.to_string()), None, None)
            })
            .collect::<Vec<CharacterRegion>>()
            .into();

        character_regions.ommit_base_font(&base_font);

        Ok(DynamicFont {
            file_name: self.file_name,
            font_name: base_font.full.to_string(),
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

impl CharacterRegions {
    pub fn ommit_base_font(&mut self, base_font: &FontNameBundle) {
        // base_fontのフォント名を持つCharacterRegionのfont_nameをNoneにする
        for region in &mut self.character_region {
            if region.font_name.as_deref() == Some(&base_font.full) {
                region.font_name = None;
            }
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

impl CharacterRegion {
    pub fn from_range(
        range: CharRange,
        font_name: Option<String>,
        size: Option<f32>,
        style: Option<FontStyle>,
    ) -> Self {
        Self {
            font_name,
            size,
            style,
            start: char::from_u32(range.start).unwrap_or_default(),
            end: char::from_u32(range.end).unwrap_or_default(),
        }
    }
}

// impl Into<Vec<CharacterRegion>> for CharRangeList {
//     fn into(self) -> Vec<CharacterRegion> {
//         self.into_iter().map(|range| range.into()).collect()
//     }
// }

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
