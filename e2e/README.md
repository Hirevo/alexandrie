End-to-End Testing
==================

This directory is dedicated to the end-to-end testing infrastructure of Alexandrie.  
End-to-End, here, means using the registry just like a user would and configuring it like it was going into production.  
This helps ensuring that nothing ever breaks when making changes to Alexandrie.  

How it works
------------

End-to-end means doing things just like any user would.  
For us, it means:

- spawning a git server to host the index repository
- spawning an instance of Alexandrie which makes makes use of this index
- spawning a sample client that runs `cargo` commands against the registry

These three distinct parts are implemented as `Docker` containers, coordinated by `docker-compose`.  

There are 3 services defined in `docker-compose.yaml`:

- **`index`**: this is the git server hosting the index repository.
- **`registry`**: this is the Alexandrie instance itself, that is configured to interact with **`index`**.
- **`runner`**: this is the client that interacts with Alexandrie (**`registry`**), as a normal user would.

Each end-to-end test cases are stored in what are called **scenarios**.  
A **scenario** is a sequence of commands and operations that represents a certain use-case.  
The **`runner`** is what takes care of running these **`scenarios`**.  

Here is a run-down of the directory hierarchy:

- **`images/`**: contains all the implementation bits of the end-to-end testing setup.
  - **`images/index/`**: contains the files and scripts required to start the git index (like the initial bare repository).
  - **`images/keys/`**: contains the SSH keys used to interact with the git index.
  - **`images/runner/`**: contains the files and scripts required to start the scenario runner.
  - **`images/gen-keys.sh`**: generates new SSH keys in **`images/keys/`**.
  - **`images/run-scenario.sh`**: takes care of everything to run a scenario.
  - **`images/docker-compose.yaml`**: contains the definition of the main Docker containers.
  - **`images/mysql-compose.yaml`**: extensions to **`images/docker-compose.yaml`** for MySQL-specific services.
  - **`images/postgres-compose.yaml`**: extensions to **`images/docker-compose.yaml`** for PostgreSQL-specific services.
- **`scenarios/`**: contains the different test scenarios that can be ran.
  - **`scenarios/<SCENARIO>/`**: contains the files and scripts needed to run the scenario of the same name.

How to run a scenario
---------------------

To run a scenario, simply navigate to the **`images/`** directory and run:

```bash
# Replace the '<...>' placeholders by the real values.
export DATABASE="<mysql|postgres|sqlite>"
export SCENARIO="<name-of-the-scenario>"
./run-scenario

# To allow the containers to read the different mounted volumes (like the SSH keys),
# by default, the UID and GID of your current user are passed to the Dockerfiles.
# So, you may encounter an issue like one of the following:
#  - groupadd: GID '..' already exists
#  - useradd: UID '..' already exists
# If so, then you can specify another UID and GID by:
#  - setting the USER_ID and GROUP_ID environment variables
#  - uncommenting the line in `run-scenario.sh` about using `chmod`
#  - re-running `run-scenario.sh`
```

It will take care of everything needed (generating new SSH keys, creating a blank repository, building and running the images to completion).

An non-zero exit code means that the scenario failed (the output logs should give more details about the why).

How to define a scenario
------------------------

To define a scenario, simply create a new folder in **`scenarios/`**, the name of the scenario is the name you choose for that folder.  
Within that folder, the **`runner`** will expect to find a **`runner.sh`** file with executable rights.  
This script should specify the `#!/bin/bash` shebang, this is the shell that we will make sure is always present within the container.  
What you do within this script is entirely up to you to define (based on what is the use-case that you want to test), but keep in mind that:

- An exit code of 0 indicates a successful run of the scenario, a non-zero one indicates a failure.
- Make sure that your script exits as a failure if one command fails, here are some ways to do this:
  - You can use `set -e` at the beginning of your script to automatically exit if any error is encountered.
  - You can also trap any error and execute a bash function of yours to deal with it by doing `trap 'name_of_my_function' ERR`.  
    In this case, do not forget to add something like `exit 1` at the end of that bash function to still signal the failure to the **`runner`**.
