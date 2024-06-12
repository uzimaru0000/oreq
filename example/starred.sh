#!/bin/bash

oreq tests/fixtures/github.yaml --path /users/{username}/starred -X GET | xargs curl
