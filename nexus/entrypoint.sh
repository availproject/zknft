#!/bin/sh
ls
cat config.yaml
#TODO: Remove app id from below once LC is fixed.
exec ./avail-light-linux-amd64 -c config.yaml --app-id 8
