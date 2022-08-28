import {
  Board,
  Color,
  ColorChar,
  Decision,
  HEIGHT,
  KumiPuyo,
  WIDTH,
} from '@src/types';
import { UnionFind } from '@src/unionfind';

const colorCharToColor = (char: ColorChar | 'O') => {
  if (char === 'R') return 'Red';
  if (char === 'G') return 'Green';
  if (char === 'B') return 'Blue';
  if (char === 'Y') return 'Yellow';
  if (char === 'P') return 'Purple';
  return 'Ojama';
};

export const kumiPuyoToColorArray = (kumiPuyo: KumiPuyo) => {
  return [
    colorCharToColor(kumiPuyo[0] as ColorChar),
    colorCharToColor(kumiPuyo[1] as ColorChar),
  ] as Color[];
};

export const isSameColor = (kumiPuyo: KumiPuyo) => {
  return kumiPuyo[0] === kumiPuyo[1];
};

const swapKumiPuyo = (kumiPuyo: KumiPuyo) => {
  return (kumiPuyo[1] + kumiPuyo[0]) as KumiPuyo;
};

const dropSinglePuyo = (
  board: Board,
  puyo: Color,
  column: number,
  heights: number[],
) => {
  const height = heights[column];
  if (height < board.length) {
    board[board.length - 1 - height][column] = puyo;
    heights[column] += 1;
  }
};

export const dropKumiPuyo = (
  board: Board,
  kumiPuyo: KumiPuyo,
  decision: Decision,
  heights: number[],
) => {
  kumiPuyo = decision.r === 2 ? swapKumiPuyo(kumiPuyo) : kumiPuyo;

  // NOTE: decision.x is [1, 6] not [0, 5]
  dropSinglePuyo(
    board,
    colorCharToColor(kumiPuyo[0] as ColorChar),
    decision.x - 1,
    heights,
  );
  dropSinglePuyo(
    board,
    colorCharToColor(kumiPuyo[1] as ColorChar),
    decision.r === 1
      ? decision.x
      : decision.r === 3
      ? decision.x - 2
      : decision.x - 1,
    heights,
  );
};

const isNormalColor = (color: Color) =>
  color == 'Red' ||
  color == 'Green' ||
  color == 'Blue' ||
  color == 'Yellow' ||
  color == 'Purple';
const isOjama = (color: Color) => color == 'Ojama' || color == 'Kata';

/* 各種ボーナス（点数計算用） */
// 連鎖ボーナス
const CHAIN_BONUS = [
  0, 0, 8, 16, 32, 64, 96, 128, 160, 192, 224, 256, 288, 320, 352, 384, 416,
  448, 480, 512,
];
// 色数ボーナス
const COLOR_BONUS = [0, 0, 3, 6, 12, 24];
// 連結ボーナス
const LONG_BONUS = [0, 0, 0, 0, 0, 2, 3, 4, 5, 6, 7, 10];

// 消えるぷよを "*-V" にして点数を返す（正ならぷよが消えている）
export const vanishPuyo = (
  board: Board,
  heights: number[],
  current_chain: number,
) => {
  const H = board.length;
  const W = board[0].length;
  const V = H * W;
  const uf = new UnionFind(V);

  // 右と下だけ
  const listAdjacentPuyo = (h: number, w: number) => {
    let ret = [];
    if (h + 1 < H) ret.push([h + 1, w]);
    if (w + 1 < W) ret.push([h, w + 1]);
    return ret;
  };
  const pair2index = (h: number, w: number) => h * W + w;

  // 一番上の列 (h === 0) は幽霊なので無視する
  for (let h = 1; h < H; ++h) {
    for (let w = 0; w < W; ++w) {
      if (!isNormalColor(board[h][w])) continue;
      const idx = pair2index(h, w);
      for (const [nh, nw] of listAdjacentPuyo(h, w)) {
        if (board[h][w] === board[nh][nw]) {
          const nidx = pair2index(nh, nw);
          uf.unite(idx, nidx);
        }
      }
    }
  }

  // TODO: 現状おじゃまは消えないので注意
  let vanished_puyo = 0;
  let long_bonus = 0;
  let colors = new Set<Color>();
  for (let h = 1; h < H; ++h) {
    for (let w = 0; w < W; ++w) {
      const idx = pair2index(h, w);
      const connectivity = uf.size(idx);
      if (isNormalColor(board[h][w]) && connectivity >= 4) {
        board[h][w] += '-V';
        --heights[w];
        if (uf.find(idx) == idx) {
          vanished_puyo += connectivity;
          long_bonus += LONG_BONUS[connectivity >= 11 ? 11 : connectivity];
          colors.add(board[h][w]);
        }
      }
    }
  }

  // 何も消えなかった
  if (vanished_puyo === 0) {
    return 0;
  }

  const color_bonus = COLOR_BONUS[colors.size];
  const chain_bonus = CHAIN_BONUS[current_chain];
  const bonus_sum = long_bonus + color_bonus + chain_bonus;
  const score = vanished_puyo * (bonus_sum === 0 ? 1 : bonus_sum) * 10;
  return score;
};

// "*-V" と "Empty" があればそれを空白とみなし、浮いているぷよを落とす
export const dropFloatingPuyo = (board: Board) => {
  const H = board.length;
  const W = board[0].length;

  for (let w = 0; w < W; ++w) {
    let column = new Array(H).fill('Empty');
    let idx = 0;
    for (let h = H - 1; h >= 0; --h) {
      if (isNormalColor(board[h][w]) || isOjama(board[h][w])) {
        column[idx] = board[h][w];
        ++idx;
      }
    }
    for (let i = 0; i < H; ++i) {
      board[H - 1 - i][w] = column[i];
    }
  }
};

// pfen文字列を盤面に変換
export const pfenToBoard = (pfen: string) => {
  let board: Board = Array.from(new Array(HEIGHT), () =>
    new Array(WIDTH).fill('Empty'),
  );
  const pfen_columns = pfen.split('/');
  for (let x = 0; x < WIDTH; ++x) {
    for (let y = 0; y < pfen_columns[x].length; ++y) {
      board[HEIGHT - 1 - y][x] = colorCharToColor(
        pfen_columns[x][y].toUpperCase() as ColorChar,
      );
    }
  }

  return board;
};
