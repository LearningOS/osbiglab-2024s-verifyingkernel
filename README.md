# osbiglab-2024s-verifyingkernel

主要工作：

-   了解了 OS 形式化验证的相关工作
    -   复现了 HyperKernel 的运行与验证
    -   学习了 Verus 工具的使用，编写了一些小练习，给 Verus 提的 4 个 PR 被 merge
-   构建出了内存相关组件经验证的 ArceOS
    -   编写并验证了 memory allocator、memory addr 组件
    -   将 Matthias Brun 的 verified page table 接入 ArceOS

其中最主要的成果是 [verified memory allocator](./arceos-modules/verified-allocator)。

每周的进度汇报见 [reports](./reports)。

## HyperKernel

阅读了论文 [Hyperkernel: Push-Button Verification of an OS Kernel](https://dl.acm.org/doi/10.1145/3132747.3132748)，搭建了用于复现的 Docker 运行环境，见 [hyperkernel 文件夹](./hyperkernel)。

## Verus

### 环境配置

使用 Git submodule + [direnv](https://direnv.net/) 配置了 Verus 运行环境，见 [verus/README.md](./verus/README.md)。

验证、编译运行只需 verus 本体，开发时可以使用 verus-analyzer 和 verusfmt。

另外，本 repo 中配置了 GitHub Actions，会自动在 CI 环境验证运行各个项目，也可以参考 [CI 脚本](.github/workflows/ci.yml) 配置本地运行环境。

### 学习

[Verus Tutorial and Reference](https://verus-lang.github.io/verus/guide/) 介绍了 Verus 基本功能的使用。

论文 [Verus: Verifying Rust Programs using Linear Ghost Types](https://dl.acm.org/doi/10.1145/3586037) 聚焦于 Verus 的 linear ghost type 这一功能。这个功能在目前的 tutorial 中还是 todo，缺少相关介绍。它对于验证指针操作、内存分配等功能非常有用，论文 [Atmosphere: Towards Practical Verified Kernels in Rust](https://dl.acm.org/doi/abs/10.1145/3625275.3625401) 称其为 “the true power of Verus”，值得学习。

简单来说，Verus 中有三种变量（variable mode），exec、tracked 和 ghost，其中 exec 就是正常的会保留在可执行代码中的变量，tracked 和 ghost 变量位于 ghost code 中，仅用于验证，没有运行时开销。tracked 和 ghost 的核心区别在于 tracked 变量依然受 ownership rule 制约，而 ghost 变量可以随意复制（不需要 `Copy` trait）。tracked 变量的这个特性使得它可以用作 permission token（相比之下，可以随意复制的 ghost 变量就不能表示独占的 permission，而如果使用 exec 变量则会带来额外的运行时开销），例如 [`PointsToRaw`](https://verus-lang.github.io/verus/verusdoc/vstd/ptr/struct.PointsToRaw.html) 表示拥有若干内存空间，它可以作为 [`PPtr`](https://verus-lang.github.io/verus/verusdoc/vstd/ptr/struct.PPtr.html) 相关函数的参数来获取指针访问内存的权限，而正常情况下需要通过 `alloc` 函数获取 `PointsToRaw` 类型的实例（或者通过 `new` 函数获取 `PointsTo<T>`），这保证了拥有 `PointsToRaw` 即意味着有权限访问一段内存，而权限拥有者永远是唯一的。更多使用上的细节可以参考 [Variable modes - Verus Reference](https://verus-lang.github.io/verus/guide/reference-var-modes.html) 以及 PPtr、PointsToRaw、PointsTo 等类型的文档。

另外可以参考 [Verus 的示例代码](https://github.com/verus-lang/verus/tree/main/source/rust_verify/example)、[标准库 vstd 文档](https://verus-lang.github.io/verus/verusdoc/vstd/)、现有项目的代码等。

除此之外，还有 [Verus Transition Systems](https://verus-lang.github.io/verus/state_machines/)，介绍了 Verus 的 tokenized state machine 功能，但这个功能主要用于验证 concurrent code，在本大实验中没有用到。

如果有难以解决的问题，可以在 [Zulip](https://verus-lang.zulipchat.com/) 上提问。

### 练习

#### 计算 Fibonacci 数列

见 [verus-exercises/src/fib.rs](./verus-exercises/src/fib.rs)，验证了 Fibonacci 数列计算的正确性，计算过程中不会发生溢出。

#### 使用 state machine refinement 验证 Fibonacci 数列计算

见 [verus-exercises/src/fib_state_machine.rs](./verus-exercises/src/fib_state_machine.rs)，仿照论文《Verified Paging for x86-64 in Rust》的证明结构，通过 state machine refinement 的形式证明了 Fibonacci 数列计算的正确性。

#### bump allocator

见 [verified-global-alloc/src/bump.rs](./verified-global-alloc/src/bump.rs)。实现了一个最简单的 memory allocator，bump allocator。验证了其正确性（alloc 满足参数要求，分配出的内存不会相交，运行过程中不会发生算术溢出等错误），并实现了 `GlobalAlloc` trait，可以直接在 `no_std` 环境下使用。

### Verus PR

见 [author:ouuan · Pull requests · verus-lang/verus](https://github.com/verus-lang/verus/pulls?q=is%3Apr+sort%3Aupdated-desc+is%3Amerged+author%3Aouuan)，4 个 PR 被 merge。

## ArceOS modules

[ArceOS](https://github.com/rcore-os/arceos) 是组件化的操作系统，所以可以为其编写经形式化验证的组件，替换掉原有的组件。

### 运行

最终得到的 ArceOS 位于 [ouuan/arceos verification 分支](https://github.com/ouuan/arceos/tree/verification)，按照 ArceOS 正常的方式编译运行即可，只不过需要有相关组件以及 Verus 的源码，在本 repo 下只要拉取了各个 Git submodule 即可（`git submodule update --init`）。

### verified memory allocator

自己编写的经验证的 bitmap allocator，详见其 [README](./arceos-modules/verified-allocator/README.md)。

### verified memory addr

验证了 ArceOS 的 `memory_addr` crate，主要是证明了地址对齐相关位运算结果的正确性，代码见 [verified-memory-addr](./arceos-modules/verified-memory-addr)。

### verified page table

阅读了论文 [Verified Paging for x86-64 in Rust](https://doi.org/10.3929/ethz-b-000594366)，将 [matthias-brun/verified-paging-for-x86-64-in-rust](https://github.com/matthias-brun/verified-paging-for-x86-64-in-rust)（[utaal/verified-nrkernel](https://github.com/utaal/verified-nrkernel)）移植接入到了 ArceOS 上，添加/修改了相关接口，保留验证代码。代码见 [nr-page-table](./arceos-modules/nr-page-table)。
