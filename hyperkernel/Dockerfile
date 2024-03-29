FROM ubuntu:18.04

RUN sed -i 's|http://[^/]*/|http://mirrors.tuna.tsinghua.edu.cn/|g' /etc/apt/sources.list
RUN apt-get update
RUN apt-get install -y ca-certificates

RUN sed -i 's|http:|https:|g' /etc/apt/sources.list
RUN apt-get update

RUN apt-get install -y binutils g++ qemu-system-x86 clang-5.0 python python-pkg-resources git make unzip wget

RUN wget https://github.com/Z3Prover/z3/releases/download/z3-4.5.0/z3-4.5.0-x64-ubuntu-14.04.zip -O z3.zip -T 30
RUN unzip z3.zip
WORKDIR /z3-4.5.0-x64-ubuntu-14.04
RUN mv bin/z3 /usr/bin
RUN mv bin/libz3.so /usr/lib
RUN mv bin/python/z3 /usr/lib/python2.7/dist-packages
RUN mkdir /usr/share/doc/z3 && mv LICENSE.txt /usr/share/doc/z3

RUN git config --global --add safe.directory /src

WORKDIR /src
