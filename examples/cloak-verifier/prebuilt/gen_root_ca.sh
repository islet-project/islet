#!/bin/bash

#openssl genrsa -out root.key 2048
openssl req -new -sha256 -nodes -out root.csr -newkey rsa:2048 -keyout root.key -config <( cat root.conf )
openssl x509 -req -in root.csr -CAcreateserial -key root.key -sha256 -days 3600 -out root.crt -extensions req_ext -extfile root.conf
