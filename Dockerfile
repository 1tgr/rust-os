FROM ubuntu:20.04
ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update -qq && apt-get install -qq -y \
    bison \
    curl \
    flex \
    genisoimage \
    git \
    libfdt-dev \
    libglib2.0-dev \
    libgmp3-dev \
    libmpc-dev \
    libmpfr-dev \
    libpixman-1-dev \
    libwayland-cursor0 \
    libwayland-dev \
    libxkbcommon-dev \
    ninja-build \
    python3-pip \
    python3-setuptools \
    zlib1g-dev

ENV PATH=$PATH:/root/.cargo/bin
WORKDIR /build

COPY 3rdparty/binutils 3rdparty/binutils
RUN make -s -C 3rdparty/binutils && rm -rf 3rdparty/binutils/{src,build}
RUN 3rdparty/target/bin/arm-none-eabi-ld --version
RUN 3rdparty/target/bin/x86_64-elf-ld --version

COPY 3rdparty/gcc 3rdparty/gcc
RUN make -s -C 3rdparty/gcc && rm -rf 3rdparty/gcc/{src,build}
RUN 3rdparty/target/bin/arm-none-eabi-gcc --version
RUN 3rdparty/target/bin/x86_64-elf-gcc --version

COPY 3rdparty/newlib 3rdparty/newlib
RUN make -s -C 3rdparty/newlib && rm -rf 3rdparty/newlib/{src,build}

COPY 3rdparty/zlib 3rdparty/zlib
RUN make -s -C 3rdparty/zlib && rm -rf 3rdparty/zlib/{src,build}

COPY 3rdparty/libpng 3rdparty/libpng
RUN make -s -C 3rdparty/libpng && rm -rf 3rdparty/libpng/{src,build}

COPY 3rdparty/freetype 3rdparty/freetype
RUN make -s -C 3rdparty/freetype && rm -rf 3rdparty/freetype/{src,build}

COPY 3rdparty/pixman 3rdparty/pixman
RUN make -s -C 3rdparty/pixman && rm -rf 3rdparty/pixman/{src,build}

COPY 3rdparty/cairo 3rdparty/cairo
RUN make -s -C 3rdparty/cairo && rm -rf 3rdparty/cairo/{src,build}

COPY 3rdparty/qemu 3rdparty/qemu
RUN make -s -C 3rdparty/qemu && rm -rf 3rdparty/qemu/{src,build}
RUN 3rdparty/target/bin/qemu-system-arm --version
RUN 3rdparty/target/bin/qemu-system-x86_64 --version

COPY requirements.txt .
RUN pip3 install --user -r requirements.txt

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --profile minimal
RUN cargo install --vers 0.3.20 xargo

COPY src src
RUN rustup toolchain install $(cat src/rust-toolchain) --component rust-src

ENV PATH=$PATH:/build/3rdparty/target/bin
