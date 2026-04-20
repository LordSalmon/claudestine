#!/usr/bin/env sh

docker build -f assets/Dockerfile --pull --no-cache -t self-claudestine:0.0.1 .
