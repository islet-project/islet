#!/bin/sh

openssl rsa -in cvm2.key -outform PEM -pubout -out cvm2.pub_key
