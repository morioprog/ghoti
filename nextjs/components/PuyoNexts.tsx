import { NextPage } from 'next';
import React from 'react';

import { Color, KumiPuyo } from '@src/types';
import { isSameColor, kumiPuyoToColorArray } from '@src/util';

import Puyo from '@components/Puyo';
import styles from '@styles/PuyoBoard.module.css';

type Props = {
  current?: KumiPuyo;
  next?: KumiPuyo;
  next2?: KumiPuyo;
  margin?: number; // 下方向に何マスずらすか
};

const PuyoNexts: NextPage<Props> = ({ current, next, next2, margin }) => {
  margin = margin ? margin : 0;

  const puyos: Color[] = new Array(9 + margin).fill('Empty');
  const same: boolean[] = new Array(9 + margin).fill(false);
  if (current) {
    puyos[1 + margin] = kumiPuyoToColorArray(current)[1];
    puyos[2 + margin] = kumiPuyoToColorArray(current)[0];
    same[1 + margin] = isSameColor(current);
    same[2 + margin] = isSameColor(current);
  }
  if (next) {
    puyos[4 + margin] = kumiPuyoToColorArray(next)[1];
    puyos[5 + margin] = kumiPuyoToColorArray(next)[0];
    same[4 + margin] = isSameColor(next);
    same[5 + margin] = isSameColor(next);
  }
  if (next2) {
    puyos[7 + margin] = kumiPuyoToColorArray(next2)[1];
    puyos[8 + margin] = kumiPuyoToColorArray(next2)[0];
    same[7 + margin] = isSameColor(next2);
    same[8 + margin] = isSameColor(next2);
  }

  return (
    <div className={styles.puyoboard}>
      {puyos.map((puyo, i) => (
        <React.Fragment key={`fragment-${i}`}>
          <Puyo
            color={puyo}
            down={i % 3 == (1 + margin) % 3 && same[i]}
            up={i % 3 == (2 + margin) % 3 && same[i]}
            without_background
          />
          <br />
        </React.Fragment>
      ))}
    </div>
  );
};

export default PuyoNexts;
