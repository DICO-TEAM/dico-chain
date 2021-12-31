# Forked from: https://github.com/paritytech/polkadot

FROM docker.io/paritytech/ci-linux:production as builder

WORKDIR /dico
COPY . /dico

# RUN cargo build --locked --release
RUN cargo build --bin dico --release

FROM docker.io/library/ubuntu:20.04

LABEL description="A decentralized and governable ICO platform." \
	io.dico.image.type="builder" \
	io.dico.image.authors="hi@dico.io" \
	io.dico.image.vendor="DICO-TEAM" \
	io.dico.image.description="A decentralized and governable ICO platform." \
	io.dico.image.source="https://github.com/DICO-TEAM/dico-chain/blob/${VCS_REF}/scripts/dockefiles/dico/dico_builder.Dockerfile" \
	io.dico.image.documentation="https://github.com/DICO-TEAM/dico-chain"

COPY --from=builder /dico/target/release/dico /usr/local/bin

RUN useradd -m -u 1000 -U -s /bin/sh -d /dico dico && \
	mkdir -p /data /dico/.local/share && \
	chown -R dico:dico /data && \
	ln -s /data /dico/.local/share/dico && \
# unclutter and minimize the attack surface
	rm -rf /usr/bin /usr/sbin && \
# check if executable works in this container
	/usr/local/bin/dico --version

USER dico

EXPOSE 30333 9933 9944 9615
VOLUME ["/data"]

ENTRYPOINT ["/usr/local/bin/dico"]