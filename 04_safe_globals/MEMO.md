# コードリーディング

## `synchronization.rs`

```rust
use core::cell::UnsafeCell;

pub mod interface {     // interface::Mutexトレイトのlock関数は
    pub trait Mutex {   // クロージャFnOnceにDataへの可変アクセスが保証する
        type Data;
        fn lock<R>(&self, f: impl FnOnce(&mut Self::Data) -> R) -> R;
    }
}

pub struct NullLock<T> where T: ?Sized, { // TにはSizedトレイト制約なし
    data: UnsafeCell<T>,
}

// Send: 型Tの所有権ははスレッド間で転送可能
unsafe impl<T> Send for NullLock<T> where T: ?Sized + Send {}
// Sync: 型Tは複数のスレッドから参照されても安全
unsafe impl<T> Sync for NullLock<T> where T: ?Sized + Send {}

impl<T> NullLock<T> {
    pub const fn new(data: T) -> Self {
        Self {
            data: UnsafeCell::new(data),
        }
    }
}

impl<T> interface::Mutex for NullLock<T> {
    type Data = T;
    // lock関数という名前だがこの実装ではdataをロックしていない
    fn lock<R>(&self, f: impl FnOnce(&mut Self::Data) -> R) -> R {
        let data = unsafe { &mut *self.data.get() };  # dataへのポインタを返す
        f(data) # 渡された関数を実行する
    }
}
```

## `const fn`: fnはconst文脈で呼び出せる

```rust
// このstaticなグローバル変数を作成するために関連するnew()はすべてconst fn
static QEMU_OUTPUT: QEMUOutput = QEMUOutput::new();

impl QEMUOutput {
    pub const fn new() -> QEMUOutput {  // const fn new()
        QEMUOutput {
            inner: NullLock::new(QEMUOutputInner::new()),
        }
    }
}

impl<T> NullLock<T> {
    pub const fn new(data: T) -> Self   // const fn new()

impl QEMUOutputInner {
    const fn new() -> QEMUOutputInner   // const fn new()
```

## `core::fmt::Write`トレイトの実装

`core::fmt::Write`トレイトを実装すると`format_args!`マクロが利用可能になり、`print!`と`println!`マクロを実装したことになる。このトレイトの実装に必要な関数は`fn write_str(&mut self, s: &str) -> Result`である。

この関数を実装すれば`fn write_fmt(&mut self, args: Arguments<'_>) -> Result`は提供されるが、ここでは`interface::Write`トイレトで`fn write_fmt`を実装を強制している（Mutexでロックするため）

`src/bsp/raspberrypi/console.rs`

```rust
impl fmt::Write for QEMUOutputInner {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            if c == '\n' {
                self.write_char('\r')
            }
            self.write_char(c);
        }
        Ok(())
    }
}

impl QEMUOutputInner {
    fn write_char(&mut self, c: char) {
        unsafe {    // 0x3F20_1000: UART0のDRアドレス
            core::ptr::write_volatile(0x3F20_1000 as *mut u8, c as u8);
        }
        self.chars_written += 1;
    }
}

impl console::interface::Write for QEMUOutput {
    fn write_fmt(&self, args: core::fmt::Arguments) -> fmt::Result {
        self.inner.lock(|inner| fmt::Write::write_fmt(inner, args))
    }
}
```

# 実行

```bash
$ make qemu

Compiling kernel - rpi3
   Compiling tock-registers v0.6.0
   Compiling mingo v0.4.0 (/Users/dspace/raspi_os/04_safe_globals)
   Compiling register v1.0.2
   Compiling cortex-a v5.1.6
    Finished release [optimized] target(s) in 3.91s

Launching QEMU
[0] Hello from Rust!
[1] Chars written: 22
[2] Stopping here.
```

# ディレクトリ構造

```bash
$ tree .
.
├── _arch
│   └── aarch64
│       ├── cpu
│       │   ├── boot.rs
│       │   └── boot.s
│       └── cpu.rs
├── bsp
│   ├── raspberrypi
│   │   ├── console.rs          # 変更
│   │   ├── cpu.rs
│   │   ├── link.ld
│   │   └── memory.rs
│   └── raspberrypi.rs
├── bsp.rs
├── console.rs                  # 変更
├── cpu
│   └── boot.rs
├── cpu.rs
├── main.rs                     # 変更
├── memory.rs
├── panic_wait.rs
├── print.rs
├── runtime_init.rs
└── synchronization.rs          # 新規追加
```
