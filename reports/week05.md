## 第五周汇报

致理-信计11 游宇凡

### 本周进展

1.  阅读了论文《Hyperkernel: Push-Button Verification of an OS Kernel》。

    因为使用的验证工具不同，学习的重点是 Hyperkernel 的设计思路，其核心在于进行简化以降低验证难度：

    -   finite interface：即 syscall 的时间复杂度控制在 O(1)，例如不能遍历整个 fd table / page table
        -   有的 syscall 需要用户提供更多参数，例如 page allocation 需要用户提供 page number
        -   一些复杂 syscall（例如 fork）的功能要进行简化，可以将一些功能交给 user space library 实现
    -   分离 user / kernel address space（rCore tutorial 中已经是这样做的了）
    -   syscall 是 atomic 的，不会被打断
    -   基于 hardware virtualization（Intel VT-x / AMD-V）实现 process
    -   基于 LLVM IR 进行验证，以简化语义处理

2.  为 Hyperkernel 配置了 Docker 运行环境，可以编译、在 QEMU 中运行 hv6、运行形式化验证。

    但是测试的运行速度显著低于作者声明的速度（只需 30min），十几个小时也没有跑完（一开始忘记开 verbose 了，不清楚详细进度），不知道是配置、命令还是什么问题。观察到运行时只使用了单线程，可能可以改成多线程，但即使是单线程速度也过慢了。

    运行全部测试：

    ```
    root@45ef337b3075:/src# make verify -- -v
     PY2      hv6-verify
    Using z3 v4.5.0.0
    test_call_proc (__main__.HV6) ... ok
    test_clone_proc (__main__.HV6) ... ok
    test_extintr (__main__.HV6) ... ok
    test_fault (__main__.HV6) ... ok
    test_preempt (__main__.HV6) ... ok
    test_recv_proc (__main__.HV6) ... ok
    test_reply_wait_proc (__main__.HV6) ... ok
    test_send_proc (__main__.HV6) ... ok
    test_switch_proc (__main__.HV6) ... ok
    test_sys_ack_intr (__main__.HV6) ... ok
    test_sys_alloc_frame (__main__.HV6) ... ok
    test_sys_alloc_intremap (__main__.HV6) ... ok
    test_sys_alloc_io_bitmap (__main__.HV6) ... ok
    test_sys_alloc_iommu_frame (__main__.HV6) ... ok
    ……
    ```

    修改代码引入 bug 后运行测试：

    ```diff
    @@ -26,8 +26,8 @@ int sys_create(int fd, fn_t fn, uint64_t type, uint64_t value, uint64_t omode)
        if (type == FD_NONE)
            return -EINVAL;
    -    if (!is_fd_valid(fd))
    -        return -EBADF;
    +//    if (!is_fd_valid(fd))
    +//        return -EBADF;
        /* fd must be empty */
        if (get_fd(current, fd) != 0)
            return -EINVAL;
    ```

    ```
    root@19ff25f9a493:/src# make hv6-verify-syscall-sys_create
    make hv6-verify -- -v --failfast HV6.test_sys_create
    make[1]: Entering directory '/src'
        PY2      hv6-verify
    Using z3 v4.5.0.0
    test_sys_create (__main__.HV6) ... [type.0 = 1,
    fd.3 = 1073741824,
    @proc_table->struct.proc::hvm.0 = [else -> 0],
    k!135 = [else ->
            If(ULE(16, Var(0)),
                If(ULE(17, Var(0)),
                    If(ULE(18, Var(0)),
                    If(ULE(20, Var(0)),
                        If(ULE(32, Var(0)),
                            If(ULE(64, Var(0)),
                                If(ULE(65, Var(0)), 65, 64),
                                32),
                            20),
                        18),
                    17),
                    16),
                1)],
    @page_desc_table->struct.page_desc::pid.0!143 = [(0, 4) ->
                                            32,
                                            (0, 1024) -> 32,
                                            else -> 16],
    @proc_table->struct.proc::state.0!142 = [(0, 16) -> 3,
                                            (0, 32) ->
                                            2147483648,
                                            else -> 0],
    @proc_table->struct.proc::io_bitmap_a.0 = [else ->
                                            @proc_table->struct.proc::io_bitmap_a.0!144(Var(0),
                                            k!135(Var(1)))],
    @proc_table->struct.proc::ipc_page.0 = [else -> 8192],
    @proc_table->struct.proc::ipc_fd.0 = [(0, 16) -> 2147483648,
                                        else -> 16],
    @proc_table->struct.proc::stack.0 = [else -> 0],
    @proc_table->struct.proc::use_io_bitmap.0 = [else -> 0],
    @proc_table->struct.proc::io_bitmap_b.0!145 = [(0, 32) ->
                                           1024,
                                           else -> 0],
    @proc_table->struct.proc::nr_devs.0 = [else -> 0],
    @proc_table->struct.proc::nr_fds.0 = [else -> 0],
    @current.0 = [else -> 16],
    @proc_table->struct.proc::nr_dmapages.0 = [else -> 0],
    @page_desc_table->struct.page_desc::type.0 = [else ->
                                            @page_desc_table->struct.page_desc::type.0!141(Var(0),
                                            k!136(Var(1)))],
    @proc_table->struct.proc::nr_ports.0 = [else -> 0],
    @proc_table.3 = [else -> 0],
    @proc_table.0 = [else -> 16],
    @proc_table.4 = [else -> 0],
    @proc_table->struct.proc::state.0 = [else ->
                                        @proc_table->struct.proc::state.0!142(Var(0),
                                            k!135(Var(1)))],
    @proc_table->struct.proc::nr_children.0 = [else -> 0],
    @page_desc_table.0 = [else -> 0],
    @page_desc_table->struct.page_desc::pid.0 = [else ->
                                            @page_desc_table->struct.page_desc::pid.0!143(Var(0),
                                            k!136(Var(1)))],
    @proc_table->struct.proc::nr_intremaps.0 = [else -> 0],
    @page_desc_table->struct.page_desc::type.0!141 = [(0, 1) ->
                                            4,
                                            else -> 2],
    @proc_table->struct.proc::page_table_root.0 = [else -> 1],
    @proc_table->struct.proc::io_bitmap_b.0 = [else ->
                                            @proc_table->struct.proc::io_bitmap_b.0!145(Var(0),
                                            k!135(Var(1)))],
    k!136 = [else ->
            If(ULE(1, Var(0)),
                If(ULE(4, Var(0)),
                    If(ULE(1024, Var(0)),
                    If(ULE(8193, Var(0)), 8193, 1024),
                    4),
                    1),
                0)],
    @proc_table->struct.proc::io_bitmap_a.0!144 = [(0, 32) -> 4,
                                            else -> 0],
    @proc_table->struct.proc::nr_pages.0 = [else -> 0],
    @proc_table->struct.proc::nr_vectors.0 = [else -> 0],
    @page_desc_table.1 = [else -> 0]]
    In function hv6/proc.h:sys_create:53:5 @[ hv6/fd.c:32:9 ]
    In function hv6/proc.h:sys_create:53:5 @[ hv6/fd.c:32:9 ]
    FAIL
    ```

### 下周计划

1.  学习 Verus 教程
2.  阅读一些 Hyperkernel 代码，学习其设计
