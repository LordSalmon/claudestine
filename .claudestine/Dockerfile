FROM ubuntu:latest

ENV DEBIAN_FRONTEND=noninteractive

RUN apt update && apt upgrade -y
RUN apt install curl git build-essential pkg-config libssl-dev -y
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -y | sh
RUN curl -fsSL https://claude.ai/install.sh | bash && mv /root/.local/bin/claude /usr/bin/claude
WORKDIR /usr/src/claudestine
ENTRYPOINT [ "claude" ]
