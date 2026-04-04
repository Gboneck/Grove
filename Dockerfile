# Grove OS — Multi-stage Docker build
# Builds the Tauri backend (Rust) and serves as headless API + MCP server

# Stage 1: Build the Rust backend
FROM rust:1.80-slim-bookworm AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libgtk-3-dev \
    libwebkit2gtk-4.1-dev \
    libjavascriptcoregtk-4.1-dev \
    libappindicator3-dev \
    librsvg2-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY src-tauri/ ./src-tauri/
COPY roles/ ./roles/
COPY soul.md ./soul.md
COPY context.json ./context.json

WORKDIR /app/src-tauri

# Build the MCP binary (headless, no GUI required)
RUN cargo build --release --bin grove-mcp

# Stage 2: Minimal runtime
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create grove user
RUN useradd -m -s /bin/bash grove

WORKDIR /home/grove

# Copy built binary
COPY --from=builder /app/src-tauri/target/release/grove-mcp /usr/local/bin/grove-mcp

# Copy default configuration
COPY --chown=grove:grove roles/ /home/grove/.grove/roles/
COPY --chown=grove:grove soul.md /home/grove/.grove/soul.md
COPY --chown=grove:grove context.json /home/grove/.grove/context.json

# Create required directories
RUN mkdir -p /home/grove/.grove/memory/longterm \
    /home/grove/.grove/memory \
    /home/grove/.grove/notes \
    /home/grove/.grove/logs \
    /home/grove/.grove/plugins \
    /home/grove/.grove/profiles \
    && chown -R grove:grove /home/grove/.grove

USER grove

# Default config
RUN echo '[models]\nlocal_model = "gemma3:4b"\ncloud_model = "claude-sonnet-4-20250514"\nollama_url = "http://ollama:11434"\nprefer_local = true\nconfidence_threshold = 0.7\nescalation_logging = true\noffline_mode = false\nperiodic_reasoning_minutes = 0' > /home/grove/.grove/config.toml

# MCP server listens on stdin/stdout
ENTRYPOINT ["grove-mcp"]
