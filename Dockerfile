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
    qemu-system-arm \
    qemu-system-x86 \
    zlib1g-dev

ENV PATH=$PATH:/root/.cargo/bin
WORKDIR /build

COPY 3rdparty/toolchain-binary 3rdparty/toolchain-binary
RUN make -s -C 3rdparty/toolchain-binary && rm -rf 3rdparty/toolchain-binary/{src,build}
RUN 3rdparty/target/bin/arm-eabi-gcc --version
RUN 3rdparty/target/bin/arm-eabi-ld --version
RUN 3rdparty/target/bin/x86_64-elf-gcc --version
RUN 3rdparty/target/bin/x86_64-elf-ld --version

COPY 3rdparty/newlib 3rdparty/newlib
RUN make -s -C 3rdparty/newlib && rm -rf 3rdparty/newlib/{src,build}

COPY requirements.txt .
RUN pip3 install --user -r requirements.txt

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --profile minimal
RUN cargo install --vers 0.3.20 xargo

COPY src src
RUN rustup toolchain install $(cat src/rust-toolchain) --component rust-src

ENV PATH=$PATH:/build/3rdparty/target/bin
