#!/bin/bash

base_dir=$(readlink -f $(dirname $0))
image_name="ghcr.io/elna-ai/sdk:latest"
fail() {
    echo "##### FAIL: $1"
    exit 1
}


docker run \
    --rm \
	--volume ${base_dir}:/work \
    --workdir /work \
    --network host \
	-it --privileged \
    ${image_name} $@ \
        || fail "running docker"
