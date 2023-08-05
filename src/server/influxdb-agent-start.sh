#!/bin/bash

export NODE_HOME=/var/lib/templogger
export PATH=/usr/bin:$PATH

/usr/bin/node $NODE_HOME/main.js &

exit 0
