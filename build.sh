#!/bin/bash

ACCOUNT=kawagami77
IMAGENAME=api-server
TAG=axum

docker build --no-cache -t $ACCOUNT/$IMAGENAME:$TAG .