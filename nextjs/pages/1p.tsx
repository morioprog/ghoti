import { GetStaticProps, NextPage } from 'next';
import { useRouter } from 'next/router';
import React from 'react';

import fs from 'fs';
import MaterialTable from 'material-table';

import { useWindowSize } from '@hooks/useWindowSize';
import { JsonData1P } from '@src/types';

import MyHead from '@components/MyHead';
import styles from '@styles/1P.module.css';

type TokoPuyoInfo = {
  pr_number: number;
  ai_name: string;
  date: string;
  average_score: number;
};

type StaticProps = {
  infos: TokoPuyoInfo[];
};

const P1: NextPage<StaticProps> = ({ infos }) => {
  const router = useRouter();
  const { height: _, width } = useWindowSize();

  return (
    <>
      <MyHead title={'Toko-Puyo Kifus'} />
      <div className={styles.container}>
        <h1 className={styles.title}>
          <a href="/">
            <code className={styles.linkname}>/</code>
          </a>
          {' > '}
          <a href="/1p">
            <code className={styles.linkname}>1p</code>
          </a>{' '}
          Toko-Puyo Kifus
        </h1>
        <div className={styles.table}>
          <MaterialTable
            columns={[
              {
                title: '#',
                field: 'pr_number',
                type: 'numeric',
                width: '10%',
                cellStyle: { width: '10%' },
                headerStyle: { width: '10%' },
              },
              {
                title: 'AI Name',
                field: 'ai_name',
                width: '32%',
                cellStyle: { width: '32%' },
                headerStyle: { width: '32%' },
              },
              {
                title: 'Date',
                field: 'date',
                hidden: width <= 600,
                width: '25%',
                cellStyle: { width: '25%' },
                headerStyle: { width: '25%' },
              },
              {
                title: 'Average Score',
                field: 'average_score',
                type: 'numeric',
                width: '33%',
                cellStyle: { width: '33%' },
                headerStyle: { width: '33%' },
                defaultSort: 'desc',
              },
            ]}
            data={infos}
            title="Toko-Puyo Kifus"
            onRowClick={(_event, { pr_number, ai_name }, _togglePanel) => {
              // https://material-table.com/#/docs/features/detail-panel
              // TODO: add some label for clarity
              router.push(`/1p/${pr_number}_${ai_name}`);
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
  const [pr_number, ...rest] = dir.split('_');
  const ai_name = rest.join('_');

  let date = '';
  let average_score = 0;

  const trial_dir = `${path}/${dir}`;
  const jsons = fs
    .readdirSync(trial_dir)
    .filter((f) => fs.statSync(`${trial_dir}/${f}`).isFile());

  for (const json of jsons) {
    const json_path = `${trial_dir}/${json}`;
    const text = fs.readFileSync(json_path, 'utf-8');
    const json_data = JSON.parse(text) as JsonData1P;

    if (date.length === 0) {
      date = json_data.date.slice(0, 10).replace('T', ' ');
    }
    average_score += json_data.score;
  }

  average_score /= jsons.length;

  return {
    pr_number,
    ai_name,
    date,
    average_score,
  };
};

export const getStaticProps: GetStaticProps = async () => {
  const path = process.cwd() + '/../kifus/simulator_1p';
  const dirs = fs
    .readdirSync(path)
    .filter((f) => fs.statSync(`${path}/${f}`).isDirectory())
    .filter((f) => f != 'ga_tuning_1p'); // TODO

  return {
    props: {
      infos: dirs.map((dir) => dir2info(path, dir)),
    },
  };
};

export default P1;
