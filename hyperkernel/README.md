# Hyperkernel 复现

用于复现 [Hyperkernel: Push-Button Verification of an OS Kernel](https://dl.acm.org/doi/10.1145/3132747.3132748) 的 Docker 运行环境。

1.  clone Hyperkernel（在本目录下）

    ```bash
    git clone https://github.com/uw-unsat/hyperkernel
    ```

2.  构建 Docker image

    ```bash
    sudo docker compose build
    ```

3.  进入 Docker 环境（进入 shell）

    ```bash
    sudo ./run.sh
    ```

    或者直接运行 QEMU

    ```bash
    sudo docker compose up
    ```

4.  在 Docker 中:

    -   构建 hv6

        ```bash
        make
        ```

    -   在 QEMU 中运行 hv6

        ```bash
        make qemu
        ```

        然后可以在 Docker 外部访问 http://localhost:8080/sh.html

    -   运行形式化验证和测试

        ```bash
        make verify -- -v
        make irpy/test
        ```

    -   并行运行形式化验证

        ```
        make hv6-verify-par
        ```
