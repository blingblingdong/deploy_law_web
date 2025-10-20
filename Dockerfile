FROM rust:1.88-bullseye AS builder

# 安裝 musl 工具與必要函式庫
RUN apt-get update && \
    apt-get install -y musl-tools pkg-config libssl-dev build-essential \
    wkhtmltopdf libxrender1 libx11-dev libfreetype6 libfontconfig1

RUN rustup target add x86_64-unknown-linux-musl

# 設定環境變數
ENV PKG_CONFIG_ALLOW_CROSS=1
ENV OPENSSL_DIR=/usr
ENV OPENSSL_STATIC=1
ENV CC=musl-gcc
ENV RUSTFLAGS="-C target-feature=-crt-static"

WORKDIR /app
COPY . .

# 檢查 wkhtmltopdf
RUN which wkhtmltopdf
RUN wkhtmltopdf --version

# 建立 release
RUN cargo build --release --target x86_64-unknown-linux-musl


# --- Runtime stage ---
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    libqt5webkit5 fonts-noto-cjk && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/law_web ./
COPY --from=builder /app/setup.toml ./
COPY --from=builder /app/mydatabase.db ./
COPY --from=builder /app/new_record.css ./
COPY --from=builder /app/output.pdf ./
COPY --from=builder /usr/bin/wkhtmltopdf /usr/bin/wkhtmltopdf

CMD ["/app/law_web"]
