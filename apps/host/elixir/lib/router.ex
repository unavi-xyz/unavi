defmodule Host.Router do
  use Plug.Router

  plug(Corsica, origins: "*")
  plug(:match)
  plug(:dispatch)

  get "/ping" do
    send_resp(conn, 200, "pong")
  end

  get "/space/:id/player_count" do
    case :ets.lookup(:player_count, id) do
      [{^id, count}] ->
        send_resp(conn, 200, "#{count}")

      [] ->
        send_resp(conn, 200, "0")
    end
  end

  match _ do
    send_resp(conn, 404, "404")
  end
end
