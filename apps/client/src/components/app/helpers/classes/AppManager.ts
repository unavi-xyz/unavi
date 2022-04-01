import { UseBoundStore, StoreApi } from "zustand";

import { Transform } from "../../AppCanvas/Multiplayer/helpers/types";
import { AppStore } from "../store";
import { Identity, Message } from "../types";

type useStoreType = UseBoundStore<AppStore, StoreApi<AppStore>>;

export class AppManager {
  useStore: useStoreType;

  identity: Identity;

  transformChannels = new Set<RTCDataChannel>();
  messageChannels = new Set<RTCDataChannel>();
  identityChannels = new Set<RTCDataChannel>();

  constructor(useStore: useStoreType) {
    this.useStore = useStore;
  }

  setSpaceId(spaceId: AppStore["spaceId"]) {
    this.useStore.setState({ spaceId });
  }

  setMuted(muted: AppStore["muted"]) {
    this.useStore.setState({ muted });
  }

  setChatInputRef(chatInputRef: AppStore["chatInputRef"]) {
    this.useStore.setState({ chatInputRef });
  }

  setIdentity(identity: Identity) {
    this.identity = identity;
    this.publishIdentity();
  }

  publishIdentity() {
    this.transformChannels.forEach((channel) => {
      if (channel.readyState === "open")
        channel.send(JSON.stringify(this.identity));
      else channel.onopen = () => channel.send(JSON.stringify(this.identity));
    });
  }

  addIdentityChannel(channel: RTCDataChannel) {
    this.identityChannels.add(channel);

    if (channel.readyState === "open")
      channel.send(JSON.stringify(this.identity));
    else channel.onopen = () => channel.send(JSON.stringify(this.identity));
  }

  publishTransform(transform: Transform) {
    this.transformChannels.forEach((channel) => {
      if (channel.readyState === "open")
        channel.send(JSON.stringify(transform));
    });
  }

  addTransformChannel(channel: RTCDataChannel) {
    this.transformChannels.add(channel);
  }

  publishMessage(message: Message) {
    this.messageChannels.forEach((channel) => {
      if (channel.readyState === "open") channel.send(JSON.stringify(message));
    });

    const localMessage = { ...message };
    localMessage.username = null;
    this.addMessage(localMessage);
  }

  addMessageChannel(channel: RTCDataChannel) {
    this.messageChannels.add(channel);
  }

  addMessage(message: Message) {
    const newMessages = [message, ...this.useStore.getState().messages];
    this.useStore.setState({ messages: newMessages });
  }
}
