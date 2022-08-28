import { NextPage } from 'next';
import React from 'react';

import { Board, WIDTH } from '@src/types';

import Puyo from '@components/Puyo';
import styles from '@styles/PuyoBoard.module.css';

type Props = {
  board: Board;
};

const PuyoBoard: NextPage<Props> = ({ board }) => {
  return (
    <div className={styles.puyoboard}>
      {board.map((row, i) => (
        <React.Fragment key={`fragment-${i}`}>
          <span
            style={{
              filter: i == 0 ? 'brightness(60%)' : '',
            }}
          >
            <Puyo color={'Wall'} />
            {row.map((color, j) => (
              <Puyo
                color={color}
                up={i > 1 && board[i - 1][j] == color}
                down={i > 0 && i < board.length - 1 && board[i + 1][j] == color}
                left={j > 0 && board[i][j - 1] == color}
                right={j < board[0].length - 1 && board[i][j + 1] == color}
                key={`board-${i}-${j}`}
              />
            ))}
            <Puyo color={'Wall'} />
          </span>
          <br />
        </React.Fragment>
      ))}
      {new Array(WIDTH + 2).fill(0).map((_, i) => (
        <Puyo color={'Wall'} key={`wall-${i}`} />
      ))}
    </div>
  );
};

export default PuyoBoard;
