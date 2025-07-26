FROM quay.io/fedora/fedora:latest
LABEL org.opencontainers.image.source="https://github.com/mscherer/lunais"
LABEL maintainer="mscherer@redhat"
COPY . /srv/
WORKDIR /srv/
RUN dnf install -y cargo rust && dnf clean all
RUN cargo build --release

FROM quay.io/fedora/fedora:latest
EXPOSE 2507
COPY --from=0 /srv/target/release/webserver /srv/
CMD ["/srv/webserver"]

