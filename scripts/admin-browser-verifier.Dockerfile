ARG BASE_IMAGE=debian@sha256:e5b6442dd2e9684cf5e87d8338b5968f3b348636fc0be6d7850a381e3731a2bd
FROM ${BASE_IMAGE}

ARG VERIFIER_KEY
ARG VERIFIER_PLATFORM
ARG VERIFIER_SCHEMA=1
ARG SNAPSHOT_TIMESTAMP=20240131T000000Z
ARG CHROMIUM_VERSION=120.0.6099.224-1~deb11u1
ARG BASE_IMAGE

LABEL io.rustzen.browser-verifier.schema="${VERIFIER_SCHEMA}" \
      io.rustzen.browser-verifier.key="${VERIFIER_KEY}" \
      io.rustzen.browser-verifier.platform="${VERIFIER_PLATFORM}" \
      io.rustzen.browser-verifier.snapshot="${SNAPSHOT_TIMESTAMP}" \
      io.rustzen.browser-verifier.chromium-version="${CHROMIUM_VERSION}" \
      io.rustzen.browser-verifier.base-image="${BASE_IMAGE}"

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && printf '%s\n' \
      "deb [check-valid-until=no] https://snapshot.debian.org/archive/debian/${SNAPSHOT_TIMESTAMP} bullseye main" \
      "deb [check-valid-until=no] https://snapshot.debian.org/archive/debian-security/${SNAPSHOT_TIMESTAMP} bullseye-security main" \
      >/etc/apt/sources.list \
    && rm -f /etc/apt/sources.list.d/* \
    && apt-get update \
    && apt-get install -y --no-install-recommends \
      "chromium=${CHROMIUM_VERSION}" \
      "chromium-common=${CHROMIUM_VERSION}" \
      "chromium-sandbox=${CHROMIUM_VERSION}" \
      fontconfig fonts-noto-cjk curl jq file procps util-linux iproute2 python3 \
    && test "$(dpkg-query -W -f='${Version}' chromium)" = "${CHROMIUM_VERSION}" \
    && fc-list :lang=zh | grep -qi 'Noto' \
    && rm -rf /var/lib/apt/lists/* \
    && printf 'schemaVersion=%s\nkey=%s\nplatform=%s\nbaseImage=%s\nsnapshot=%s\nchromiumVersion=%s\n' \
      "${VERIFIER_SCHEMA}" "${VERIFIER_KEY}" "${VERIFIER_PLATFORM}" "${BASE_IMAGE}" "${SNAPSHOT_TIMESTAMP}" "${CHROMIUM_VERSION}" \
      >/usr/local/share/rustzen-browser-verifier.provenance
