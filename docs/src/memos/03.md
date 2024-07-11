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
│   │   ├── console.rs          # NEW
│   │   ├── cpu.rs
│   │   ├── link.ld
│   │   └── memory.rs
│   └── raspberrypi.rs
├── bsp.rs
├── console.rs                  # NEW
├── cpu
│   └── boot.rs
├── cpu.rs
├── main.rs
├── memory.rs
├── panic_wait.rs
├── print.rs                    # NEW
└── runtime_init.rs
```

# 実行

```bash
$ make qemu

Compiling kernel - rpi3
   Compiling mingo v0.3.0 (/Users/dspace/raspi_os/03_hacky_hello_world)
    Finished release [optimized] target(s) in 0.92s

Launching QEMU
docker: Cannot connect to the Docker daemon at unix:///var/run/docker.sock. Is the docker daemon running?.
See 'docker run --help'.
make: *** [qemu] Error 125
```

## アプリケーションDockerをオープン。

```bash
$ make qemu

Compiling kernel - rpi3
    Finished release [optimized] target(s) in 0.00s

Launching QEMU
[0] Hello from Rust!                # main.rs

Kernel panic: Stopping here.        # panic_wait.rs
```

# ドキュメント

```bash
$ make doc
```
