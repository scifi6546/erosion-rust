/* tslint:disable */
/* eslint-disable */
/**
* @param {Map<any, any>} resolution
* @returns {WebGame}
*/
export function init_game(resolution: Map<any, any>): WebGame;
/**
*/
export class ScreenResolution {
  free(): void;
/**
* @param {number} x
* @param {number} y
* @returns {ScreenResolution}
*/
  static new(x: number, y: number): ScreenResolution;
/**
* @returns {number}
*/
  x: number;
/**
* @returns {number}
*/
  y: number;
}
/**
*/
export class WebGame {
  free(): void;
/**
* @param {Array<any>} events
*/
  render_frame(events: Array<any>): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_webgame_free: (a: number) => void;
  readonly webgame_render_frame: (a: number, b: number) => void;
  readonly __wbg_screenresolution_free: (a: number) => void;
  readonly __wbg_get_screenresolution_x: (a: number) => number;
  readonly __wbg_set_screenresolution_x: (a: number, b: number) => void;
  readonly __wbg_get_screenresolution_y: (a: number) => number;
  readonly __wbg_set_screenresolution_y: (a: number, b: number) => void;
  readonly init_game: (a: number) => number;
  readonly screenresolution_new: (a: number, b: number) => number;
  readonly __wbindgen_malloc: (a: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number) => number;
  readonly __wbindgen_free: (a: number, b: number) => void;
  readonly __wbindgen_exn_store: (a: number) => void;
}

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
*
* @returns {Promise<InitOutput>}
*/
export default function init (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;
