# Use the official Rust image as a builder
FROM rust:1.75-alpine as builder

# Install build dependencies
RUN apk add --no-cache musl-dev

# Set the working directory
WORKDIR /app

# Copy the Cargo files
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies
RUN cargo build --release

# Remove the dummy main.rs and copy the real source code
RUN rm src/main.rs
COPY src ./src
COPY migrations ./migrations

# Build the application
RUN cargo build --release

# Create a new stage with a minimal image
FROM alpine:latest

# Install runtime dependencies
RUN apk add --no-cache ca-certificates

# Create a non-root user
RUN addgroup -g 1001 -S app && \
    adduser -S app -u 1001

# Set the working directory
WORKDIR /app

# Copy the binary from the builder stage
COPY --from=builder /app/target/release/collaborative-docs-rs .

# Change ownership to the app user
RUN chown app:app /app/collaborative-docs-rs

# Switch to the app user
USER app

# Expose the port
EXPOSE 3000

# Run the binary
CMD ["./collaborative-docs-rs"]