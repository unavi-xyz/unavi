import { Device } from "mediasoup-client";
import { createContext, useEffect, useState } from "react";
import { Socket, io } from "socket.io-client";

import {
  ConnectTransportResponseSchema,
  ConsumeAudioResposneSchema,
  CreateTransportResponseSchema,
  GetRouterRtpCapabilitiesResponseSchema,
  JoinSpaceResponseSchema,
  NewConsumerDataSchema,
  ProduceResponseSchema,
} from "./schemas";
import { SentWebsocketMessage, SocketEvents } from "./types";

type TypedSocket = Socket<SocketEvents, SocketEvents>;

export const NetworkingContext = createContext<{
  socket: WebSocket | undefined;
  sendMessage: (message: SentWebsocketMessage) => void;
}>({
  socket: undefined,
  sendMessage: () => {},
});

interface NetworkingProviderProps {
  spaceId: string;
  host: string;
  handle?: string;
  children: React.ReactNode;
}

export function NetworkingProvider({
  spaceId,
  host,
  handle,
  children,
}: NetworkingProviderProps) {
  const [socket, setSocket] = useState<TypedSocket>();
  const [device, setDevice] = useState<Device>();

  //create the websocket connection
  useEffect(() => {
    if (!spaceId || !host) {
      setSocket(undefined);
      return;
    }

    const newSocket: TypedSocket = io(host);
    setSocket(newSocket);

    //connect to websocket server
    newSocket.on("connect", () => {
      console.info("✅ Connected to host server");

      //join the space

      newSocket.emit(
        "join_space",
        {
          spaceId,
        },
        (res) => {
          try {
            const { success } = JoinSpaceResponseSchema.parse(res);

            if (!success) {
              throw new Error("Failed to join space");
            }
            console.info("✅ Joined space");

            //get router rtp capabilities
            newSocket.emit("get_router_rtp_capabilities", async (res) => {
              try {
                const { success, routerRtpCapabilities } =
                  GetRouterRtpCapabilitiesResponseSchema.parse(res);

                if (!success || !routerRtpCapabilities) {
                  throw new Error("Failed to get router rtp capabilities");
                }
                console.info("✅ Got router RTP capabilities");

                //create mediasoup device
                try {
                  const newDevice = new Device();
                  await newDevice.load({
                    routerRtpCapabilities,
                  });
                  setDevice(newDevice);
                } catch (error) {
                  console.info("❌ Error creating mediasoup device");
                  console.error(error);
                  setDevice(undefined);
                  return;
                }
              } catch (error) {
                console.error(error);
              }
            });
          } catch (error) {
            console.error(error);
            return;
          }
        }
      );
    });

    newSocket.on("connect_error", () => {
      console.info("❌ Error connecting to host server");
    });

    newSocket.on("disconnect", () => {
      console.info("❌ Disconnected from host server");
    });

    return () => {
      newSocket.close();
    };
  }, [spaceId, host]);

  useEffect(() => {
    if (!socket || !device) return;

    //👩‍🍳 PRODUCER
    //create producer transport
    socket.emit("create_audio_producer_transport", async (res) => {
      try {
        const { success, params } = CreateTransportResponseSchema.parse(res);

        if (!success || !params) {
          throw new Error("Failed to create producer transport");
        }
        console.info("✅ Created producer transport");

        //create local transport
        const transport = device.createSendTransport(params as any);

        //handle connect
        transport.on("connect", ({ dtlsParameters }, callback, errcallback) => {
          socket.emit(
            "connect_audio_producer_transport",
            { dtlsParameters },
            (res) => {
              try {
                const { success } = ConnectTransportResponseSchema.parse(res);

                if (!success) {
                  throw new Error("Failed to connect producer transport");
                }

                callback();
              } catch (error) {
                console.error(error);
                errcallback();
              }
            }
          );
        });

        //log connection state
        transport.on("connectionstatechange", (state) => {
          switch (state) {
            case "connected":
              console.info("✅👩‍🍳 Producer transport connected");
              break;

            case "connecting":
              console.info("🔄👩‍🍳 Producer transport connecting");
              break;

            case "failed":
              console.info("❌👩‍🍳 Producer transport connection failed");
              transport.close();
              break;

            default:
              break;
          }
        });

        //handle produce
        transport.on("produce", (params, callback, errcallback) => {
          const { kind, rtpParameters } = params;

          socket.emit(
            "produce_audio",
            {
              kind,
              rtpParameters,
            },
            (res) => {
              try {
                const { success, id } = ProduceResponseSchema.parse(res);

                if (!success || !id) {
                  throw new Error("Failed to produce");
                }

                callback({ id });
              } catch (error) {
                console.error(error);
                errcallback();
              }
            }
          );
        });

        //get audio stream
        const stream = await navigator.mediaDevices.getUserMedia({
          audio: true,
        });
        const track = stream.getAudioTracks()[0];

        //produce audio track
        await transport.produce({ track });
      } catch (error) {
        console.error(error);
      }
    });

    //🍔 CONSUMER
    //create consumer transport
    socket.emit("create_audio_consumer_transport", async (res) => {
      try {
        const { success, params } = CreateTransportResponseSchema.parse(res);

        if (!success || !params) {
          throw new Error("Failed to create consumer transport");
        }
        console.info("✅ Created consumer transport");

        //create local transport
        const transport = device.createRecvTransport(params as any);

        //handle connect
        transport.on("connect", ({ dtlsParameters }, callback, errcallback) => {
          socket.emit(
            "connect_audio_consumer_transport",
            { dtlsParameters },
            (res) => {
              try {
                const { success } = ConnectTransportResponseSchema.parse(res);

                if (!success) {
                  throw new Error("Failed to connect producer transport");
                }

                callback();
              } catch (error) {
                console.error(error);
                errcallback();
              }
            }
          );
        });

        //log connection state
        transport.on("connectionstatechange", (state) => {
          switch (state) {
            case "connected":
              console.info("✅🍔 Consumer transport connected");
              break;

            case "connecting":
              console.info("🔄🍔 Consumer transport connecting");
              break;

            case "failed":
              console.info("❌🍔 Consumer transport connection failed");
              transport.close();
              break;

            default:
              break;
          }
        });

        //start consuming
        socket.emit(
          "consume_audio",
          {
            rtpCapabilities: device.rtpCapabilities,
          },
          (res) => {
            try {
              const { success } = ConsumeAudioResposneSchema.parse(res);
              if (!success) {
                throw new Error("Failed to consume audio");
              }
            } catch (error) {
              console.error(error);
            }
          }
        );

        //receive new consumers
        socket.on("new_consumer", async (data) => {
          try {
            const { producerId, id, kind, rtpParameters } =
              NewConsumerDataSchema.parse(data);

            if (transport.closed) throw new Error("Transport closed");

            console.info("✨🍔 New consumer");

            //start consuming
            const consumer = await transport.consume({
              producerId,
              id,
              kind,
              rtpParameters,
            });

            const stream = new MediaStream();
            stream.addTrack(consumer.track);

            //play stream
            function startAudio() {
              const audio = document.createElement("audio");
              document.body.appendChild(audio);
              audio.srcObject = stream;
              audio.play();
            }

            document.addEventListener("click", startAudio, { once: true });
          } catch (error) {
            console.error(error);
          }
        });
      } catch (error) {
        console.error(error);
      }
    });
  }, [socket, device]);

  return (
    <NetworkingContext.Provider
      value={{
        socket: undefined,
        sendMessage: (message: SentWebsocketMessage) => {},
      }}
    >
      {children}
    </NetworkingContext.Provider>
  );
}
