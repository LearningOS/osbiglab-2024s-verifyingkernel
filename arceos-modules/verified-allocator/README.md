# Verified Allocator

经 Verus 验证的 bitmap allocator。

## 验证

```sh
verus --crate-type lib src/lib.rs
```

## 使用

```toml
# Cargo.toml
[dependencies]
verified-allocator = { path = "/path/to/verified-allocator" }
```

注意：编译本 crate 还需要 Verus vstd 源码（已在 [Cargo.toml](./Cargo.toml) 中通过 dependency vstd 指定）。

API 请参考代码文档（`cargo doc --open`）。

## 结构设计

API 支持添加不连续的 memory region，所以设计上把每次添加进来的 memory region 独立作为一个 block。

每个 block 内部通过 bitmap 记录内存分配情况，即一个 block 的空间分为 metadata、bitmap、user space 三部分，其中 bitmap 的每个 bit 表示 user space 中对应的一个 byte 是否 allocated。

各个 block 之间通过循环链表相连，metadata 中存储 block size 和链表中下一个 block 的地址。

在全局的 allocator 中，还记录了当前 block 以及当前 byte 的位置，相当于实现了 next fit，每次从上次搜索结束的位置开始往后搜索可用的空闲内存，以提升运行效率。

![结构示意图](../../images/bitmap-allocator-structure.png)

## 实现细节

代码分为三个 module：

-   [bitmask](./src/bitmap/bitmask.rs)：位运算
-   [block](./src/bitmap/block.rs)：单个 block
-   [allocator](./src/bitmap/allocator.rs)：整体的 allocator

### bitmask

主要验证了 bset、bclr、bext 这三个位运算的相关性质，即设置、清除、查询一个 bit。这些位运算以及相关引理在 bitmap 相关操作中被调用。

### block

`struct BitmapBlock` 是一个 `tracked struct`，里面的各个 field 都是 permission token `PointsTo`/`PointsToRaw`，仅用于验证。每个 block 的地址会在运行时计算得到（`BitmapBlock` 的 method 基本上都需要传入 block 地址参数），保存下来的只有当前 block 的地址，其他 block 的地址会在需要访问时由链表指针得到，所以全局占用的空间是常数大小，每个 block 的额外内存开销都存储在 block memory 自身。而每个 block 的这些 permissiont token 都会一直保存在外部 allocator 内（虽然是“保存”，但只是逻辑上在 ghost code 中保存，运行时没有开销）。

在创建一个 block 时（函数 `new`），需要传入对应于这个 block memory 的 permission token，然后会将这块 block memory 拆解成小块的 size、next、user space、bitmap 的 permission token，并写入各个字段以及清空 bitmap，完成初始化。

在 alloc 时，会遍历 bitmap，找到足够大的连续空闲内存，然后更新 bitmap，并删去 `BitmapBlock` 中相应的 user space 的 permission token，收集起来作为返回值，以证明 allocate 出来的这些内存可以安全地转移给 caller 使用。

在 dealloc 时，需要传入相应的 permission token，会更新 bitmap，并将传入的 permission token 再放回原处。

为了辅助证明，会证明整个 block 一直保持着一些不变量（即每个函数的 precondition 要求满足这些性质，postcondition 证明仍然满足这些性质），包括：

-   block 地址加上整个 block 的大小不超过 `usize::MAX`
-   block 地址对齐到 `usize` 的 alignment
-   各个 field 的 permission token 对应的地址位于它们该在的地方，且已赋值（initialized）
-   user space permission token 与 bitmap 的取值是一致的，即 bitmap 中是 1 当且仅当对应的 permission token 存在

### allocator

allocator 中记录了当前搜索到的 block 地址和 byte position，以及空间使用量的信息。除此之外，还有两个用于辅助证明的 ghost variable，`block_seq` 按链表顺序记录了各个 block 的地址，`block_map` 从 block 地址映射到 `BitmapBlock`，即各个 permission token。

在 `add_memory` 中，首先会遍历各个 block，检查是否有内存相交的情况，然后会创建新的 block，更新链表指针以及全局维护的相关信息。

在 `alloc` 中，会从当前搜索到的 block 以及 byte position 开始尝试调用各个 block 的 `alloc`，成功的话则更新全局信息并返回。

在 `dealloc` 中，会寻找地址位于哪个 block，然后调用这个 block 的 `dealloc`。

`add_memory` 和 `dealloc` 需要传入 permission token，而为了能在 Verus 外使用，还提供了 `unsafe_add_memory` 和 `unsafe_dealloc`，它们不需要传入 permission token，而是会内部通过 `unsafe_obtain_pt_for_region` 函数获取到需要的 permission token，需要 caller 保证调用正确（只不过 `add_memory` 还是会检查是否有 memory region 相交的情况）。

维护证明的不变量主要包括：

-   `block_seq` 正确：
    -   记录的当前 block 地址是 seq 的第一个元素（或者 allocator 为空，一个 block 都没有）
    -   seq 中每个元素的链表指针指向 seq 中的下一个元素，最后一个元素指向第一个元素
    -   seq 中没有重复元素
-   `block_map` 正确：
    -   `block_map` 的定义域等于 `block_seq`
    -   block 的地址区间均不相交
    -   每个 block（`BitmapBlock`）满足其不变性质
-   全局信息正确：
    -   当前 byte position 不超过当前 block size
    -   available bytes 不超过 total bytes

为各个 API 所证明的的正确性如下：

-   `add_memory`：报错时 allocator 没有发生改变，没有报错时这段 memory region 真的作为一个 block 添加进了 allocator，且不变量保证了各个 memory region 不相交
-   `alloc`：报错时 allocator 没有发生改变，没有报错时返回的地址满足对齐要求，且返回的 permission token 保证了 allocator 有权将这段内存转移给 caller 使用
-   `dealloc`：传入的这段内存要么不位于任何一个 block（正确调用时不应发生），要么被正确释放

## 代码量

| file                  | Spec | Proof | Exec | Proof+Exec | Comment | Layout | unaccounted | Directives |
|:---------------------:|:----:|:-----:|:----:|:----------:|:-------:|:------:|:-----------:|:----------:|
| `lib.rs`              |    0 |     0 |    6 |          0 |       0 |      0 |          15 |          1 |
| `bitmap/mod.rs`       |    0 |     0 |    0 |          0 |       0 |      0 |           5 |          0 |
| `bitmap/allocator.rs` |  133 |   160 |  130 |          9 |      36 |      4 |          35 |          1 |
| `bitmap/bitmask.rs`   |   45 |    20 |   12 |          0 |       0 |      0 |          21 |          3 |
| `bitmap/block.rs`     |  162 |    70 |  125 |         20 |       0 |      0 |          37 |          0 |
| total                 |  340 |   250 |  273 |         29 |      36 |      4 |         113 |          5 |
