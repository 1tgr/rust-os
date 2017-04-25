FROM travisci/ci-garnet:packer-1490989530
RUN add-apt-repository -y 'deb http://ppa.launchpad.net/jonathonf/tup/ubuntu trusty main' && apt-get update
RUN apt-get install -y genisoimage
RUN apt-get install -y python3-pip
RUN apt-get install -y qemu
RUN apt-get install -y --allow-unauthenticated tup
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly-2016-08-10
ADD build.sh /docker/
RUN chmod a+x /docker/build.sh

