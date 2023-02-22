import { Group, Mesh, MeshBasicMaterial, Shape, ShapeGeometry } from "three";
import { Font, FontLoader } from "three/examples/jsm/loaders/FontLoader";

export class Nameplate {
  group = new Group();

  #name: string | null = null;
  #font: Font | null = null;
  #nameplate: Mesh | null = null;
  #loadPromise: Promise<void> | null = null;

  static TEXT_MATERIAL = new MeshBasicMaterial({ color: 0xffffff });
  static BG_MATERIAL = new MeshBasicMaterial({ color: 0x010101, opacity: 0.95, transparent: true });

  get name(): string | null {
    return this.#name;
  }

  set name(value: string | null) {
    if (this.#name === value) return;

    this.#name = value;

    if (!value) return;

    // Load font if not loaded yet
    if (!this.#loadPromise) this.#loadPromise = this.#loadFont();

    // Create nameplate when font is loaded
    this.#loadPromise.then(() => this.#createNameplate(value));
  }

  async #loadFont() {
    const loader = new FontLoader();
    this.#font = await loader.loadAsync(new URL("./font.json", import.meta.url).href);
  }

  #createNameplate(text: string | null) {
    if (!this.#font) throw new Error("Font not loaded");

    // Remove old nameplate
    if (this.#nameplate) {
      this.group.remove(this.#nameplate);
      this.#nameplate.traverse((object) => {
        if (object instanceof Mesh) object.geometry.dispose();
      });
      this.#nameplate = null;
    }

    if (!text) return;

    // Create text
    const shapes = this.#font.generateShapes(text, 0.075);
    const textGeometry = new ShapeGeometry(shapes);

    // Center horizontally
    textGeometry.computeBoundingBox();
    if (!textGeometry.boundingBox) throw new Error("No bounding box");
    const xMid = -0.5 * (textGeometry.boundingBox.max.x - textGeometry.boundingBox.min.x);
    textGeometry.translate(xMid, 0, 0);

    this.#nameplate = new Mesh(textGeometry, Nameplate.TEXT_MATERIAL);

    // Create background
    const width = textGeometry.boundingBox.max.x - textGeometry.boundingBox.min.x + 0.15;
    const height = textGeometry.boundingBox.max.y - textGeometry.boundingBox.min.y + 0.06;
    const radius = height / 2;

    const shape = new Shape();
    shape.moveTo(0, radius);
    shape.lineTo(0, height - radius);
    shape.quadraticCurveTo(0, height, radius, height);
    shape.lineTo(width - radius, height);
    shape.quadraticCurveTo(width, height, width, height - radius);
    shape.lineTo(width, radius);
    shape.quadraticCurveTo(width, 0, width - radius, 0);
    shape.lineTo(radius, 0);
    shape.quadraticCurveTo(0, 0, 0, radius);

    const roundedRectangle = new ShapeGeometry(shape);

    const background = new Mesh(roundedRectangle, Nameplate.BG_MATERIAL);
    background.position.x = -width / 2;
    background.position.y = -height / 4;
    background.position.z = -0.001;

    this.#nameplate.add(background);
    this.group.add(this.#nameplate);
  }
}
