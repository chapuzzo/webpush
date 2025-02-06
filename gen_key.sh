#!/bin/sh

openssl ecparam -genkey -name prime256v1 -out private.pem
openssl ec -in private.pem -pubout -outform DER | tail -c 65 | base64 | tr '/+' '_-' | tr -d '\n' > public.b64