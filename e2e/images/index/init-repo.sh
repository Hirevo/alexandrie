#!/bin/bash

# dont allow errors
set -e

rm -rf repos
mkdir -p repos/index
cd repos/index

cat > config.json << EOF
{
    "dl": "http://registry:3000/api/v1/crates/{crate}/{version}/download",
    "api": "http://registry:3000",
    "allowed-registries": ["https://github.com/rust-lang/crates.io-index"]
}
EOF

git init --shared=true
git add config.json
git commit -m 'Added first files'

cd ..
git clone --bare index index.git
rm -rf index

cd ..
