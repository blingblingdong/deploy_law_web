# 使用 Rust 官方的最新版作為基礎映像
FROM rust:latest AS builder

# 添加 musl 目標平台
RUN rustup target add x86_64-unknown-linux-musl

# 安裝 musl 工具和其他必要的開發工具
RUN apt-get update && \
    apt-get install -y musl-tools musl-dev gcc-x86-64-linux-gnu build-essential libssl-dev pkg-config

# 設定環境變量以支持交叉編譯和靜態連接
ENV PKG_CONFIG_ALLOW_CROSS=1
ENV OPENSSL_DIR=/usr
ENV OPENSSL_STATIC=1

# 設定用於 musl 的 Rust 和 C 連接器
ENV RUSTFLAGS='-C linker=musl-gcc'
ENV CC_x86_64_unknown_linux_musl="musl-gcc"

# 設定工作目錄
WORKDIR /app

# 複製本地的源碼到容器中
COPY ./ .

# 列出 OpenSSL 標頭檔案的位置，確保正確設定
RUN ls -l /usr/include/openssl

# 使用 musl 進行靜態編譯
RUN cargo build --target x86_64-unknown-linux-musl --release

# 檢查可執行文件是否存在
RUN ls -la /app/target/x86_64-unknown-linux-musl/release/

# 檢查可執行文件是否為靜態連接
RUN ldd /app/target/x86_64-unknown-linux-musl/release/law_web || echo "Static binary"

# 確保可執行文件具有執行權限
RUN chmod +x /app/target/x86_64-unknown-linux-musl/release/law_web


# 使用 scratch 基礎映像來創建最終映像
FROM alpine
WORKDIR /app
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/law_web ./
COPY --from=builder /app/setup.toml ./
COPY --from=builder /app/mydatabase.db ./
CMD ["/app/law_web"]


