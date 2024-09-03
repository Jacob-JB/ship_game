Path to git openssl is `C:\Program Files\Git\usr\bin\openssl.exe`


# Certificate Authority

Root CA trusted by the game is CA.key.
In development the passphrase is `1234`
`openssl genrsa -des3 -out CA.key 4096`

Certificate shipped is CA.pem
`openssl req -x509 -new -nodes -key CA.key -sha256 -days 3650 -out CA.pem`


# Server Certificate

Private key for server
`openssl genrsa -out dev.shipgame.key 4096`

Certificate sign request
`openssl req -new -key dev.shipgame.key -out dev.shipgame.csr`

X509 V3 certificate extension config is `dev.shipgame.ext`

The signed certificate is `dev.shipgame.crt`
`openssl x509 -req -in dev.shipgame.csr -CA CA.pem -CAkey CA.key -CAcreateserial -out dev.shipgame.crt -days 3650 -sha256 -extfile dev.shipgame.ext`
