#!/bin/bash

echo "Waiting for nodes to start ..."
sleep 30

echo "Checking (and initialising) replica set ..."
mongo --norc --host node1 /files/init.js
