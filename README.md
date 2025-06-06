# FontGenerator for ExternalLocalizer

ExternalLocalizerで使用しているXNBフォントを生成するツール

フォントを指定すると、対応している文字全てを含むxnbフォントが生成されます。

## Requirement

### 1. DynamicSpriteFontGenerator

ダウンロード: [TerrariaForums](https://forums.terraria.org/index.php?threads/dynamicspritefontgenerator-0-4-generate-fonts-without-xna-game-studio.57127/)

`.dynamicfont`から`.xnb`フォントを自動生成するツール。

ダウンロードして`dfg/DynamicFontGenerator.exe`に展開してください。

### 2. Noto Serif Japanese & Noto Sans Japanese

ダウンロード: [GoogleFonts](https://fonts.google.com/share?selection.family=Noto+Sans+JP:wght@100..900|Noto+Serif+JP:wght@200..900)

ExternalLocalizerのフォールバックフォントやNoxusBossのフォントに使用しています。

### 3. にくまるフォント

ダウンロード: [BOOTH](https://flopdesign.booth.pm/items/4571432)

NoxusBossのフォントに使用しています。

## Usage

### 1. TTFフォントの生成

ExternalLocalizerで使用している`Y.Oz`フォントは、
[FontForge](https://fontforge.org/)を用いて輪郭の拡大を行っています。

以下で実行できます。

```sh
cd fontforge
.\boldify.ps1
```

生成された`fontforge\ttf\YOzBC_Black.ttf`をインストールしてください。

### 2. XNBフォントへの変換

以下で実行できます。

```sh
cargo run
```

完了すると`fonts/xnb/..`にXNBフォントが生成されているはずです。

> [!TIP]
> `src/fonts.rs` にフォントの定義が書いてあります。
> このファイルを編集すれば独自のフォントを作成できます。
