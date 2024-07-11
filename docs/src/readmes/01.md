# チュートリアル 01 - 永久にウエイト

## tl;dr

- プロジェクトスケルトンを設定する。
- カーネルコードを実行しているすべてのCPUを停止するだけの小さなアセンブリコードを実行する。

## ビルド

- `Makefile`ターゲット:
    - `doc`: ドキュメントを生成する。
    - `qemu`: `kernel`をQEMUで実行する。
    - `clippy`
    - `clean`
    - `readelf`: `ELF`出力を見る。
    - `objdump`: アセンブリコードを見る。
    - `nm`: シンボルを見る。

## 見るべきコード

- `BSP`-固有の`link.ld` リンカスクリプト。
    - ロードアドレスは`0x8_0000`
    - `.text`セクションのみ
- `main.rs`: 重要な [内部属性]:
    - `#![no_std]`, `#![no_main]`
- `boot.s`: `_start()`を実行しているすべてのコアを停止する`wfe` (Wait For Event) を実行するアセンブリ関数 `_start()`
- コンパイラを満足させるために`#[panic_handler]`関数を定義する（必要がある）。
    - 使われないため削除されるだろうから、`unimplemented!()`とする。

[内部属性]: https://doc.rust-lang.org/reference/attributes.html

### テストする

プロジェクトフォルダで、QEMUを起動して、CPUコアが`wfe`でスピンしているのを観察する。

```console
$ make qemu
[...]
IN:
0x00080000:  d503205f  wfe
0x00080004:  17ffffff  b        #0x80000
```
