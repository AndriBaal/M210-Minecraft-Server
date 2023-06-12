# Stage 1: Build
FROM rust:1.70 as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo install --path .

# Stage 2: Run
FROM debian:bullseye-slim
RUN rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/app /usr/local/bin/app
COPY --from=builder /usr/src/app/static /static
COPY --from=builder /usr/src/app/views /views
EXPOSE 8069
CMD ["app"]
