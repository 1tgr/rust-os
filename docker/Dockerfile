FROM travisci/ci-garnet:packer-1490989530
RUN add-apt-repository -y 'deb http://ppa.launchpad.net/jonathonf/tup/ubuntu trusty main' && apt-get update
RUN apt-get install -y genisoimage
RUN apt-get install -y python3-pip
RUN apt-get install -y qemu
RUN apt-get install -y --allow-unauthenticated tup
RUN git clone --depth=1 https://github.com/1tgr/rust-os.git /host-bootstrap  # 2b4c91158
WORKDIR /host-bootstrap
RUN ./travis-setup.sh
ADD build.sh /docker/
WORKDIR /docker
RUN chmod a+x build.sh
