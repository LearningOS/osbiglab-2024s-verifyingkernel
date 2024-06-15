## 第八周汇报

致理-信计11 游宇凡

### 本周进展

阅读了论文[《Verified Paging for x86-64 in Rust》](https://github.com/matthias-brun/verified-paging-for-x86-64-in-rust)。

主体设计采用了 state machine refinement，但没有使用 Verus 的 tokenized state machine 功能，而是手动编写，每个状态转移是一个 spec 函数，参数为起始状态以及 label、返回值为 bool 表示转移是否合法，低层 state 可以转换为高层 state，最后需要证明 refinement，就是证明，如果一个低层的状态转移是合法的，则其对应的高层状态转移是合法的。

下图（摘自论文）是该项目的总体结构，其中红色部分是未验证、受信任的，蓝色部分是验证了的。

![verified paging overview](../images/verified-paging-overview.png)

-   application specification: 抽象地描述了页表的行为，是证明的最终目标，定义了正确性的具体含义；同时，如果需要对上层应用进行验证，也可以使用这个 specification 来描述页表的行为。这部分 specification 是以 application 的视角定义的，包含 read、write、map、unmap、resolve (translate) 等操作，由于底层细节的缺失，对操作返回值等（比如返回什么错误类型）没有具体的规定，需要更低层的 specification 进行补充。
-   hardware specification: 描述了硬件的相关行为，维护了 memory 和 TLB，状态转移包括修改 page table memory、读写 application memory、添加/删除 TLB entry。
-   OS state machine: 将 hardware specification 和 page table state machine 结合在一起，并假设 OS 的其他部分不会修改 page table。
-   interface specification: 连接 page table state machine 和实际的 imperative implementation，规定了 precondition（满足 invariant 和 state machine 的 enabling condition）和 postcondition（保持 invariant，state machine transition 合法），使用 trait 编写，从而和 implementation 分离。
-   page table state machine: 和 application specification 类似，包含 map、unmap、resolve 等操作。
-   implemtation 分为三层，对各层依次证明 refinement：
    1.  抽象的 map，描述了证明的目标行为
    2.  抽象的树状结构，模仿了页表的层级结构
    3.  具体的页表实现

verified-page-table 的代码使用的是旧版 Verus，需要编译 Verus fork 的整个 Rust，编译出错了，暂时没有去解决。该项目专门提供了人工提取出的没有验证相关部分的普通 Rust 代码，之后如果需要可以直接使用，也不需要跑 Verus。

我编写了模仿其证明结构的 Fibonacci 数列计算。

另外，verus-mimalloc 在官方更新后成功用最新 Verus 通过了验证。

### 后续计划

-   尝试搭建一个使用 Verus 编译、可以在 QEMU 内运行的 OS 框架
-   或者，尝试编写一个更完善的 memory allocator
-   或者，尝试基于 verus-mimalloc 实现 GlobalAlloc
-   或者，尝试将 verified page table 移植到新版 Verus（移植到 RISC-V？）
