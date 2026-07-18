FROM debian:trixie-slim

RUN apt-get update && apt-get install -y --no-install-recommends curl grep tzdata \
    && rm -rf /var/lib/apt/lists/*

# 时区设置为 Asia/Shanghai
ENV TZ=Asia/Shanghai
RUN ln -snf /usr/share/zoneinfo/Asia/Shanghai /etc/localtime \
    && echo "Asia/Shanghai" > /etc/timezone

# 创建应用目录与非 root 用户
RUN mkdir /app \
    && groupadd --system rust \
    && useradd --system --gid rust --create-home --shell /bin/bash rust

COPY target/x86_64-unknown-linux-musl/release/weihuda_backend app/

RUN chown -R rust:rust /app

WORKDIR /app
USER rust

CMD ["./weihuda_backend"]
