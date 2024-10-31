####################################################################################################
## Builder
####################################################################################################
FROM registry.cn-beijing.aliyuncs.com/cndev/cndev-builder:0.0.1 AS builder

WORKDIR /cndev

COPY ./api ./api
COPY ./entity ./entity
COPY ./migration ./migration
COPY ./service ./service
COPY ./src ./src
COPY ./Cargo.* ./

RUN cargo build --target x86_64-unknown-linux-musl --release

####################################################################################################
## Final image
####################################################################################################
FROM scratch

# Import from builder.
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /cndev

# Copy our build
COPY --from=builder /cndev/target/x86_64-unknown-linux-musl/release/cndev ./

# Use an unprivileged user.
USER cndev:cndev

CMD ["/cndev/cndev"]