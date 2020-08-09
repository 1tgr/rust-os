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
    python3-pip \
    python3-setuptools \
    texinfo \
    zlib1g-dev

ENV PATH=$PATH:/root/.cargo/bin
WORKDIR /build

COPY 3rdparty 3rdparty

RUN make -j -C 3rdparty download

RUN make -s -C 3rdparty tools-arm32 && rm -rf 3rdparty/build/arm32
RUN 3rdparty/target/bin/arm-none-eabi-ld --version
RUN 3rdparty/target/bin/arm-none-eabi-gcc --version

RUN make -s -C 3rdparty tools-amd64 && rm -rf 3rdparty/build/amd64
RUN 3rdparty/target/bin/x86_64-elf-ld --version
RUN 3rdparty/target/bin/x86_64-elf-gcc --version

RUN make -s -C 3rdparty target/bin/qemu-system-arm target/bin/qemu-system-x86_64 && rm -rf 3rdparty/build/qemu
RUN 3rdparty/target/bin/qemu-system-arm --version
RUN 3rdparty/target/bin/qemu-system-x86_64 --version

COPY requirements.txt .
RUN pip3 install --user -r requirements.txt

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --profile minimal
RUN cargo install --vers 0.3.20 xargo

COPY src src
RUN rustup toolchain install $(cat src/rust-toolchain) --component rust-src

ENV PATH=$PATH:/build/3rdparty/target/bin
