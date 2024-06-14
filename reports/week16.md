## 第十六周汇报

致理-信计11 游宇凡

### 本周进展

1.  为 verified bitmap allocator 添加了一些功能正确的证明（memory region 不相交，`add_memory` 真的添加了内存，`dealloc` 真的释放了内存，这些性质影响性能而不影响安全性，所以之前暂时没写）
2.  为 verified bitmap allocator 编写了文档
3.  为整个大实验编写了总结文档
