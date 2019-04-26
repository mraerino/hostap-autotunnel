#!/bin/bash

docker-compose run --rm build
scp target/mips-unknown-linux-musl/debug/unifi-wifi-tunnels root@[fe80::ea94:f6ff:fef3:c8c%en5]:/tmp/
