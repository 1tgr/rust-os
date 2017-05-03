FROM travisci/ci-garnet:packer-1490989530
RUN apt-get update -qq
RUN apt-get install -qq -y genisoimage
RUN apt-get install -qq -y python3-pip
RUN apt-get install -qq -y qemu
RUN git clone --depth=1 https://github.com/1tgr/rust-os.git /host-bootstrap  # 22b11fa0
WORKDIR /host-bootstrap
RUN ./travis-setup.sh
ADD build.sh /docker/
WORKDIR /docker
RUN chmod a+x build.sh
