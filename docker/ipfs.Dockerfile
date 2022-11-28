FROM ipfs/kubo

# This just makes sure that:
# 1. There's an fs-repo, and initializes one if there isn't.
# 2. The API and Gateway are accessible from outside the container.
COPY ./docker/start_ipfs.sh /start_ipfs.sh
ENTRYPOINT ["/sbin/tini", "--", "/start_ipfs.sh"]

# Execute the daemon subcommand by default
CMD ["daemon", "--migrate=true", "--agent-version-suffix=docker"]