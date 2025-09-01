FROM rust:latest AS builder

RUN rustup target add x86_64-unknown-linux-musl
RUN apt-get update && \
    apt-get install -y \
    musl-tools \
    musl-dev \
    gcc-x86-64-linux-gnu \
    build-essential \
    libssl-dev \
    pkg-config \
    wget \
    xz-utils \
    libxrender1 \
    libx11-dev \
    libfreetype6 \
    libfontconfig1 && \
    wget https://github.com/wkhtmltopdf/packaging/releases/download/0.12.6-1/wkhtmltox_0.12.6-1.focal_amd64.deb && \
    apt-get install -y ./wkhtmltox_0.12.6-1.focal_amd64.deb && \
    rm wkhtmltox_0.12.6-1.focal_amd64.deb


ENV PKG_CONFIG_ALLOW_CROSS=1
ENV OPENSSL_DIR=/usr
ENV OPENSSL_STATIC=1
ENV RUSTFLAGS='-C linker=x86_64-unknown-linux-gnu-gcc'
ENV CC_x86_64_unknown_linux_musl="x86_64-unknown-linux-gnu-gcc"

WORKDIR /app
COPY ./ .

RUN which wkhtmltopdf
RUN cargo build --target x86_64-unknown-linux-musl --release
RUN wkhtmltopdf --version


FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    libqt5webkit5 \
    fonts-noto-cjk \
    libxrender1 \
    libx11-6 \
    libfreetype6 \
    libfontconfig1 \
    xz-utils

WORKDIR /app
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/law_web ./
COPY --from=builder /app/setup.toml ./
COPY --from=builder /app/mydatabase.db ./
COPY --from=builder /app/new_record.css ./
COPY --from=builder /app/output.pdf ./
COPY --from=builder /usr/local/bin/wkhtmltopdf /usr/bin/wkhtmltopdf

CMD ["/app/law_web"]
