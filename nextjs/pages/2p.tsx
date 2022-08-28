import { GetStaticProps, NextPage } from 'next';
import { useRouter } from 'next/router';
import React from 'react';

import fs from 'fs';
import MaterialTable from 'material-table';

import { useWindowSize } from '@hooks/useWindowSize';
import { JsonData2P } from '@src/types';

import MyHead from '@components/MyHead';
// TODO: 別のCSSを用意する？（適用したいスタイルは同じなのでこのままでもよいが...）
import styles from '@styles/1P.module.css';

type BattleInfo = {
  pr_number: number;
  ai_name_1p: string;
  ai_name_2p: string;
  win_count_1p: number;
  win_count_2p: number;
  date: string;
};

type StaticProps = {
  infos: BattleInfo[];
};

const P2: NextPage<StaticProps> = ({ infos }) => {
  const router = useRouter();
  const { height: _, width } = useWindowSize();

  return (
    <>
      <MyHead title={'Battle Kifus'} />
      <div className={styles.container}>
        <h1 className={styles.title}>
          <a href="/">
            <code className={styles.linkname}>/</code>
          </a>
          {' > '}
          <a href="/2p">
            <code className={styles.linkname}>2p</code>
          </a>{' '}
          Battle Kifus
        </h1>
        <div className={styles.table}>
          <MaterialTable
            columns={[
              {
                title: '#',
                field: 'pr_number',
                type: 'numeric',
                defaultSort: 'desc',
                width: '10%',
                cellStyle: { width: '10%' },
                headerStyle: { width: '10%' },
              },
              {
                title: 'AI (1P)',
                field: 'ai_name_1p',
                width: '20%',
                cellStyle: { width: '20%' },
                headerStyle: { width: '20%' },
              },
              {
                title: 'AI (2P)',
                field: 'ai_name_2p',
                width: '20%',
                cellStyle: { width: '20%' },
                headerStyle: { width: '20%' },
              },
              {
                title: '1P',
                field: 'win_count_1p',
                width: '15%',
                cellStyle: { width: '15%' },
                headerStyle: { width: '15%' },
              },
              {
                title: '2P',
                field: 'win_count_2p',
                width: '15%',
                cellStyle: { width: '15%' },
                headerStyle: { width: '15%' },
              },
              {
                title: 'Date',
                field: 'date',
                hidden: width <= 800,
                width: '20%',
                cellStyle: { width: '20%' },
                headerStyle: { width: '20%' },
              },
            ]}
            data={infos}
            title="Battle Kifus"
            onRowClick={(
              _event,
              { pr_number, ai_name_1p, ai_name_2p },
              _togglePanel,
            ) => {
              // https://material-table.com/#/docs/features/detail-panel
              // TODO: add some labels for clarification
              router.push(`/2p/${pr_number}_${ai_name_1p}_vs_${ai_name_2p}`);
            }}
            options={{
              headerStyle: { fontWeight: 'bold' },
              pageSize: 10,
            }}
          />
        </div>
      </div>
    </>
  );
};

const dir2info = (path: string, dir: string) => {
  // NOTE: AIの名前に `_` が含まれないことを仮定
  const [pr_number, ai_name_1p, _, ai_name_2p] = dir.split('_');

  let date = '';
  let win_count_1p = 0;
  let win_count_2p = 0;

  const trial_dir = `${path}/${dir}`;
  const jsons = fs
    .readdirSync(trial_dir)
    .filter((f) => fs.statSync(`${trial_dir}/${f}`).isFile());

  for (const json of jsons) {
    const json_path = `${trial_dir}/${json}`;
    const text = fs.readFileSync(json_path, 'utf-8');
    const json_data = JSON.parse(text) as JsonData2P;

    if (date.length === 0) {
      date = json_data.date.slice(0, 10).replace('T', ' ');
    }
    win_count_1p += json_data.win_count_1p;
    win_count_2p += json_data.win_count_2p;
  }

  return {
    pr_number,
    ai_name_1p,
    ai_name_2p,
    win_count_1p,
    win_count_2p,
    date,
  };
};

export const getStaticProps: GetStaticProps = async () => {
  const path = process.cwd() + '/../kifus/simulator_2p';
  const dirs = fs
    .readdirSync(path)
    .filter((f) => fs.statSync(`${path}/${f}`).isDirectory())
    .filter((f) => f != 'ga_tuning_2p'); // TODO

  return {
    props: {
      infos: dirs.map((dir) => dir2info(path, dir)),
    },
  };
};

export default P2;
