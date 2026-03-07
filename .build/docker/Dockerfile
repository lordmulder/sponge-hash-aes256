FROM busybox:stable-musl

ARG TARGETARCH

COPY resources/launcher-$TARGETARCH.sh /bin/sponge256sum
COPY bin/LICENSE bin/BUILD_INFO /
COPY bin/$TARGETARCH/sponge256sum-* /usr/libexec/
COPY resources/data/* /data/

ENTRYPOINT ["/bin/sponge256sum"]
CMD ["-rkm", "/data"]
