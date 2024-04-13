# Verified `GlobalAlloc`

简单的经 Verus 验证的在 `no_std` 下实现了 [`GlobalAlloc`](https://doc.rust-lang.org/alloc/alloc/trait.GlobalAlloc.html) 的 memory allocator。

-   [BumpAllocator](https://os.phil-opp.com/allocator-designs/#bump-allocator)：有大量内存浪费，只有所有 alloc 得到的内存全部 dealloc 后才会真正释放、重用内存。

## Build / Verify

```sh
make
make RELEASE=y
make verify
make clean
```
