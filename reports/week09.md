## 第九周汇报

致理-信计11 游宇凡

### 本周进展

简单了解分析了 ArceOS，成功运行了 helloworld、memtest app。

ArceOS 使用 axalloc 组件进行内存分配，其实际被外部调用了的 api 包括：

-   `global_allocator`
-   `global_add_memory`
-   `global_init`
-   `GlobalAllocator::name`
-   `GlobalAllocator::alloc`
-   `GlobalAllocator::dealloc`
-   axdriver / axhal: `GlobalAllocator::{alloc_pages, dealloc_pages}`
-   posix API: `GlobalAllocator::available_pages`

axalloc 使用了 `allocator` crate，提供了 bitmap、buddy、slab、TLSF allocator，每种 allocator 需要实现 `ByteAllocator` 或 `PageAllocator` trait。

```rust
pub trait BaseAllocator {
    fn init(&mut self, start: usize, size: usize);
    fn add_memory(&mut self, start: usize, size: usize) -> AllocResult;
}

pub trait ByteAllocator: BaseAllocator {
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>>;
    fn dealloc(&mut self, pos: NonNull<u8>, layout: Layout);
    fn total_bytes(&self) -> usize;
    fn used_bytes(&self) -> usize;
    fn available_bytes(&self) -> usize;
}
```

其中 `total_bytes`、`used_bytes`、`available_bytes` 实际上未被使用，可以不实现。

由于需要支持动态添加多段 memory，原来的 bump allocator 需要进行一些修改才能使用。

只不过 bump allocator 无法正常回收内存，实际使用应该很可能遇到问题。所以直接在尝试实现一个简单的 linked list allocator。

目前只实现了一小部分，感觉难度还是较高，除了证明，光是完善地描述 allocator 正确性就需要考虑很多细节，例如：

-   参数可能不合法
-   由于地址对齐、存初始化信息等原因，`add_memory` 不一定整块都能使用
-   由于要原地存一些信息，分配出去的空间可能只是被占用的空间的子集，而 dealloc 时就要允许释放掉比分配出去的内存更多的内存，同时要避免释放掉其他分配出去了的内存
-   要保证 allocator 保存信息使用的地址和分配出去的地址不相交，需要维护 allocator 可读写的内存，在 spec 中体现出内存的可读写特征

所以光是最高层的 spec 就改了几版。

另外，verus-mimalloc 代码很长，没有文档、注释，阅读非常困难，看接口也没找到是否支持添加内存区域，而且如果要接入 ArceOS 应该还是需要把验证代码手动去掉，有一定工作量。

也看了一下 page table，ArceOS 是在 `page_table` crate 中实现了一个架构无关的通用 page table，然后将硬件部分拆分到 `page_table_entry` 和 `axhal` 实现，在 `page_table_entry` 中为各个架构的 PTE 实现通用接口供 `page_table` 使用，在 `axhal` 中实际对硬件进行操作。这个架构和 verified paging for x86_64 不同，后者是页表直接操作内存，各个 state machine 之间也是由内存内容连接在一起进行证明，如果要搬过来用可能也需要进行较大的改动。

### 后续计划

继续实现 linked list allocator state machine，或者改成 bitmap 试试，可能会简单一些。
