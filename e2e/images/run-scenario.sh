#!/bin/bash

# don't allow errors
set -e

RED='\033[91m'
BLUE='\033[94m'
BOLD='\033[1m'
RESET='\033[0m'

function error_message {
    echo -e "${BOLD}${RED}error:${RESET} $@"
}

function hint_message {
    echo -e "${BOLD}${BLUE}hint:${RESET} $@"
}

function not_defined_error {
    error_message "${1} is not defined, please define it before running the tests."
}

ERROR=""

if [ -z "${DATABASE}" ]; then
    not_defined_error "DATABASE"
    ERROR="true"
fi

if [ -z "${SCENARIO}" ]; then
    not_defined_error "SCENARIO"
    ERROR="true"
fi

if [ "${DATABASE}" = "sqlite" ]; then
    FLAGS="-f docker-compose.yaml"
elif [ "${DATABASE}" = "mysql" ]; then
    FLAGS="-f docker-compose.yaml -f mysql-compose.yaml"
elif [ "${DATABASE}" = "postgres" ]; then
    FLAGS="-f docker-compose.yaml -f postgres-compose.yaml"
else
    error_message "expected \`sqlite\`, \`mysql\` or \`postgres\` as DATABASE, got \`${DATABASE}\`"
    ERROR="true"
fi

if [ ! -z "${ERROR}" ]; then
    hint_message "you can find all the details about how to run the E2E tests in the dedicated README page."
    hint_message "you can find that page at ../README.md."
    exit 1
fi

# generate new rsa keys
./gen-keys.sh

# initialize index repos
cd index
./init-repo.sh

cd ..

function cleanup {
    docker-compose ${FLAGS} down -t 2
    exit 1
}

trap 'cleanup' ERR

docker-compose ${FLAGS} build
docker-compose ${FLAGS} run runner
docker-compose ${FLAGS} down -t 2
