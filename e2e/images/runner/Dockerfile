FROM rust:1.40

# install a few system dependencies
RUN apt update
RUN apt install -y curl jq

# copy the test runner script into the container
COPY ./runner/runner.sh /home/runner/runner.sh

# combine run instructions to reduce docker layers & overall image size
RUN \
    # make a non-root user
    groupadd -g 1000 runner && \
    useradd -u 1000 -g 1000 runner && \
    # make the user directory & give them access to everything in it
    # mkdir -p /home/runner && \
    mkdir -p /home/runner/.ssh && \
    chown -R runner:runner /home/runner && \
    # give runner ownership of the startup script & make it executable
    chmod +x /home/runner/runner.sh

# switch to the non-root user to run the main process
USER runner
WORKDIR /home/runner

CMD [ "./runner.sh" ]
