## 第七周汇报

致理-信计11 游宇凡

### 本周进展

1.  阅读了论文《Verus: Verifying Rust Programs using Linear Ghost Types》
    -   Verus code 分为 spec、proof、exec 以连接 SMT 和 Rust。其他证明工具一般使用专门用于验证的语言，而在 Verus 中通过对 spec 施加限制来使其适合证明，同时又允许在 exec 中编写正常的 Rust 代码。
    -   利用了由 Rust borrow checker 所保证的性质来简化 SMT reasoning。
    -   不直接支持 unsafe，但提供一些 core primitive 来代替 unsafe Rust 提供的功能。
    -   通过 [`PPtr`](https://verus-lang.github.io/verus/verusdoc/vstd/ptr/struct.PPtr.html) 来验证指针操作，将原本的 `unsafe` 变为了 `requires`，使用 *ghost permission token* 保证操作合法性，只在用于验证的 ghost code 中维护访问权限。
    -   提供了两种 interior mutability 机制，[`PCell`](https://verus-lang.github.io/verus/verusdoc/vstd/cell/struct.PCell.html) 和 [`InvCell`](https://verus-lang.github.io/verus/verusdoc/vstd/cell/struct.InvCell.html)。`PCell` 和 `PPtr` 类似，使用 ghost permission token。`InvCell` 保证存储的数据满足某个 invariant。
    -   提供了 [tokenized state machine](https://verus-lang.github.io/verus/state_machines)，用于处理 concurrent code 等场景。

2.  尝试了一下 [verified-memory-allocator](https://github.com/verus-lang/verified-memory-allocator)
    -   `verus-bitmap-allocator` 有大量报错，作者也说它 “Incomplete, not maintained”，感觉用的是老版本的 Verus，与现在已经差别很大。
    -   `verus-mimalloc` 可以构建出 `.so` 文件，还没测试实际使用。verify 的时候遇到一个奇怪的错误，如下代码 `assert` 验证失败，另外还有几个函数验证超时。
        ```rust
        if idx - 1 > page_id.idx {
            assert(idx - 1 > page_id.idx);
            ...
        }
        ```

3.  编写了一个简单的经验证的 bump allocator。

### 下周计划

-   尝试编写更完善的（可以正常回收 dealloc 的内存）allocator
-   尝试将使用 Verus 的代码编译到 RISC-V 并在 QEMU 中运行
