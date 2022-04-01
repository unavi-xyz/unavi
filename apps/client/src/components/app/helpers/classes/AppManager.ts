import { UseBoundStore, StoreApi } from "zustand";
import { createMessage } from "../message";

import { AppStore } from "../store";
import { Channels, Identity, Message } from "../types";

type useStoreType = UseBoundStore<AppStore, StoreApi<AppStore>>;
type StoredChannels = Record<keyof Channels, RTCDataChannel[]>;

export class AppManager {
  useStore: useStoreType;

  identity: Identity;
  channels: StoredChannels = { identity: [], message: [], transform: [] };

  constructor(useStore: useStoreType) {
    this.useStore = useStore;
  }

  publishAll<T extends keyof Channels>(type: T, value: Channels[T]) {
    this.channels[type].forEach((channel) => {
      this.publish(channel, value);
    });

    if (type === "message") {
      const localMessage = createMessage(value as string);
      this.addMessage(localMessage);
    }
  }

  publish(channel: RTCDataChannel, value: any) {
    if (channel.readyState === "open") {
      channel.send(JSON.stringify(value));
      return;
    }

    channel.onopen = () => channel.send(JSON.stringify(value));
  }

  addChannel(type: keyof Channels, channel: RTCDataChannel) {
    this.channels[type].push(channel);

    if (type === "identity") {
      this.publish(channel, this.identity);
    }
  }

  addMessage(message: Message) {
    const newMessages = [message, ...this.useStore.getState().messages];
    this.useStore.setState({ messages: newMessages });
  }

  setIdentity(identity: Identity) {
    this.identity = identity;
    this.publishAll("identity", identity);
  }
}
