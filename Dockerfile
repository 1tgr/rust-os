FROM ubuntu:20.04
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update -qq

RUN apt-get install -qq -y \
    curl \
    genisoimage \
    git \
    libfdt-dev \
    libglib2.0-dev \
    libpixman-1-dev \
    libwayland-cursor0 \
    libwayland-dev \
    libxkbcommon-dev \
    python3-pip \
    zlib1g-dev

ENV PATH=$PATH:/root/.cargo/bin:/build/3rdparty/bin
WORKDIR /build

COPY 3rdparty 3rdparty
RUN make -s -C 3rdparty tools
RUN x86_64-elf-ld --version
RUN qemu-system-x86_64 --version

COPY requirements.txt .
RUN pip3 install --user -r requirements.txt

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --profile minimal
RUN cargo install --vers 0.3.20 xargo

COPY src src
RUN rustup toolchain install $(cat src/rust-toolchain) --component rust-src
