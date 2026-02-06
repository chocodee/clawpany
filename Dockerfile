FROM rust:1.77-bookworm as builder
WORKDIR /app
COPY . .
RUN cargo build -p orchestrator --release

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/orchestrator /app/orchestrator
EXPOSE 3000
ENV ORCH_API_KEY=dev_key
CMD ["/app/orchestrator"]
