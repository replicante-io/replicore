#!/bin/bash


# Pod name and other variables ... 
name=stub-pod


# Podman commands to create configure and start a pod.
podman pod create --name $name \
  --label 'test=something' \
  --hostname 'stub.pod' \
  --publish '8080:80'

podman create --pod $name \
  --annotation 'other=note' \
  --detach \
  'nginx:1.17'

podman pod start $name
