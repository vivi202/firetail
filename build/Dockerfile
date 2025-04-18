FROM rust:1.86.0
ENV CROSS_CONTAINER_IN_CONTAINER=true 
RUN rustup target add x86_64-unknown-freebsd
RUN cargo install cross

COPY --from=docker:dind /usr/local/bin/docker /usr/local/bin

WORKDIR /firetail
ENTRYPOINT ["cross"]


