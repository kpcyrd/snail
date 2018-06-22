#!/bin/sh
set -ex

case "$BUILD_MODE" in
#    docker)
#        docker build -t snail .
#        docker images snail
#        ;;
#    reprotest)
#        docker build -t reprotest-snail -f ci/Dockerfile.reprotest .
#        docker run --privileged reprotest-snail ci/reprotest.sh
#        ;;
    *)
        docker build --build-arg TARGET="$TARGET" -t "snail-test-$TARGET" -f ci/Dockerfile .
        docker run -t -e TARGET="$TARGET" "snail-test-$TARGET" ci/build.sh
        ;;
esac
