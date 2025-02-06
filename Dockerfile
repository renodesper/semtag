# Build stage
FROM rust:bookworm AS builder

WORKDIR /app
COPY . .
RUN cargo build --release

# Final run stage
FROM scratch AS runner

WORKDIR /
COPY --from=builder /app/target/release/semtag /semtag
CMD ["/semtag"]
