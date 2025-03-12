FROM archlinux:base-devel

RUN pacman-db-upgrade
RUN pacman -Syu --noconfirm rust curl python3 libmpc mpfr gmp base-devel texinfo gperf patchutils bc zlib expat libslirp git

# Install the cross compiler with the three potential targets we're simulating
RUN mkdir /opt/riscv-cross
RUN git clone https://github.com/riscv-collab/riscv-gnu-toolchain
RUN cd riscv-gnu-toolchain && ./configure --prefix=/opt/riscv-cross --enable-multilib --with-multilib-generator="rv32i-ilp32--;rv32im-ilp32--;rv32imf-ilp32--" --disable-linux && make 

ENV HOME=/mount
ENV PATH="/opt/riscv-cross/bin:$PATH"
