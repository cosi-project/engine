#!/usr/bin/env sh

: ${TARGETPLATFORM=}
: ${TARGETOS=}
: ${TARGETARCH=}

: ${OS=}
: ${ARCH=}

env

if [ -z "${TARGETPLATFORM}" ]; then
  echo "TARGETPLATFORM environment variable is not set"
  exit 1
fi

TARGETOS="$(echo ${TARGETPLATFORM} | cut -d"/" -f1)"
if [ -z "${TARGETOS}" ]; then
  echo "no operating system found in ${TARGETPLATFORM}"
  exit 1
fi


TARGETARCH="$(echo ${TARGETPLATFORM} | cut -d"/" -f2)"
if [ -z "${TARGETARCH}" ]; then
  echo "no CPU architecture found in ${TARGETPLATFORM}"
  exit 1
fi

OS="${TARGETOS}"

case "${TARGETARCH}" in
"amd64")
  ARCH="x86_64"
  ;;
"arm64")
  ARCH="aarch64"
  ;;
*)
  echo "${TARGETARCH} is not a supported CPU architecture"
  exit 1
esac

OUT_DIR=/usr/local/target/x86_64-unknown-linux-musl/release

mkdir -p /binaries/generators \
mkdir -p /binaries/plugins \
&& cp -v ${OUT_DIR}/client /binaries/client \
&& cp -v ${OUT_DIR}/engine /binaries/engine \
&& cp -v ${OUT_DIR}/generator-acpi /binaries/generators/acpi-${OS}-${ARCH} \
&& cp -v ${OUT_DIR}/generator-disk /binaries/generators/disk-${OS}-${ARCH} \
&& cp -v ${OUT_DIR}/plugin-mount /binaries/plugins/mount-${OS}-${ARCH} \
&& cp -v ${OUT_DIR}/plugin-resolver /binaries/plugins/resolver-${OS}-${ARCH} \
&& find /binaries -type f -exec strip -v {} \;
