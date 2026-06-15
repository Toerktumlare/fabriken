FROM debian:bookworm-slim

# (Optional but common) install runtime libs
RUN apt-get update && apt-get install -y \
    ca-certificates \
 && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy compiled binary from builder
COPY target/release/controller /app/controller

# Run it
CMD ["/app/controller"]
