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
    wkhtmltopdf  # 安裝 wkhtmltopdf

ENV PKG_CONFIG_ALLOW_CROSS=1
ENV OPENSSL_DIR=/usr
ENV OPENSSL_STATIC=1
ENV RUSTFLAGS='-C linker=x86_64-linux-gnu-gcc'
ENV CC_x86_64_unknown_linux_musl="x86_64-linux-gnu-gcc"

WORKDIR /app
COPY ./ .

RUN ls -l /usr/include/openssl
RUN cargo build --target x86_64-unknown-linux-musl --release

FROM scratch
WORKDIR /app
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/law_web ./
COPY --from=builder /app/setup.toml ./
COPY --from=builder /app/mydatabase.db ./
COPY --from=builder /app/new_record.css ./

CMD ["/app/law_web"]


