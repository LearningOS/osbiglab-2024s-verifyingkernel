# Verus 运行环境

1.  确认 Git submodules 已 clone: `git submodule update --init`
2.  构建 verus：[INSTALL.md](./verus/INSTALL.md)
3.  构建 verus-analyzer（配置 VS Code）：[README.md](./verus-analyzer/README.md)
4.  构建 verusfmt（`cargo build --release`）
5.  使用 [direnv](https://direnv.net/) 自动将 [env](./env) 中的 symlink 和脚本添加到 `PATH`

`nvim-lspconfig` 配置示例:

```lua
lsp.rust_analyzer.setup {
  ...
  on_init = function(client)
    local path = client.workspace_folders[1].name

    if string.find(path, 'verifyingkernel') then
      client.config.settings['rust-analyzer'].checkOnSave.overrideCommand = { 'verus' }
      client.notify('workspace/didChangeConfiguration', { settings = client.config.settings })
    end

    return true
  end,
  ...
}
```
