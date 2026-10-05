# uuidx

[English](README.md) · [简体中文](README.zh-CN.md)

[![CI](https://img.shields.io/github/actions/workflow/status/Nazumeki/uuidx/ci.yml?style=for-the-badge&logo=githubactions&logoColor=white&label=CI)](https://github.com/Nazumeki/uuidx/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/Nazumeki/uuidx?style=for-the-badge&logo=github&logoColor=white&label=release)](https://github.com/Nazumeki/uuidx/releases/latest)
[![crates.io](https://img.shields.io/crates/v/uuidx-cli?style=for-the-badge&logo=rust&logoColor=white&label=crates.io)](https://crates.io/crates/uuidx-cli)

[![Rust](https://img.shields.io/badge/Rust-1.88%2B-000000?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Coverage](https://img.shields.io/badge/coverage-%E2%89%A597%25-brightgreen?style=for-the-badge)](https://github.com/Nazumeki/uuidx/actions/workflows/ci.yml)
[![License](https://img.shields.io/github/license/Nazumeki/uuidx?style=for-the-badge&color=blue&label=license)](LICENSE)

[![Stars](https://img.shields.io/github/stars/Nazumeki/uuidx?style=for-the-badge&logo=github&logoColor=white&label=stars)](https://github.com/Nazumeki/uuidx/stargazers)
[![Forks](https://img.shields.io/github/forks/Nazumeki/uuidx?style=for-the-badge&logo=github&logoColor=white&label=forks)](https://github.com/Nazumeki/uuidx/forks)
[![Commits](https://img.shields.io/github/commit-activity/t/Nazumeki/uuidx?style=for-the-badge&logo=github&logoColor=white&label=commits)](https://github.com/Nazumeki/uuidx/commits)
[![Issues](https://img.shields.io/github/issues/Nazumeki/uuidx?style=for-the-badge&logo=github&logoColor=white&label=issues)](https://github.com/Nazumeki/uuidx/issues)
[![PRs](https://img.shields.io/github/issues-pr/Nazumeki/uuidx?style=for-the-badge&logo=github&logoColor=white&label=PRs)](https://github.com/Nazumeki/uuidx/pulls)

`uuidx` 是一个用于生成、检查、校验和格式化 UUID 的 Rust 工作区。同一个领域
crate 驱动了 `uuidx` CLI、Rust 应用、通过 WebAssembly 的 JavaScript，以及通过
稳定 C ABI 的原生应用。

## 为什么选择 uuidx

- 生成 UUID v3、v4、v5、v6、v7 和 v8，并支持各版本专属选项。
- 检查 UUID 版本、变体、时间戳、节点元数据、负载和位布局，包括 legacy、
  保留和未知版本的值。
- 从参数、文件或换行分隔的标准输入批量校验和转换。
- 输出适合终端的格式、可安全用于管道的纯值，或用于自动化的 JSONL。
- 自动检查 UUID、标准 21 字符 NanoID、原始 Twitter Snowflake，以及（启用
  `ulid-inspect` 时）ULID。
- 从 Rust、JavaScript 或 C 复用同一套策略，适配器 crate 之间互不依赖。

## 安装

从 crates.io 安装已发布的 CLI：

```console
cargo install uuidx-cli --locked
uuidx --version
```

手动下载发布归档、校验并配置 `PATH` 的步骤，请参见
[`docs/INSTALLATION.md`](docs/INSTALLATION.md)。基于源码检出进行构建、自定义
feature 或目标，或构建库产物，请参见
[`docs/SOURCEBUILD.md`](docs/SOURCEBUILD.md)。

## 快速开始

生成 UUID v7（默认版本）：

```console
uuidx generate
uuidx generate v7 --output plain
```

生成五个 UUID v4，每行一个：

```console
uuidx generate v4 --count 5 --output plain
```

生成确定性的 UUID v5：

```console
uuidx generate v5 --namespace dns --name example.com --output plain
```

以 JSON 检查 UUID，包括时间戳和位元数据：

```console
uuidx inspect 018f2c0b-6c5b-7d2e-8f4a-123456789abc --output json --layout
```

校验换行分隔的数据流。无效记录会输出到 stderr 并返回退出状态 `1`：

```console
printf '%s\n' 018f2c0b-6c5b-7d2e-8f4a-123456789abc not-a-uuid | uuidx validate --output plain
```

转换为 32 字符的 simple 形式：

```console
uuidx convert 018f2c0b-6c5b-7d2e-8f4a-123456789abc --to simple --output plain
```

## UUID 支持策略

即使某个版本不是生成目标，检查、校验和转换仍接受语法有效的 UUID。生成有意
只支持 v3 到 v8。

| 族                     | 检查 | 校验 | 转换 | 生成 |
| ---------------------- | :--: | :--: | :--: | :--: |
| v1, v2                 |  是  |  是  |  是  |  否  |
| v3, v4, v5, v6, v7, v8 |  是  |  是  |  是  |  是  |
| Nil、Max、未知版本     |  是  |  是  |  是  |  否  |

`v3` 和 `v5` 需要命名空间和名称。`v6` 和 `v7` 接受 RFC 3339 或 Unix 毫秒
时间戳。`v8` 需要恰好 16 字节的十六进制应用数据。为保持标准兼容，遗留的
MD5/SHA-1 生成仍然可用，并会发出警告。

## 命令参考

```text
uuidx [OPTIONS] <COMMAND>
```

| 命令       | 别名 | 用途                                         |
| ---------- | ---- | -------------------------------------------- |
| `generate` | `g`  | 生成 UUID v3-v8 值。                         |
| `inspect`  | `i`  | 检查 UUID 及受支持的标识符族。               |
| `validate` | `v`  | 批量校验 UUID 语法。                         |
| `convert`  | `c`  | 转换 canonical、simple、URN 和 braced 形式。 |

全局输出模式为 `auto`、`pretty`、`plain` 和 `json`。使用
`uuidx <command> --help` 查看由解析器生成的选项参考。面向自动化的 JSONL
契约记录在 [`docs/JSON.md`](docs/JSON.md)。

## 集成

### Rust

`uuidx-core` 没有默认 feature，无需 CLI 或 FFI 依赖即可提供解析、生成、
格式化和检查能力：

```toml
[dependencies]
uuidx-core = "0.1"
```

```rust
use uuidx_core::{generate_uuid, GeneratableUuidVersion, GenerationOptions};

let value = generate_uuid(&GenerationOptions::new(GeneratableUuidVersion::V7))?;
println!("{value}");
```

当 Rust 应用需要进行只读 ULID 检查时，启用 `ulid-inspect`。

### WebAssembly

`uuidx-wasm` 通过 `wasm-bindgen` 导出 `generateUuid`、`validateUuid`、
`formatUuid`、`inspectUuid`、`inspectIdentifier`、`inspectUlid`、
`inspectNanoid` 和 `inspectSnowflake`。构建受支持的目标：

```console
rustup target add wasm32-unknown-unknown
just build-wasm
```

### C ABI

`uuidx-ffi` 构建静态库和动态库，并在
[`crates/uuidx-ffi/include/uuidx.h`](crates/uuidx-ffi/include/uuidx.h) 发布其头文件：

```console
just build-ffi
```

## 架构

```text
uuidx-cli  ---+
uuidx-wasm ---+--> uuidx-core --> uuid
uuidx-ffi  ---+
```

`uuidx-core` 负责领域行为和公共策略。CLI 负责输入与呈现；WebAssembly 负责
JavaScript 转换；FFI 负责 C ABI 与内存边界。crate 归属、依赖方向和扩展规则
记录在 [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md)。

## 贡献

对 crate 边界的修改应遵循 [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md)。
对 JSON 输出的修改应遵循 [`docs/JSON.md`](docs/JSON.md)。贡献者工作流、
测试要求和 pull request 检查清单见
[`CONTRIBUTION.md`](CONTRIBUTION.md)。

## 许可证

`uuidx` 基于 [MIT 许可证](LICENSE) 发布。
