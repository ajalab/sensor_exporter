FROM rustembedded/cross:armv7-unknown-linux-gnueabihf-0.2.0

RUN dpkg --add-architecture armhf && \
    apt-get update && \
    apt-get install -y libudev-dev:armhf

ENV PKG_CONFIG_ALLOW_CROSS=1
ENV PKG_CONFIG_PATH=/usr/lib/arm-linux-gnueabihf/pkgconfig