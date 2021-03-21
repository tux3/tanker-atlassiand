FROM rust:1.50 as builder

# Prebuild deps layer
COPY Cargo.* ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs
RUN cargo build --release
RUN rm src/ -r
RUN rm target/release/deps/tanker_atlassiand*

# Build
COPY src ./src
RUN cargo build --release

FROM debian:bullseye-slim
ARG CONFIG_FILE=tanker-atlassiand.conf
ENV CONFIG_FILE ${CONFIG_FILE}

EXPOSE 80

COPY --from=builder target/release/tanker-atlassiand .

CMD ./tanker-atlassiand -c ${CONFIG_FILE}

