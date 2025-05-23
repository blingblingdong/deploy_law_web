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
    wkhtmltopdf \
    libxrender1 \
    libx11-dev \
    libfreetype6 \
    libfontconfig1 # 安裝 wkhtmltopdf

ENV PKG_CONFIG_ALLOW_CROSS=1
ENV OPENSSL_DIR=/usr
ENV OPENSSL_STATIC=1
ENV RUSTFLAGS='-C linker=x86_64-linux-gnu-gcc'
ENV CC_x86_64_unknown_linux_musl="x86_64-linux-gnu-gcc"

WORKDIR /app
COPY ./ .


RUN which wkhtmltopdf
RUN cargo build --target x86_64-unknown-linux-musl --release
RUN wkhtmltopdf --version


FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    libqt5webkit5

RUN apt-get update && apt-get install -y fonts-noto-cjk


WORKDIR /app
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/law_web ./
COPY --from=builder /app/setup.toml ./
COPY --from=builder /app/mydatabase.db ./
COPY --from=builder /app/new_record.css ./
COPY --from=builder /app/output.pdf ./
COPY --from=builder /usr/bin/wkhtmltopdf /usr/bin/wkhtmltopdf

CMD ["/app/law_web"]




