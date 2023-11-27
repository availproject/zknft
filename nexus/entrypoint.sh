#!/bin/sh
ls
cat config.yaml
exec ./avail-light-linux-amd64 -c config.yaml
