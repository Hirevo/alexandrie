Available crate stores
======================

'disk': Local on-disk store
---------------------------

This strategy implements simple local storage of crates as files in a given directory.

Here is an example of configuration to use this storage strategy:

```toml
[storage]
type = "disk"           # required.
path = "crate-storage"  # required: path of the directory in which to store the crates.
```

's3': AWS S3 object storage
---------------------------

This strategy stores crate archives and READMEs as objects within an AWS S3 bucket.
To use the S3 storage type the feature `s3` needs to be enabled when compiling alexandrie.
When using cargo to build use:

```
cargo build --release --features s3
```

Here is an example of configuration to use this storage strategy:

```toml
[storage]
type = "s3"                     # required.
region = ["eu-west-1"]          # required: name of the operating region of the S3 bucket.
bucket = "eu-polomack-crates"   # required: name of the S3 bucket to use.
key_prefix = "crates"           # optional: arbitrary prefix to apply on the objects' keys
                                #           allowing to place them in subdirectories.
```

You can specify a custom S3 endpoint, instead of the official S3 ones, using the `region` key, like this:

```toml
region = ["custom", "https://my.custom.s3.endpoint/"]
```

### S3 Authentication

In order to authenticate the registry to S3, you can either:

- define both `AWS_ACCESS_KEY_ID` and `AWS_SECRET_ACCESS_KEY` environment variables when running the registry.
- have a `credential_process` command specified in `~/.aws/config`.
- have credentials stored in `~/.aws/credentials` and giving the registry read permission to it.
- using an IAM instance profile, which will only work if running on an EC2 instance with an instance profile/role assigned.

These different options are attempted in that same order, whichever is found to have valid credentials first.

> For more details on how authentication is resolved, you can refer to the Rusoto's documentation on that matter.
