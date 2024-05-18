## 第十周汇报

致理-信计11 游宇凡

### 本周进展

看了 verus-mimalloc 和 OS 之间的接口。如果要在 ArceOS 上使用，需要用被动的 `add_memory` 替换掉原来主动 `mmap` 获取更多内存的功能（位于 [`os_mem.rs`](https://github.com/verus-lang/verified-memory-allocator/blob/main/verus-mimalloc/os_mem.rs)），还需要提供一个 “thread local ID”（位于 [`thread.rs`](https://github.com/verus-lang/verified-memory-allocator/blob/main/verus-mimalloc/thread.rs)）。询问了作者，作者认为这应该是可行的。只不过，verus-mimalloc 只支持 128KiB 以内的 alloc，可能也无法运行 ArceOS 上的一些 app。暂时没有尝试真的接到 ArceOS 上。

编写了 bitmap allocator，（尚未验证的版本）成功接到了 ArceOS 上，可以运行 memtest 等 app，见 <https://github.com/ouuan/arceos/commit/ec1a64f225b02cdecbf5b0a30ee7db183ae703d7>。正在编写证明。

### 后续计划

完成 bitmap allocator 的证明。
