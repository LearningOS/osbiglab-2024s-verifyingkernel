## 第十三周汇报

致理-信计11 游宇凡

### 本周进展

给 verified bitmap allocator 跑了 line count。之前编译错误是因为没有读取到 `.cargo/config.toml` 中的 `rustflags`，被全局的 `~/.cargo/config.toml` 里的选项覆盖了。

| file                | Trusted | Spec | Proof | Exec | Proof+Exec | Comment | Layout | unaccounted | Directives |
|---------------------|---------|------|-------|------|------------|---------|--------|-------------|------------|
| lib.rs              |       0 |    0 |     0 |    6 |          0 |       0 |      0 |          15 |          1 |
| bitmap/mod.rs       |       0 |    0 |     0 |    0 |          0 |       0 |      0 |           5 |          0 |
| bitmap/allocator.rs |       0 |   92 |   119 |  130 |          9 |       5 |      4 |          29 |          1 |
| bitmap/bitmask.rs   |       0 |   45 |    20 |   12 |          0 |       0 |      0 |          21 |          3 |
| bitmap/block.rs     |       0 |  151 |    70 |  124 |         20 |       0 |      0 |          36 |          0 |
|---------------------|---------|------|-------|------|------------|---------|--------|-------------|------------|
| total               |       0 |  288 |   209 |  272 |         29 |       5 |      4 |         106 |          5 |

阅读学习了一下 ArceOS page table 的设计：`PageTable64` 会调用 generic 的 PTE 参数、PTE 接口、page allocator 接口，目前实际被外部使用的 API 只有 `try_new`、`root_paddr`、`map`、`map_region`。

编写了经验证、标注 spec 的 `memory_addr` crate（它被 page table 依赖）。

开始编写经验证的 generic page table。

### 后续计划

编写经验证的 generic page table。
