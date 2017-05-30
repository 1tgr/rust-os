FROM travisci/ci-garnet:packer-1490989530
RUN apt-get update -qq
RUN apt-get install -qq -y \
    genisoimage \
    libfdt-dev \
    libglib2.0-dev \
    libpixman-1-dev \
    python3-pip \
    zlib1g-dev
RUN git clone --depth=1 https://github.com/1tgr/rust-os.git /host-bootstrap  # ed21e1de
WORKDIR /host-bootstrap
RUN ./travis-setup.sh
ADD build.sh /docker/
WORKDIR /docker
RUN chmod a+x build.sh
