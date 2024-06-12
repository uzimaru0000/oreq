#!/bin/bash

cargo run -- tests/fixtures/firebase.yaml \
	--path /v1/accounts:signInWithPassword \
	-X POST \
	--query-param key=$KEY |
	xargs curl |
	jq '{ idToken: .idToken, email: .email, id: .localId }'
