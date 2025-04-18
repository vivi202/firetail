#!/bin/bash

check_docker() {
      if ! command -v docker &> /dev/null; then
        echo "ERROR: Docker is not installed. Please install Docker to proceed."
        exit 1
      fi
}

build_docker_image() {
  if ! docker buildx build build -t firetail/freebsd_build; then
    echo "ERROR: Failed to build the Docker image 'firetail/freebsd_build'"
    exit 1
  fi
}

compile()  {
  echo "INFO: Compiling firetail for FreeBSD..."
  docker run -v /var/run/docker.sock:/var/run/docker.sock \
  -v "$PWD:/firetail" firetail/freebsd_build:latest build "$@"
}

restore_ownership() {
  echo "INFO: Restoring ownership of the 'target' directory..."
  if ! sudo chown -R "$USER:$USER" target; then
    echo "Failed to change ownership of target folder"
  fi
}

check_docker

if ! docker image inspect firetail/freebsd_build &> /dev/null; then
  echo "INFO: 'firetail/freebsd_build' image not found. Initiating build process..."
  build_docker_image
fi

compile --target=x86_64-unknown-freebsd --release

if [ -d "target" ]; then
  restore_ownership
fi


