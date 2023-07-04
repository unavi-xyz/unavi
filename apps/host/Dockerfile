FROM node:18-alpine AS builder
WORKDIR /app

# Install global dependencies
RUN npm i -g npm@latest; \
 npm install -g pnpm turbo

# Prune files
COPY . .
RUN pnpm turbo prune --scope=@unavi/host --docker

FROM node:18 AS installer
WORKDIR /app

# Install global dependencies
RUN npm i -g npm@latest; \
 npm install -g pnpm turbo

# Install mediasoup dependencies
RUN \
	set -x \
	&& apt-get update \
	&& apt-get install -y net-tools build-essential python3 python3-pip valgrind

# Install NPM dependencies
COPY .gitignore .gitignore
COPY --from=builder /app/out/json/ .
COPY --from=builder /app/out/pnpm-lock.yaml ./pnpm-lock.yaml
RUN pnpm install

# Build the project and its dependencies
COPY --from=builder /app/out/full/ .
COPY turbo.json turbo.json
RUN pnpm turbo build --filter=@unavi/host...

FROM node:18 AS runner
WORKDIR /app

# Don't run production as root
RUN addgroup --system --gid 1001 nodejs
RUN adduser --system --uid 1001 nodejs
USER nodejs
COPY --from=installer /app .

CMD node --experimental-specifier-resolution=node apps/host/dist/server.js