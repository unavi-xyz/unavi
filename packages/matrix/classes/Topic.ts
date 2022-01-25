const KEY_CHAR = "#key#";
const VALUE_CHAR = "#value#";

export class Topic {
  string: string;

  constructor(value?: string) {
    this.string = value ?? "";
  }

  setKey(key: string, value: string) {
    const str = `${this.string}${KEY_CHAR}${key}${VALUE_CHAR}${value}`;
    this.string = str;
  }

  getKey(key: string) {
    const obj: any = this.parse();
    const value: string = obj[key];
    return value;
  }

  parse() {
    const obj: { [key: string]: string } = {};

    const split = this.string.split(KEY_CHAR);

    split.forEach((str) => {
      const pair = str.split(VALUE_CHAR);

      if (pair.length > 1) {
        obj[pair[0]] = pair[1];
      }
    });

    return obj;
  }
}
