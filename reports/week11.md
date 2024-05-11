## 第十一周汇报

致理-信计11 游宇凡

### 本周进展

学习了一下 verus-bitmap-allocator 的实现。它的特点是，支持多线程（内部维护的是 `atomic`，操作具有原子性），不支持动态添加内存区域，不知为何内部使用了 `Vec`（应该会依赖于 global allocator）。询问了作者 “incomplete” 具体指的是什么，作者表示不记得了。

只不过学习到了重要的设计思路，即使用 permission token (`PointsTo`/`PointsToRaw`) 而非几个集合来维护内存的使用：一方面，使用 permission token 自动就说明了内存分配不会存在重复分配等问题，不需要另外编写相关的 spec，证明也容易一些；另一方面，permission token 也可以用来维护 allocator 对 metadata 的读写权限。我也终于搞懂了 permission token 以及 `tracked`、`ghost` 变量怎么用，其核心在于，permission token 是利用了 Rust 的 ownership (linear type) 机制，token 需要作为 `tracked` 变量，它位于 ghost code (proof mode) 中，没有运行时开销，但受 borrow checker 制约，所以不能被随意复制，可以用来做权限检查（相对地，可以被随意复制的用于辅助证明的变量就可以是 `ghost`）。

根据这一新思路，我几乎重写了自己的 bitmap allocator，感觉顺畅了很多，比较有信心了，但还是暂未写完。

另外：

-   发现 Verus 其实[自 2023.9 起](https://github.com/verus-lang/verus/pull/733)就支持将 verified crate 直接用于 unverified crate，在 Cargo.toml 中指定好 vstd、builtin 等依赖即可，不需要手动删去证明。
-   发现使用 `verus --crate-type lib` 就不需要写 `fn main() {}`，在 `#![no_std]` 时也不需要补 global allocator 和 panic handler。
-   发现了 Verus 全局设置 `size_of` 的一个 bug：[global size\_of/layout does not work across multiple modules](https://github.com/verus-lang/verus/issues/1114)

### 后续计划

完成 bitmap allocator 的证明。
