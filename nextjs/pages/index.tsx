import { FaTwitter } from 'react-icons/fa';

import MyHead from '@components/MyHead';
import styles from '@styles/Home.module.css';

const Home = () => (
  <>
    <MyHead />
    <div className={styles.container}>
      <main className={styles.main}>
        <h1 className={styles.title}>
          Welcome to <a href="https://ja.wikipedia.org/wiki/Ghoti">ghoti!</a>🐟
        </h1>

        <p className={styles.description}>
          Puyo AI developed by{' '}
          <a href="https://twitter.com/morio_puyo">
            <FaTwitter
              style={{ verticalAlign: 'text-bottom' }}
              color={'#2498d8'}
            />
            morio_puyo
          </a>
        </p>

        <div className={styles.grid}>
          <a href="1p" className={styles.card}>
            <h3>1P &rarr;</h3>
            <p>Kifus of Toko-Puyo</p>
          </a>

          <a href="2p" className={styles.card}>
            <h3>2P &rarr;</h3>
            <p>Kifus of two-player battle</p>
          </a>
        </div>
      </main>
    </div>
  </>
);

export default Home;
