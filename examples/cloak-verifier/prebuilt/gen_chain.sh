#!/bin/sh

cp -f cvm1.crt chain1.crt
cat root.crt >> chain1.crt

cp -f cvm2.crt chain2.crt
cat root.crt >> chain2.crt
