FROM rust

RUN rustup target add mips-unknown-linux-musl

RUN mkdir -p /opt/openwrt_sdk/ && \
    cd /opt/openwrt_sdk/ && \
    wget -q https://downloads.openwrt.org/releases/18.06.2/targets/ar71xx/generic/openwrt-sdk-18.06.2-ar71xx-generic_gcc-7.3.0_musl.Linux-x86_64.tar.xz && \
    tar -xf *.tar.xz --strip-components=1 && \
    mv /opt/openwrt_sdk/staging_dir/toolchain-* /opt/openwrt_sdk/staging_dir/toolchain

ENV PATH="/opt/openwrt_sdk/staging_dir/toolchain/bin:$PATH"

#ENV TARGET="mips-unknown-linux-musl"
#ENV LINKER="mips-openwrt-linux-gcc"
