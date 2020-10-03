#!/usr/bin/env bash

PORT=${1:-9000}
LATEST=$(ls logs | grep "port_${PORT}" | sort | tail -1)
less "logs/$LATEST"
