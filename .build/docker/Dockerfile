FROM busybox:stable-musl

ARG TARGETARCH

COPY bin/LICENSE /sponge256sum.LICENSE
COPY bin/BUILD_INFO /sponge256sum.BUILD_INFO
COPY resources/data/*.txt resources/data/LICENSE /data/
COPY bin/$TARGETARCH/sponge256sum-* /usr/libexec/
COPY resources/launcher-$TARGETARCH.sh /bin/sponge256sum

ENTRYPOINT ["/bin/sponge256sum"]
CMD ["-rkm", "/data"]
