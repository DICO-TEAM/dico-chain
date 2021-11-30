# Forked from: https://github.com/paritytech/polkadot

FROM docker.io/paritytech/ci-linux:production as builder

WORKDIR /kico
COPY . /kico

# RUN cargo build --locked --release
RUN cargo build --locked --bin kico --release

FROM docker.io/library/ubuntu:20.04

LABEL description="A decentralized and governable ICO platform." \
	io.dico.image.type="builder" \
	io.dico.image.authors="hi@dico.io" \
	io.dico.image.vendor="DICO-TEAM" \
	io.dico.image.description="A decentralized and governable ICO platform." \
	io.dico.image.source="https://github.com/DICO-TEAM/dico-chain/blob/${VCS_REF}/scripts/kico/kico/kico_builder.Dockerfile" \
	io.dico.image.documentation="https://github.com/DICO-TEAM/dico-chain"

COPY --from=builder /kico/target/release/kico /usr/local/bin

RUN useradd -m -u 1000 -U -s /bin/sh -d /kico kico && \
	mkdir -p /data /kico/.local/share && \
	chown -R kico:kico /data && \
	ln -s /data /kico/.local/share/kico && \
# unclutter and minimize the attack surface
	rm -rf /usr/bin /usr/sbin && \
# check if executable works in this container
	/usr/local/bin/kico --version

USER kico

EXPOSE 30333 9933 9944 9615
VOLUME ["/data"]

ENTRYPOINT ["/usr/local/bin/kico"]