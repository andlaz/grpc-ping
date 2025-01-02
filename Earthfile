VERSION 0.7

BUILD_DEPS:
    COMMAND
    RUN apt-get update && \
        apt-get remove -y jq && \
        apt-get install -y bash curl libssl-dev python3-pip && pip3 install yq==2.13.0 && \
        curl -L -o /usr/bin/jq https://github.com/stedolan/jq/releases/download/jq-1.6/jq-linux64 && chmod +x /usr/bin/jq && \
        curl https://raw.githubusercontent.com/helm/helm/main/scripts/get-helm-3 | bash
    RUN jq --version && pip3 show yq

RUST_BUILD_ENV:
    COMMAND
    FROM clux/muslrust:stable

    WORKDIR /work

    COPY --dir src .
    COPY --dir proto .
    COPY build.rs .
    COPY Cargo.toml .
    COPY Cargo.lock .

    COPY (+version/version) /work/version
    RUN version=$(cat version); \
        echo "Updating version to ${version} in Cargo.toml"; \
        sed -iE "s/^version = \".*\"\$/version = \"${version}\"/g" Cargo.toml;

checks:
    DO +RUST_BUILD_ENV
    RUN rustup component add clippy && \
        rustup component add rustfmt
    RUN cargo fmt --check
    RUN cargo clippy

build:
    DO +RUST_BUILD_ENV
    RUN cargo build --release && chmod +x target/x86_64-unknown-linux-musl/release/grpc-ping
    ARG checks=true
    IF [ "$checks" = "true" ]
        BUILD +checks
    END
    SAVE ARTIFACT target/x86_64-unknown-linux-musl/release/grpc-ping /grpc-ping
    SAVE ARTIFACT version /version

image:
    FROM gcr.io/distroless/static
    COPY (+build/grpc-ping) /grpc-ping

    ENTRYPOINT ["/grpc-ping"]

    ARG save_cmd="SAVE_IMAGE"
    ARG name="grpc-ping"
    ARG version
    ARG tag=$version
    DO .+${save_cmd} --image_name=$name --tag=$tag

versioned:
    FROM debian:buster
    COPY (+version/version) /version

    ARG --required next_cmd
    ARG version="$(cat /version)"
    BUILD .+${next_target} --version=$version

chart:
    FROM debian:buster

    DO +BUILD_DEPS

    WORKDIR /work
    COPY --dir charts /work/
    COPY (+version/version) /work/

    ARG chart
    RUN cd charts/${chart} && \
        helm package . --version=$(cat ../../version) --app-version=$(cat ../../version)

    SAVE ARTIFACT /work/charts/${chart}/${chart}*.tgz

SAVE_IMAGE:
    COMMAND
    ARG --required image_name
    ARG --required tag
    SAVE IMAGE ${image_name}:${tag}

SAVE_IMAGE_PUSH:
    COMMAND
    ARG EARTHLY_GIT_SHORT_HASH
    ARG EARTHLY_TARGET_TAG_DOCKER
    ARG --required tag
    ARG --required image_name
    SAVE IMAGE --push ${image_name}:${tag}

SAVE_IMAGE_GHCR:
    COMMAND
    ARG --required tag
    ARG org_name=andlaz
    ARG repo_name=grpc-ping
    DO +SAVE_IMAGE_PUSH --image_name="ghcr.io/${org_name}/${repo_name}" --tag=${tag}

version:
    FROM gittools/gitversion:5.12.0-ubuntu.18.04-6.0

    WORKDIR /work
    COPY --dir .git /work/
    COPY gitversion.yml /work/

    DO +BUILD_DEPS

    RUN set -xe; cd /work && \
        (/tools/dotnet-gitversion || true) && \
        /tools/dotnet-gitversion /config gitversion.yml > version.json && cat version.json && \
        cat version.json | jq -r .LegacySemVer > version

    SAVE ARTIFACT /work/version
    SAVE ARTIFACT /work/version.json