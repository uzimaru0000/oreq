#!/bin/bash

cargo run -- tests/fixtures/github.yaml --path /users/{username}/starred -X GET | xargs curl
