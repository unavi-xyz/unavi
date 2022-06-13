defmodule Host.SocketHandler do
  @behaviour :cowboy_websocket


  def init(request, _state) do
    IO.puts("Host.SocketHandler.init")

    state = %{space_id: request.path, user_id: UUID.uuid4(), handle: nil}

    {:cowboy_websocket, request, state}
  end

  def websocket_init(state) do

    Registry.Host
    |> Registry.register(state.space_id, {})

    {:ok, state}
  end

  def websocket_handle({:text, json}, state) do
    payload = Jason.decode!(json)
    type = payload["type"]
    data = payload["data"]

    case type do
      "identity" ->
        IO.puts("identity from #{state.user_id}")

        state = %{state | handle: data}

        {:ok, state}

      "chatmessage" ->
        IO.puts("chatmessage from #{state.user_id}")

        username = state.handle || get_guest_name(state)

        chatMessage = %{
          type: "chatmessage",
          data: %{
            id: UUID.uuid4(),
            timestamp: :os.system_time(),
            username: username,
            message: data
          }
        }

        text = Jason.encode!(chatMessage)

        # broadcast to all connections in space
        Registry.Host
        |> Registry.dispatch(state.space_id, fn entries ->
          for {pid, _} <- entries do
            Process.send(pid, {:text, text}, [])
          end
        end)

        {:ok, state}

      "location" ->
        locationMessage = %{
          type: "location",
          data: %{
            userid: state.user_id,
            handle: state.handle,
            location: data
          }
        }

        text = Jason.encode!(locationMessage)

        # broadcast to all other connections in space
        Registry.Host
        |> Registry.dispatch(state.space_id, fn entries ->
          for {pid, _} <- entries do
            if pid != self() do
              Process.send(pid, {:text, text}, [])
            end
          end
        end)

        {:ok, state}

      _ ->
        {:error, "unknown type"}
    end
  end

  def websocket_info(info, state) do
    {:reply, info, state}
  end

  defp get_guest_name(state) do
    <<head::binary-size(6)>> <> _ = state.user_id
    "Guest-#{head}"
  end
end
