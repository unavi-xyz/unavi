import { POSITION_ARRAY_ROUNDING, ROTATION_ARRAY_ROUNDING } from "../constants";
import { Engine } from "../Engine";

const UPDATE_HZ = 15;

/**
 * Manages the audio context and audio elements.
 *
 * @group Modules
 */
export class AudioModule {
  readonly #engine: Engine;

  readonly context = new AudioContext();
  readonly audios = new Set<HTMLAudioElement>();
  readonly autoPlay = new WeakMap<HTMLAudioElement, boolean>();

  #interval: NodeJS.Timeout | null = null;

  constructor(engine: Engine) {
    this.#engine = engine;
  }

  async start() {
    this.context.resume();

    // Try to play audio now
    this.#playAudio();

    // Play audio on user interaction
    const play = () => {
      if (this.context.state === "suspended") this.#playAudio();
      if (this.context.state === "running") {
        document.removeEventListener("click", play);
        document.removeEventListener("touchstart", play);
      }
    };

    document.addEventListener("click", play);
    document.addEventListener("touchstart", play);

    this.#interval = setInterval(() => {
      const camPosX = Atomics.load(this.#engine.cameraPosition, 0) / POSITION_ARRAY_ROUNDING;
      const camPosY = Atomics.load(this.#engine.cameraPosition, 1) / POSITION_ARRAY_ROUNDING;
      const camPosZ = Atomics.load(this.#engine.cameraPosition, 2) / POSITION_ARRAY_ROUNDING;
      const camYaw = Atomics.load(this.#engine.cameraYaw, 0) / ROTATION_ARRAY_ROUNDING;

      // Set audio listener location
      const listener = this.context.listener;

      if (listener.positionX !== undefined) {
        listener.positionX.value = camPosX;
        listener.positionY.value = camPosY;
        listener.positionZ.value = camPosZ;
      } else {
        listener.setPosition(camPosX, camPosY, camPosZ);
      }

      if (listener.forwardX !== undefined) {
        listener.forwardX.value = Math.sin(camYaw);
        listener.forwardZ.value = Math.cos(camYaw);
      } else {
        listener.setOrientation(Math.sin(camYaw), 0, Math.cos(camYaw), 0, 1, 0);
      }
    }, 1000 / UPDATE_HZ);
  }

  #playAudio() {
    if (!this.#engine.isPlaying) return;

    this.audios.forEach((audio) => {
      if (this.autoPlay.get(audio) === true) {
        audio.play();
      }
    });
  }

  stop() {
    if (this.#interval) clearInterval(this.#interval);

    this.context.suspend();

    for (const audio of this.audios) {
      audio.pause();
      audio.currentTime = 0;
    }
  }

  createAudio() {
    const audio = new Audio();
    audio.crossOrigin = "anonymous";

    this.audios.add(audio);

    return audio;
  }

  createAudioPanner() {
    const panner = this.context.createPanner();
    panner.panningModel = "HRTF";
    panner.connect(this.context.destination);

    return panner;
  }

  removeAudio(audio: HTMLAudioElement) {
    this.audios.delete(audio);
    this.autoPlay.delete(audio);
    audio.remove();
  }

  async setAutoPlay(audio: HTMLAudioElement, value: boolean) {
    this.autoPlay.set(audio, value);

    if (value && this.#engine.isPlaying) {
      await audio.play();
    } else if (!value && this.#engine.isPlaying) {
      audio.pause();
      audio.currentTime = 0;
    }
  }

  destroy() {
    this.stop();
    this.context.close();
    this.audios.forEach((audio) => this.removeAudio(audio));
  }
}
