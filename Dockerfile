# Rune VCS Docker Image
# Multi-stage build for optimal image size

# Build stage
FROM rust:1.82-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy workspace configuration
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/

# Build the application
RUN cargo build --release --bin rune

# Generate shell completions (if the binary was built successfully)
RUN if [ -f ./target/release/rune ]; then \
        mkdir -p /completions && \
        ./target/release/rune completions bash > /completions/rune.bash && \
        ./target/release/rune completions zsh > /completions/_rune && \
        ./target/release/rune completions fish > /completions/rune.fish; \
    else \
        mkdir -p /completions && touch /completions/rune.bash /completions/_rune /completions/rune.fish; \
    fi

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    git \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN useradd -m -s /bin/bash rune

# Copy the binary and completions
COPY --from=builder /app/target/release/rune /usr/local/bin/rune-vcs
COPY --from=builder /completions/ /usr/share/bash-completion/completions/

# Set proper permissions
RUN chmod +x /usr/local/bin/rune-vcs

# Create workspace directory
RUN mkdir -p /workspace && chown rune:rune /workspace

# Switch to non-root user
USER rune
WORKDIR /workspace

# Set environment variables
ENV RUNE_CONFIG_HOME=/home/rune/.config/rune
ENV RUNE_DATA_HOME=/home/rune/.local/share/rune

# Create configuration directory
RUN mkdir -p $RUNE_CONFIG_HOME $RUNE_DATA_HOME

# Verify installation (only run version as doctor might fail in container)
RUN rune-vcs --version || true

# Default command
CMD ["rune-vcs", "--help"]

# Labels for metadata
LABEL org.opencontainers.image.title="Rune VCS"
LABEL org.opencontainers.image.description="A modern, intelligent version control system"
LABEL org.opencontainers.image.url="https://github.com/CaptainOtto/rune-vcs"
LABEL org.opencontainers.image.source="https://github.com/CaptainOtto/rune-vcs"
LABEL org.opencontainers.image.version="0.3.0-alpha.4"
LABEL org.opencontainers.image.licenses="Apache-2.0"
LABEL org.opencontainers.image.vendor="Rune Maintainers"
