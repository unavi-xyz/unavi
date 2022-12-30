FROM ipfs/kubo

COPY /docker/start_ipfs.sh /start_ipfs.sh

# Fix permissions on start_ipfs (ignore the build machine's permissions)
RUN chmod 0755 /start_ipfs.sh

# This just makes sure that:
# 1. There's an fs-repo, and initializes one if there isn't.
# 2. The API and Gateway are accessible from outside the container.
ENTRYPOINT ["/sbin/tini", "--", "/start_ipfs.sh"]

# Execute the daemon subcommand
CMD ["daemon", "--migrate=true", "--agent-version-suffix=docker"]