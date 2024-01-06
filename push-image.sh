#!/bin/bash

# 用來將 build 好的 image 推上 docker hub 的 shell

ACCOUNT=kawagami77
IMAGENAME=api-server
TAG=axum

docker tag $ACCOUNT/$IMAGENAME:$TAG $ACCOUNT/$IMAGENAME:$TAG
docker push $ACCOUNT/$IMAGENAME:$TAG
