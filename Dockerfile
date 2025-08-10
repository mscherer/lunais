FROM quay.io/fedora/fedora-minimal:latest
LABEL org.opencontainers.image.source="https://github.com/mscherer/lunais"
LABEL maintainer="mscherer@redhat"
COPY . /srv/
WORKDIR /srv/
RUN dnf install -y cargo glibc-static --setopt=install_weak_deps=False && dnf clean all
# see https://msfjarvis.dev/posts/building-static-rust-binaries-for-linux/
# run on 1 single line to not take too much space on my disk with the intermediate container
RUN RUSTFLAGS='-C target-feature=+crt-static' cargo build --release --target $(rustc --print host-tuple) && mv target/$(rustc --print host-tuple)/release/webserver . && rm -Rf target

FROM scratch
EXPOSE 2507
COPY --from=0 /srv/webserver /srv/
CMD ["/srv/webserver"]
