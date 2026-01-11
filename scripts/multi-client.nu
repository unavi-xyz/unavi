# Runs two clients and a server.
# Useful for testing multiplayer.

cargo build -p unavi-server;
cargo build -p unavi-client;

[
  { cmd: { cargo run -p unavi-server } }
  { cmd: { sleep 1sec; cargo run -p unavi-client -- --in-memory } }
  { cmd: { sleep 3sec; cargo run -p unavi-client -- --in-memory } }
] | par-each { |it| do $it.cmd }
