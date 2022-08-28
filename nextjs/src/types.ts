export type Color =
  // Normal
  | 'Red'
  | 'Green'
  | 'Blue'
  | 'Yellow'
  | 'Purple'
  // Vanishing
  | 'Red-V'
  | 'Green-V'
  | 'Blue-V'
  | 'Yellow-V'
  | 'Purple-V'
  // Other
  | 'Wall'
  | 'Empty'
  | 'Ojama'
  | 'Kata';

export type ColorChar = 'R' | 'G' | 'B' | 'Y' | 'P';
export type KumiPuyo = `${ColorChar}${ColorChar}`;

export const HEIGHT = 13;
export const WIDTH = 6;
export type Board = Color[][];

// for `simulator_1p`
export type Decision = {
  x: number;
  r: number;
};
export type JsonDecision = {
  think_ms: number;
  log_output: string;
  decisions: Decision[];
  boards?: Board[];
  scores?: [number, number][]; // [累積点数, 加算された点数]
};
export type JsonData1P = {
  date: string;
  score: number;
  visible_tumos: number;
  tumos: KumiPuyo[];
  json_decisions: JsonDecision[];
  url: string;
};

// for `simulator_2p`
export type JsonState = {
  tumo_index: number;
  field: string; // pfen-like の形式で表現された盤面
  score: number;
  ojama_fixed: number;
  ojama_ongoing: number;
  current_chain: number;
  board?: Board; // `field` を変換した結果
};
export type JsonEvent = {
  frame: number;
  json_state_1p: JsonState;
  json_state_2p: JsonState;
};
export type JsonMatch = {
  won_1p: boolean;
  tumos: KumiPuyo[];
  json_events: JsonEvent[];
};
export type JsonData2P = {
  date: string;
  win_count_1p: number;
  win_count_2p: number;
  visible_tumos: number;
  json_matches: JsonMatch[];
};
