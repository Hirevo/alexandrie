FROM rust:1.50

# install a few system dependencies
RUN apt update
RUN apt install -y curl jq

# copy the test runner script into the container
COPY ./runner/runner.sh /home/runner/runner.sh

ARG USER_ID=1000
ARG GROUP_ID=1000

# combine run instructions to reduce docker layers & overall image size
RUN \
    # make a non-root user
    groupadd -g ${GROUP_ID} runner && \
    useradd -u ${USER_ID} -g ${GROUP_ID} runner && \
    # make the user directory & give them access to everything in it
    # mkdir -p /home/runner && \
    mkdir -p /home/runner/.ssh && \
    chown -R ${USER_ID}:${GROUP_ID} /home/runner && \
    # give runner ownership of the startup script & make it executable
    chmod +x /home/runner/runner.sh

# switch to the non-root user to run the main process
USER runner
WORKDIR /home/runner

CMD [ "./runner.sh" ]
