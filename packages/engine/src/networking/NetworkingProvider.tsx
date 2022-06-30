import produce from "immer";
import { Device } from "mediasoup-client";
import { DataConsumer } from "mediasoup-client/lib/DataConsumer";
import { DataProducer } from "mediasoup-client/lib/DataProducer";
import { createContext, useEffect, useState } from "react";
import { Socket, io } from "socket.io-client";

import {
  ConnectTransportResponseSchema,
  ConsumeAudioResposneSchema,
  CreateTransportResponseSchema,
  GetRouterRtpCapabilitiesResponseSchema,
  JoinSpaceResponseSchema,
  NewAudioConsumerDataSchema,
  NewDataConsumerDataSchema,
  ProduceAudioResponseSchema,
  ProduceDataResponseSchema,
} from "./schemas";
import { SocketEvents } from "./types";

type TypedSocket = Socket<SocketEvents, SocketEvents>;

type DataConsumers = {
  [key: string]: DataConsumer;
};

export const NetworkingContext = createContext<{
  dataProducer: DataProducer | undefined;
  dataConsumers: DataConsumers;
}>({
  dataProducer: undefined,
  dataConsumers: {},
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

  const [dataProducer, setDataProducer] = useState<DataProducer>();
  const [dataConsumers, setDataConsumers] = useState<DataConsumers>({});

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
    socket.emit("create_producer_transport", async (res) => {
      try {
        const { success, params } = CreateTransportResponseSchema.parse(res);

        if (!success || !params) {
          throw new Error("Failed to create producer transport");
        }

        //create local transport
        const transport = device.createSendTransport(params as any);

        //handle connect
        transport.on("connect", ({ dtlsParameters }, callback, errcallback) => {
          socket.emit(
            "connect_producer_transport",
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
                const { success, id } = ProduceAudioResponseSchema.parse(res);

                if (!success || !id) {
                  throw new Error("Failed to produce audio");
                }

                callback({ id });
              } catch (error) {
                console.error(error);
                errcallback();
              }
            }
          );
        });

        //handle produce data
        transport.on("producedata", (params, callback, errcallback) => {
          const { sctpStreamParameters } = params;

          socket.emit(
            "produce_data",
            {
              sctpStreamParameters,
            },
            (res) => {
              try {
                const { success, id } = ProduceDataResponseSchema.parse(res);

                if (!success || !id) {
                  throw new Error("Failed to produce data");
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
        const audioProducer = await transport.produce({ track });

        //produce data
        const newDataProducer = await transport.produceData();

        newDataProducer.on("open", () => {
          setDataProducer(newDataProducer);
        });

        newDataProducer.on("close", () => {
          setDataProducer(undefined);
        });
      } catch (error) {
        console.error(error);
      }
    });

    //🍔 CONSUMER
    //create consumer transport
    socket.emit("create_consumer_transport", async (res) => {
      try {
        const { success, params } = CreateTransportResponseSchema.parse(res);

        if (!success || !params) {
          throw new Error("Failed to create consumer transport");
        }

        //create local transport
        const transport = device.createRecvTransport(params as any);

        //handle connect
        transport.on("connect", ({ dtlsParameters }, callback, errcallback) => {
          socket.emit(
            "connect_consumer_transport",
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
        socket.on("new_audio_consumer", async (data) => {
          try {
            const { producerId, id, kind, rtpParameters } =
              NewAudioConsumerDataSchema.parse(data);

            if (transport.closed) throw new Error("Transport closed");

            console.info("✨🍔 New audio consumer");

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

        socket.on("new_data_consumer", async (data) => {
          try {
            const { dataProducerId, sctpStreamParameters, id } =
              NewDataConsumerDataSchema.parse(data);

            if (transport.closed) throw new Error("Transport closed");

            if (!sctpStreamParameters)
              throw new Error("No sctpStreamParameters");

            if (!dataProducerId) throw new Error("No dataProducerId");

            console.info("✨🍔 New data consumer");

            //start consuming
            const consumer = await transport.consumeData({
              dataProducerId,
              id,
              sctpStreamParameters,
            });

            setDataConsumers(
              produce((draft) => {
                draft[consumer.id] = consumer;
              })
            );
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
        dataProducer,
        dataConsumers,
      }}
    >
      {children}
    </NetworkingContext.Provider>
  );
}
