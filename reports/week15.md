## 第十五周汇报

致理-信计11 游宇凡

### 本周进展

1.  将 verified-nrkernel 中的代码复制出来，它直接就可以用最新版 Verus 进行验证，但需要进行一些修改才能编译。
2.  添加使用外部 page allocator 来 alloc/dealloc page 的接口。
3.  添加设置 PTE 的 disable cache flag 的接口并补充相关验证代码（这个 flag 在 ArceOS 中被 MMIO 使用）。
4.  添加用于接入 ArceOS 的上层接口。
5.  修复 bug：verified page table 之前只支持 lower half (user space) virtual address，但 ArceOS 用的是 higher half (kernel space) virtual address，直接传过去就会出错。调了很久才找到错误的原因，但修起来很简单，因为 PTE index 看的是 virtual address 的低 48 位（lower half 的高 17 位、higher half 的高 17 位分别是固定的），只需要取低 48 位传给 verified page table 即可。

最终可以通过 Verus 验证，并成功编译运行了启用了 paging feature 的 shell、httpserver、httpclient app。

### 后续计划

编写各项工作的文档，以及最终展示的 slide。
