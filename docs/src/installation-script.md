Installation script
===================

Rather than reading paragraphs of text, you may prefer to learn the steps to setup an Alexandrie instance through a simple shell script.  

Here is such a script that you can use to configure an initial instance of Alexandrie, but keep in mind that:  

**This script is not a substitute for the actual documentation.**  
**You may need to configure/change a few things, even after running the script.**  
**It is only there to help you get started and/or be a more concrete documentation resource.**  

```bash
#!/bin/bash

# function to run when an error is encountered
function setup_error {
    echo "-------- An error occurred during configuration --------"
    exit 1
}

# exit on error
trap 'setup_error' ERR

# directory to clone Alexandrie into:
ALEXANDRIE_DIR="";

# URL to the crate index repository.
CRATE_INDEX_GIT_URL="$1";

while [ -z "$ALEXANDRIE_DIR" ]; then
    read -p 'Directory to clone/find Alexandrie into: ' ALEXANDRIE_DIR;
fi

while ! git ls-remote -h $CRATE_INDEX_GIT_URL; do
    read -p 'CRATE_INDEX_GIT_URL: ' CRATE_INDEX_GIT_URL;
done

if ! cargo -V; then
    echo;
    echo "In order to build an instance of Alexandrie, you need to have Rust installed on your system";
    echo "You can learn how to install Rust on your system on the official Rust website:";
    echo "https://www.rust-lang.org/tools/install";
    echo;
    ! :;    # trigger error trap
fi

if [ -d "$ALEXANDRIE_DIR" ]; then
    echo
    echo "'{}' (ALEXANDRIE_DIR) is an existing directory, pulling latest changes ...";
    cd "$ALEXANDRIE_DIR";
    git pull;
    echo "Changes have been pulled successfully !";
    echo;
else
    echo;
    echo "Cloning Alexandrie in '$ALEXANDRIE_DIR' ...";
    git clone https://github.com/Hirevo/alexandrie.git "$ALEXANDRIE_DIR";
    cd "$ALEXANDRIE_DIR";
    echo "Successfully cloned Alexandrie !";
    echo;
fi

echo "Building Alexandrie (using the default features)...";
echo "(keep in mind that the default features may not fit your use-case, be sure to review them before deplying it to production)";
cargo build -p alexandrie;
echo "Alexandrie has been built successfully !";

# create the directory serving as the storage of crate archives.
mkdir -p crate-storage;

# setup the crate index.
if [ -d crate-index ]; then
    echo;
    echo "'${ALEXANDRIE_DIR}/crate-index' is an existing directory, pulling latest changes ...";
    cd crate-index;
    git pull;
    echo "Changes have been pulled successfully !";
    echo;
else
    echo;
    echo "Cloning crate index in '${ALEXANDRIE_DIR}/crate-index' ...";
    git clone "$CRATE_INDEX_GIT_URL" crate-index;
    cd crate-index;
    echo "Successfully cloned the crate index !";
    echo;
fi

# configure the crate index
if [ ! -f config.json ]; then
    echo "The crate index does not have a 'config.json' file.";
    echo "Creating an initial one (please also review it before deploying the registry in production) ..."
    cat > config.json << EOF;
{
    "dl": "http://$(hostname):3000/api/v1/crates/{crate}/{version}/download",
    "api": "http://$(hostname):3000",
    "allowed-registries": ["https://github.com/rust-lang/crates.io-index"]
}
EOF
    git add config.json;
    git commit -m 'Added `config.json`';
    git push -u origin master;
    echo "Initial 'config.json' file has been created and pushed to the crate index !";
    echo;
fi

echo "Alexandrie should be good to go for an initial run.";
echo "You can start the Alexandrie instance by:";
echo "  - navigating to '${ALEXANDRIE_DIR}'";
echo "  - tweaking the 'alexandrie.toml' file";
echo "  - run `./target/debug/alexandrie`";
echo;
```
