####################################################################################################
## Builder
####################################################################################################
FROM rust:latest AS builder

RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt install -y musl-tools musl-dev
RUN update-ca-certificates

# Create appuser
ENV USER=rustacean
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"


WORKDIR /actixbook

COPY . .

RUN cargo build --target x86_64-unknown-linux-musl --release

####################################################################################################
## Final image
####################################################################################################
FROM alpine:latest

# Import from builder.
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group
COPY --from=builder /actixbook/files /actixbook/files
COPY --from=builder /actixbook/react-front /actixbook/react-front
WORKDIR /actixbook

# Copy our build
COPY --from=builder /actixbook/target/x86_64-unknown-linux-musl/release/actixbook ./

# Use an unprivileged user.
# This is a chown from myapp user to myapp folder
RUN chown -R rustacean /actixbook
USER rustacean
EXPOSE 80/tcp
CMD ["/actixbook/actixbook"]