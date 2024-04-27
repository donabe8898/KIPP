#!/bin/sh

CURRENT=$(cd $(dirname $0);pwd)

cd $CURRENT && sudo docker-compose up -d
