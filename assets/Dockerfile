FROM ubuntu:latest
RUN apt update && apt upgrade -y
RUN apt install curl -y
RUN curl -fsSL https://claude.ai/install.sh | bash && mv /root/.local/bin/claude /usr/bin/claude
WORKDIR /usr/src/claudestine
ENTRYPOINT [ "claude" ]
