FROM rust:stretch

RUN git clone https://github.com/irevoire/brust && \
	cd brust && \
	cargo build --release && \
	mv target/release/brust . && \
	rm -rf src target

WORKDIR brust

EXPOSE 8787

VOLUME /brust/config

CMD ./brust
