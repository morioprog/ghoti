import { NextPage } from 'next';

import styles from '@styles/PuyoOjama.module.css';

type Props = {
  ojama: number;
};

const PuyoOjama: NextPage<Props> = ({ ojama }) => {
  let ojama_str = '';
  while (ojama > 0 && ojama_str.length < 6) {
    if (ojama >= 720) {
      ojama_str += '👑';
      ojama -= 720;
    } else if (ojama >= 360) {
      ojama_str += '🌙';
      ojama -= 360;
    } else if (ojama >= 180) {
      ojama_str += '⭐';
      ojama -= 180;
    } else if (ojama >= 30) {
      ojama_str += '🔴';
      ojama -= 30;
    } else if (ojama >= 6) {
      ojama_str += '⚪';
      ojama -= 6;
    } else {
      ojama_str += '◽';
      ojama -= 1;
    }
  }

  return <div className={styles.puyoojama}>{`${ojama_str}`}</div>;
};

export default PuyoOjama;
