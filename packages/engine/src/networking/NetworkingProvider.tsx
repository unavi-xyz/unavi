import { Device } from "mediasoup-client";
import { createContext, useEffect, useState } from "react";
import { Socket, io } from "socket.io-client";

import {
  ConnectTransportSchema,
  CreateTransportResponseSchema,
  GetRouterRtpCapabilitiesResponseSchema,
  JoinSpaceData,
  JoinSpaceResponseSchema,
  NewConsumerDataSchema,
  ProduceResponseSchema,
} from "./schemas";
import { SentIdentity, SentWebsocketMessage } from "./types";

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
  const [socket, setSocket] = useState<Socket>();
  const [device, setDevice] = useState<Device>();

  //create the websocket connection
  useEffect(() => {
    if (!spaceId || !host) {
      setSocket(undefined);
      return;
    }

    const newSocket = io(host);
    setSocket(newSocket);

    //connect to websocket server
    newSocket.on("connect", () => {
      console.info("✅ Connected to host server");

      //join the space
      const data: JoinSpaceData = {
        spaceId,
      };

      newSocket.emit("join_space", data, (res: any) => {
        try {
          const { success } = JoinSpaceResponseSchema.parse(res);

          if (!success) {
            throw new Error("Failed to join space");
          }
          console.info("✅ Joined space");

          //get router rtp capabilities
          newSocket.emit("get_router_rtp_capabilities", async (res: any) => {
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
      });
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
    socket.emit("create_audio_producer_transport", async (res: any) => {
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
            (res: any) => {
              try {
                const { success } = ConnectTransportSchema.parse(res);

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
              transportId: transport.id,
              kind,
              rtpParameters,
            },
            (res: any) => {
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
    socket.emit("create_audio_consumer_transport", async (res: any) => {
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
            (res: any) => {
              try {
                const { success } = ConnectTransportSchema.parse(res);

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
        socket.emit("consume_audio", {
          rtpCapabilities: device.rtpCapabilities,
        });

        //receive new consumers
        socket.on("new_consumer", async (data: any) => {
          try {
            const options = NewConsumerDataSchema.parse(data);

            console.info("✨🍔 New consumer");

            if (transport.closed) return;

            //start consuming
            const { producerId, id, kind, rtpParameters } = options;
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
