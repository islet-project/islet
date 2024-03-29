#!/bin/bash

NAME=$1

openssl req -new -key $NAME.key -out $NAME.csr -config <( cat $NAME.conf )
openssl x509 -req -in $NAME.csr -CA root.crt -CAkey root.key -CAcreateserial -out $NAME.crt -days 3600 -sha256 -extensions req_ext -extfile $NAME.conf

#openssl req -new -key $NAME.key -out $NAME.csr -subj '/CN=localhost' -addext subjectAltName='localhost'
#openssl req -new -key $NAME.key -out $NAME.csr -subj '/CN=localhost' -addext subjectAltName='DNS:localhost'
#openssl x509 -req -in $NAME.csr -CA root.crt -CAkey root.key -CAcreateserial -out $NAME.crt -days 3600 -sha256 -copy_extensions copy
