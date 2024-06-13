# Verus 运行环境

1.  确认 Git submodules 已 clone: `git submodule update --init`
2.  构建 verus：[INSTALL.md](./verus/INSTALL.md)，注意使用 `vargo build --release --features singular --vstd-no-std`，以启用 [Singular](https://verus-lang.github.io/verus/guide/nonlinear.html)（需要另外安装 Singular），并使 `vstd` 可以在 `no_std` 环境下使用
3.  构建 verus-analyzer（配置 VS Code）：[README.md](./verus-analyzer/README.md)
4.  构建 verusfmt（`cargo build --release`）
5.  使用 [direnv](https://direnv.net/) 自动将 [env](./env) 中的 symlink 和脚本添加到 `PATH`

`nvim-lspconfig` 配置示例:

```lua
lsp.rust_analyzer.setup {
  ...
  on_init = function(client)
    local path = os.getenv('PATH')

    if path and path:find('verus/env') then
      client.config.settings['rust-analyzer'].checkOnSave.overrideCommand = {
        'verus',
        '--expand-errors',
        '--crate-type',
        'lib',
      }
      client.config.settings['rust-analyzer'].diagnostics = {
        disabled = {
          'syntax-error',
          'break-outside-of-loop',
        },
      }
      client.notify('workspace/didChangeConfiguration', { settings = client.config.settings })
    end

    return true
  end,
  ...
}
```
