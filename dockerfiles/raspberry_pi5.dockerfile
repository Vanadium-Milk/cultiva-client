FROM rust:trixie
LABEL authors="churra"

ARG CROSS_DEB_ARCH
ENV CROSS_DEB_ARCH=${CROSS_DEB_ARCH} \
    PKG_CONFIG_DIR=/dev/null \
    PKG_CONFIG_ALLOW_CROSS=1 \
    PKG_CONFIG_PATH=/usr/lib/aarch64-linux-gnu/pkgconfig \
    OPENSSL_DIR=/usr \
    OPENSSL_INCLUDE_DIR=/usr/include/openssl \
    OPENSSL_LIB_DIR=/usr/lib/aarch64-linux-gnu \
    UDEV_LIB_DIR=/usr/lib/aarch64-linux-gnu \
    UDEV_INCLUDE_DIR=/usr/include \
    CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
    CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc \
    CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++

RUN dpkg --add-architecture $CROSS_DEB_ARCH
RUN apt-get update
RUN apt-get install -y gcc-aarch64-linux-gnu
RUN apt-get install -y g++-aarch64-linux-gnu
RUN apt-get install -y pkg-config
RUN apt-get install -y libssl-dev:$CROSS_DEB_ARCH
RUN apt-get install -y libudev-dev:$CROSS_DEB_ARCH

RUN ln -sf /usr/bin/aarch64-linux-gnu-pkg-config /usr/bin/pkg-config

RUN rustup target add aarch64-unknown-linux-gnu

RUN mkdir -p /root/.cargo
RUN echo '[target.aarch64-unknown-linux-gnu]\n\
linker = "aarch64-linux-gnu-gcc"\n\
rustflags = ["-C", "link-arg=-Wl,-rpath-link,/usr/lib/aarch64-linux-gnu"]' \
    > /root/.cargo/config.toml

COPY . /cultiva-client
WORKDIR /cultiva-client

CMD ["cargo", "build", "--target", "aarch64-unknown-linux-gnu", "--release"]