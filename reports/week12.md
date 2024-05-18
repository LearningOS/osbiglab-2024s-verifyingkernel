## 第十二周汇报

致理-信计11 游宇凡

### 本周进展

写完了 [verified bitmap byte allocator](../arceos-modules/verified-allocator/src/bitmap/)，[接入了 ArceOS](https://github.com/ouuan/arceos/commit/87e2df03425b16925ad97654c420a6ac31ff803c)，成功运行了 memtest、yield、parallel、sleep、shell、httpclient、httpserver 等 app。

也成功接入了 rcore-tutorial（ch6 和 ch8），主要遇到的问题是 rcore-tutorial 里的 heap space 是一个 static 数组，使用 verified allocator 时应该是编译器没看出来 allocator 会访问这段空间，数组就被优化到了栈上，结果爆栈了，调试了很久才发现，后来改成了作为一个 struct field 就好了。

另外给 Verus 开了一些 PR: <https://github.com/verus-lang/verus/pulls?q=is%3Apr+author%3Aouuan>。

发现 verified page table 有一个保持更新、支持新版 Verus 的版本，在 <https://github.com/utaal/verified-nrkernel/tree/main/page-table> 而不是原仓库。

### 后续计划

实现 verified bitmap page allocator（和 byte allocator 基本一致，但代码不太好复用，应该要复制一遍再改一改），接入 ArceOS。

后续可能写 ArceOS 的 page table 模块，或者研究一下 verus-mimalloc 的接入 / verified paging 论文的具体实现。
