#!/bin/bash

# Temp-Logger can't directly send data to influxDB API since ESP32-C3 has not TSL software stack.
# This agent program is responsible for changing HTTP communication from Temp-Logger to HTTPS 
# communication and passing data to the InfluxDB API. This program is purpose only for a local
# network because it has no security.

# To install ./install.sh in src/server directory.

TEMPLOGGERDIR="/var/lib/templogger"
TEMPLOGGERSH="influxdb-agent-start.sh"

sudo mkdir $TEMPLOGGERDIR
sudo cp $TEMPLOGGERSH $TEMPLOGGERDIR
sudo chmod 755 $TEMPLOGGERDIR/$TEMPLOGGERSH
sudo chown root $TEMPLOGGERDIR/$TEMPLOGGERSH
sudo chgrp root $TEMPLOGGERDIR/$TEMPLOGGERSH
sudo cp main.js $TEMPLOGGERDIR
sudo cp influxdb-agent.service /lib/systemd/system/.

cd $TEMPLOGGERDIR
sudo npm install --save @influxdata/influxdb-client

sudo systemctl daemon-reload
sudo systemctl enable influxdb-agent.service
sudo systemctl start influxdb-agent.service
sudo systemctl status influxdb-agent.service

exit 0