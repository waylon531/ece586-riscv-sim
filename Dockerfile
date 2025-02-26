FROM archlinux:base-devel

RUN pacman-db-upgrade
RUN pacman -Syu --noconfirm rust wget

# NOTE: The AUR has packages for 4 variants of the toolchain, we only need
#       the unknown-elf variant
RUN useradd --no-create-home build && usermod -L build
RUN wget https://aur.archlinux.org/cgit/aur.git/snapshot/riscv32-gnu-toolchain-elf-bin.tar.gz
RUN tar xvf riscv32-gnu-toolchain-elf-bin.tar.gz
RUN chown -R build:build riscv32-gnu-toolchain-elf-bin
# Archlinux does not let you run makepkg as root, so swap to the freshly created user
USER build
RUN cd riscv32-gnu-toolchain-elf-bin; makepkg
USER root
RUN pacman -U --noconfirm riscv32-gnu-toolchain-elf-bin/riscv32-gnu-toolchain-elf-bin-2025.01.20-1-x86_64.pkg.tar.zst
ENV HOME=/mount
