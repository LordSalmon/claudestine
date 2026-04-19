FROM oven/bun:latest AS installer
USER bun
RUN bun install --global @anthropic-ai/claude-code

FROM ubuntu:22.04
COPY --from=installer /home/bun/.bun/install/global/node_modules/@anthropic-ai/claude-code-linux-x64/claude /usr/local/bin/claude
ENV HOME=/root
WORKDIR /usr/src/claudestine
ENTRYPOINT ["claude"]
