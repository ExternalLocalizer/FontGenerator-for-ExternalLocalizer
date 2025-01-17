mod types;

use types::{CharRangeList, Counter as _};

fn main() -> anyhow::Result<()> {
    // Windowsシステムにあるフォントファイル（例: Arial のフォントファイルのパス）
    // let font_path = r"C:\Users\eva828\AppData\Local\Microsoft\Windows\Fonts\natumemozi-o.ttf";
    // let font_path =
    //     r"C:\Users\eva828\AppData\Local\Microsoft\Windows\Fonts\NotoSansJP-VariableFont_wght.ttf";

    let font_path_list = [
        r"C:\Users\eva828\AppData\Local\Microsoft\Windows\Fonts\natumemozi-o.ttf",
        r"C:\Users\eva828\AppData\Local\Microsoft\Windows\Fonts\NotoSansJP-VariableFont_wght.ttf",
    ];

    let mut include_chars = Vec::with_capacity(font_path_list.len());

    for font_path in font_path_list.iter() {
        // フォントファイルを読み込む
        let font = font::File::open(font_path)?;

        // フォントがサポートするグリフ（文字）を列挙する
        let mut supported_chars = Vec::new();

        for mut f in font.fonts {
            supported_chars.extend(f.characters()?);
        }

        let mut supported_chars = CharRangeList::from(supported_chars);

        for chars in include_chars.iter() {
            supported_chars.subtract_range_list(&chars);
        }

        // // サポートされているグリフを表示する
        // for c in supported_chars.iter() {
        //     println!("{}", c);
        // }

        include_chars.push(supported_chars);
    }

    for (i, chars) in include_chars.iter().enumerate() {
        println!("Font: {}", font_path_list[i]);
        println!("\tCount: {}", chars.count());
    }

    Ok(())
}
