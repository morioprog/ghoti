import classNames from 'classnames';

import { Color } from '@src/types';

import styles from '@styles/Puyo.module.css';

type Props = {
  color: Color;
  up?: boolean;
  down?: boolean;
  left?: boolean;
  right?: boolean;
  without_background?: boolean;
};

const Puyo = ({ color, up, down, left, right, without_background }: Props) => {
  const background_style = without_background ? styles.without_background : '';

  // Non-color blocks
  if (color == 'Wall')
    return (
      <div className={classNames(styles.puyo, background_style, styles.wall)} />
    );
  if (color == 'Empty')
    return (
      <div
        className={classNames(styles.puyo, background_style, styles.empty)}
      />
    );
  if (color == 'Ojama')
    return (
      <div
        className={classNames(styles.puyo, background_style, styles.ojama)}
      />
    );
  if (color == 'Kata')
    return (
      <div className={classNames(styles.puyo, background_style, styles.kata)} />
    );

  // Vanishing
  if (color.includes('-V')) {
    if (color == 'Red-V') {
      return (
        <div
          className={classNames(
            styles.puyo,
            background_style,
            styles.red_vanishing,
          )}
        ></div>
      );
    } else if (color == 'Green-V') {
      return (
        <div
          className={classNames(
            styles.puyo,
            background_style,
            styles.green_vanishing,
          )}
        ></div>
      );
    } else if (color == 'Blue-V') {
      return (
        <div
          className={classNames(
            styles.puyo,
            background_style,
            styles.blue_vanishing,
          )}
        ></div>
      );
    } else if (color == 'Yellow-V') {
      return (
        <div
          className={classNames(
            styles.puyo,
            background_style,
            styles.yellow_vanishing,
          )}
        ></div>
      );
    } else if (color == 'Purple-V') {
      return (
        <div
          className={classNames(
            styles.puyo,
            background_style,
            styles.purple_vanishing,
          )}
        ></div>
      );
    }
  }

  // Sprite image
  let backgroundPositionY =
    -32 * ((up ? 1 : 0) + (down ? 2 : 0) + (left ? 4 : 0) + (right ? 8 : 0));

  if (color == 'Red') {
    return (
      <div
        className={classNames(styles.puyo, background_style, styles.red)}
        style={{ backgroundPositionY }}
      ></div>
    );
  } else if (color == 'Green') {
    return (
      <div
        className={classNames(styles.puyo, background_style, styles.green)}
        style={{ backgroundPositionY }}
      ></div>
    );
  } else if (color == 'Blue') {
    return (
      <div
        className={classNames(styles.puyo, background_style, styles.blue)}
        style={{ backgroundPositionY }}
      ></div>
    );
  } else if (color == 'Yellow') {
    return (
      <div
        className={classNames(styles.puyo, background_style, styles.yellow)}
        style={{ backgroundPositionY }}
      ></div>
    );
  } else if (color == 'Purple') {
    return (
      <div
        className={classNames(styles.puyo, background_style, styles.purple)}
        style={{ backgroundPositionY }}
      ></div>
    );
  }

  // どれにも当てはまらなかったら背景を返す
  return (
    <div
      className={classNames(styles.puyo, background_style, styles.background)}
    />
  );
};

export default Puyo;
