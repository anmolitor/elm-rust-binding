import { Elm } from "../binding2.js";

export default () => {
  return new Promise((resolve) => {
    const elm = Elm.Binding.init({ flags: 5 });
    elm.ports.out.subscribe((output) => {
      resolve(output);
    });
  });
};
