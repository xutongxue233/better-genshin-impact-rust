# 项目规则

始终使用中文回复。

本仓库主线正在从旧 C#/.NET 实现迁移到完整 Rust 实现。迁移事实源以 `Docs/rust-migration.md` 为准；旧 C# 代码只作为行为参考，不能因为 Rust 占位实现存在就删除。

## 默认技术栈

- 根目录是 Rust workspace，优先按 `Cargo.toml` 和各 crate 的现有边界组织代码。
- 桌面应用位于 `apps/desktop`，前端是 Tauri 2 + Svelte + TypeScript，后端入口在 `apps/desktop/src-tauri`。
- Rust 侧主要 crate 包括 `bgi-core`、`bgi-script`、`bgi-script-engine`、`bgi-input`、`bgi-hotkey`、`bgi-capture`、`bgi-vision`、`bgi-task`、`bgi-cli`。
- 任务资产默认放在 `crates/bgi-task/assets`；发布包应保持安装根目录同时包含 `GameTask` 和 `Assets`，让运行时路径解析稳定。

## Rust/Tauri 编写规范

- 优先复用现有 crate、模块、类型和错误处理风格，不为局部需求引入跨层依赖。
- 共享业务逻辑放在 Rust crate 中；桌面命令层只做参数转换、运行时适配、取消控制和错误映射。
- JSON 序列化优先使用项目现有的 `serde`/`serde_json` 模型和兼容层。
- Tauri/Svelte UI 遵循现有页面、组件、状态管理和命令调用方式，不引入 WPF/MVVM/XAML 约束。
- 需要从旧实现迁移行为时，先把 C# 行为转换成 Rust 测试或计划模型，再接入真实运行时。

## 旧 C# 代码规则

- `BetterGenshinImpact/`、`Fischless.*`、`Test/` 是 legacy 参考代码。只有任务明确要求修改这些目录时，才应用旧 WPF/.NET 规范。
- 修改 legacy WPF/C# 文件时，ViewModel 继续遵循 CommunityToolkit.Mvvm、`ObservableObject`、`[ObservableProperty]`、`[RelayCommand]`。
- 修改 legacy WPF/C# 文件时，交互优先使用 Microsoft.Xaml.Behaviors.Wpf，简单对话框优先使用 ThemedMessageBox。
- 修改 legacy C# JSON 模型时，优先保持原有 Newtonsoft.Json 或 System.Text.Json 兼容策略。
- 删除旧 C# 代码必须满足 `Docs/rust-migration.md` 的 parity gate；不要在未证明功能等价前移除旧工程或旧资产来源。

## 验证要求

最后，程序能够编译就认为成功，无需实际运行程序。

默认验证优先使用：

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm -C apps/desktop build
```

需要验证桌面发布包时再运行：

```powershell
pnpm -C apps/desktop install --frozen-lockfile
pnpm -C apps/desktop build
cargo build -p bettergi-desktop --release
```

只有明确修改 legacy C#/.NET 代码时，才补充运行：

```powershell
dotnet build BetterGenshinImpact.sln -c Debug
```

如果出现程序占用导致 legacy 编译无法完成，可以放弃该 legacy 编译验证并说明原因。
