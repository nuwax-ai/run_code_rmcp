FROM rust:1.85

# 设置环境变量
ENV DEBIAN_FRONTEND=noninteractive
ENV TZ=Asia/Shanghai
ENV PATH="/root/.deno/bin:/root/.cargo/bin:${PATH}"

# 安装基础依赖
RUN apt-get update && apt-get install -y \
    python3.11 \
    python3.11-venv \
    python3.11-distutils \
    python3.11-dev \
    python3-pip \
    curl \
    vim \
    net-tools \
    gettext-base \
    telnet \
    wget \
    && rm -rf /var/lib/apt/lists/* \
    && ln -snf /usr/share/zoneinfo/$TZ /etc/localtime \
    && echo $TZ > /etc/timezone

# 安装 Node.js
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
    && apt-get install -y nodejs \
    && npm install -g npm@latest

# 安装 go 
# 设置 Go 环境变量
ENV GOPATH=/root/go
ENV PATH=$PATH:/usr/local/go/bin:/root/go/bin

# 创建 Go 工作目录
RUN mkdir -p $GOPATH/bin $GOPATH/src $GOPATH/pkg
# 安装 Go 1.24（根据架构自动选择下载链接）
ARG TARGETARCH
RUN echo "Target Architecture: $TARGETARCH" \
    && if [ "$TARGETARCH" = "amd64" ]; then \
        GO_URL="https://go.dev/dl/go1.24.3.linux-amd64.tar.gz"; \
    elif [ "$TARGETARCH" = "arm64" ]; then \
        GO_URL="https://go.dev/dl/go1.24.3.linux-arm64.tar.gz"; \
    else \
        echo "Unsupported architecture: $TARGETARCH"; \
        exit 1; \
    fi \
    && echo "Downloading Go from: $GO_URL" \
    && curl -fsSL "$GO_URL" -o go.tar.gz \
    && tar -C /usr/local -xzf go.tar.gz \
    && rm go.tar.gz \
    && ln -s /usr/local/go/bin/go /usr/local/bin/go \
    && ln -s /usr/local/go/bin/gofmt /usr/local/bin/gofmt


RUN go version

# 测试mcp用 go-mcp-mysql@latest
RUN go install -v github.com/Zhwt/go-mcp-mysql@latest


# 安装 Deno
RUN curl -fsSL https://deno.land/install.sh | sh


# 安装 uv
RUN curl -LsSf https://astral.sh/uv/install.sh | sh

# 添加uv到PATH
ENV PATH="/root/.local/bin:${PATH}"

# 创建虚拟环境
RUN uv venv

# 设置工作目录
WORKDIR /app

# 暴露端口,实际端口一般是8080
EXPOSE 8080

