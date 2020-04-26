#!/bin/bash

function trigger_error {
    return 1
}

function wait_for_registry {
    echo
    echo -n "Waiting for the registry to become available... "
    STATUS=1
    # give the database 2 mins to start
    TIMEOUT=120
    ELAPSED=0
    while [ ${STATUS} -ne 0 ]; do
        # the following trick with '&&' et '||' allows the current ERR trap to not be triggered.
        curl "${REGISTRY_URL}" > /dev/null 2>&1 && STATUS=$? || STATUS=$?
        if [ ${STATUS} -ne 0 ]; then
            sleep 2
            ELAPSED=$((${ELAPSED} + 2))
            if [ ${ELAPSED} -ge ${TIMEOUT} ]; then
                exit ${STATUS}
            fi
        fi
    done
    echo "OK !"
    echo
}

##### SSH config #####

function ssh_setup_error {
    echo "-------- SSH setup error --------"
    exit 1
}

trap 'ssh_setup_error' ERR

# # create `${HOME}/.ssh/` directory
# mkdir -p "${HOME}/.ssh"

# # copy private key to that `${HOME}/.ssh/` folder
# cp "/run/secrets/git_ssh_key" "${HOME}/.ssh/id_rsa"
# chmod 400 "${HOME}/.ssh/id_rsa"

# fetch public key of the crate index
ssh-keyscan -t rsa index >> "${HOME}/.ssh/known_hosts"

# start the ssh agent
eval $(ssh-agent)

# add ssh keys
ssh-add

##### Scenario config #####

function scenario_setup_error {
    echo "-------- Scenario setup error --------"
    exit 1
}

trap 'scenario_setup_error' ERR

cp -r "${HOME}/original" "${HOME}/scenario"
cd "${HOME}/scenario"

##### Runner config #####

function runner_setup_error {
    echo "-------- Runner setup error --------"
    exit 1
}

trap 'runner_setup_error' ERR

# setup a few constants
export USER="runner"
export REGISTRY_NAME='alternative'
export REGISTRY_URL='http://registry:3000'
export INDEX_URL='git@index:/git-server/repos/index.git'
export AUTHOR_EMAIL='john.doe@example.com'
export AUTHOR_NAME='John Doe'
export AUTHOR_PASSWD='test'

# this is necessary for Cargo to make it use the configured git CLI tool instead of libgit2
export CARGO_NET_GIT_FETCH_WITH_CLI="true"

# configure Cargo for the alternative registry
# (and make it the default one to avoid publishing to crates.io by mistake)
mkdir -p "${HOME}/.cargo"
cat > "${HOME}/.cargo/config" << EOF
[registry]
default = "${REGISTRY_NAME}"

[registries.${REGISTRY_NAME}]
index = "ssh://${INDEX_URL}"
EOF

wait_for_registry

# create an account in the registry and get back the authentication token
export REGISTRY_TOKEN="$(curl -X POST "${REGISTRY_URL}/api/v1/account/register" \
    -H 'Content-Type: application/json; charset=utf-8' \
    -d "{\"email\":\"${AUTHOR_EMAIL}\",\"name\":\"${AUTHOR_NAME}\",\"passwd\":\"${AUTHOR_PASSWD}\"}" \
    | jq --raw-output '.token')"

if [ "${REGISTRY_TOKEN}" = "null" ]; then
    echo "error: failed to get a token from the registry."
    trigger_error
fi

# pass the token over to Cargo
cargo login --registry "${REGISTRY_NAME}" -- "${REGISTRY_TOKEN}"

trap - ERR

# start the scenario
./runner.sh
