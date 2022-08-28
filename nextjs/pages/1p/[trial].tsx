import { GetStaticPaths, GetStaticProps, NextPage } from 'next';
import { useRouter } from 'next/router';
import { useEffect, useState } from 'react';

import Button from '@material-ui/core/Button';
import ButtonGroup from '@material-ui/core/ButtonGroup';
import Tooltip from '@material-ui/core/Tooltip';
import classNames from 'classnames';
import fs from 'fs';
import MaterialTable from 'material-table';
import { FaPlay, FaStop } from 'react-icons/fa';
import {
  HiOutlineChevronDoubleLeft,
  HiOutlineChevronDoubleRight,
  HiOutlineChevronLeft,
  HiOutlineChevronRight,
} from 'react-icons/hi';

import { useInterval } from '@hooks/useInterval';
import { useWindowSize } from '@src/hooks/useWindowSize';
import { Board, HEIGHT, JsonData1P, JsonDecision, WIDTH } from '@src/types';
import { dropFloatingPuyo, dropKumiPuyo, vanishPuyo } from '@src/util';

import MyHead from '@components/MyHead';
import PuyoBoard from '@components/PuyoBoard';
import PuyoNexts from '@components/PuyoNexts';
import styles from '@styles/1P_trial.module.css';

type TrialInfo = {
  trial_index: number;
  visible_tumos: number;
  turns: number;
  score: number;
  average_think_ms: string; // TODO: apply format within MaterialTable
};
type StaticProps = { infos: TrialInfo[]; games: JsonData1P[] };
type PathParams = { trial: string };

const TrialDetail: NextPage<StaticProps> = ({ infos, games }) => {
  const router = useRouter();
  const { trial } = router.query;
  const { height: _, width } = useWindowSize();

  // どのtrialを選んでいるか
  const [selectedIndex, setSelectedIndex] = useState(infos[0].trial_index);
  // 何手目を表示しているか
  const [tumoIndex, setTumoIndex] = useState(0);
  // その手の何番目の盤面か
  const [frameIndex, setFrameIndex] = useState(0);
  // 自動でアニメーションを再生するか
  const [autoPlayStatus, { start: startAutoPlay, stop: stopAutoPlay }] =
    useInterval({
      interval: 500,
      autostart: false,
      onUpdate: () => updateFrameIndex(1),
    });

  // 最後まで到達したらアニメーションを止める
  useEffect(() => {
    if (autoPlayStatus === 'STOPPED') {
      return;
    }

    const tumoLength = games[selectedIndex].json_decisions.length - 1;
    const frameLength =
      games[selectedIndex].json_decisions[tumoIndex].boards.length;

    if (tumoIndex === tumoLength - 1 && frameIndex === frameLength - 1) {
      stopAutoPlay();
    }
  }, [selectedIndex, tumoIndex, frameIndex]);

  // `boards` を準備する
  games = games.map((game, j) => {
    let board: Board = Array.from(new Array(HEIGHT), () =>
      new Array(WIDTH).fill('Empty'),
    );
    let heights: number[] = new Array(WIDTH).fill(0);

    // 配列の初期化
    game.json_decisions.map((decision) => {
      decision.boards = [];
      decision.scores = [];
    });

    let nextBoards: Board[] = [JSON.parse(JSON.stringify(board))];
    let nextScores: [number, number][] = [[0, 0]];
    let accumulateScore = 0;

    game.json_decisions = game.json_decisions.map((decision, i) => {
      decision.boards = decision.boards.concat(nextBoards);
      decision.scores = decision.scores.concat(nextScores);

      dropKumiPuyo(
        board,
        game.tumos[i % game.tumos.length],
        decision.decisions[0],
        heights,
      );

      nextBoards = [];
      nextScores = [];

      let chain = 1;

      while (true) {
        const oldBoard = JSON.parse(JSON.stringify(board));
        const scoreDiff = vanishPuyo(board, heights, chain);
        if (scoreDiff === 0) break;

        nextBoards.push(JSON.parse(JSON.stringify(oldBoard)));
        if (chain === 1) {
          nextScores.push(JSON.parse(JSON.stringify([accumulateScore, 0])));
        } else {
          nextScores.push(
            JSON.parse(JSON.stringify(nextScores[nextScores.length - 1])),
          );
        }

        accumulateScore += scoreDiff;
        nextBoards.push(JSON.parse(JSON.stringify(board)));
        nextScores.push(
          JSON.parse(JSON.stringify([accumulateScore, scoreDiff])),
        );

        dropFloatingPuyo(board);
        chain += 1;
      }

      nextBoards.push(JSON.parse(JSON.stringify(board)));
      if (nextScores.length === 0) {
        nextScores.push(JSON.parse(JSON.stringify([accumulateScore, 0])));
      } else {
        nextScores.push(
          JSON.parse(JSON.stringify(nextScores[nextScores.length - 1])),
        );
      }

      return decision;
    });

    // 最後の1手を表示する
    if (game.json_decisions.length <= infos[j].turns + 1) {
      game.json_decisions.push({
        think_ms: -1,
        log_output: '',
        decisions: [{ x: -1, r: -1 }],
        boards: nextBoards,
      } as JsonDecision);
    }

    return game;
  });

  const updateTumoIndex = (margin: number) => {
    const newTumoIndex = tumoIndex + margin;
    const tumoLength = games[selectedIndex].json_decisions.length - 1;

    if (newTumoIndex < 0) {
      setTumoIndex(0);
    } else if (newTumoIndex >= tumoLength) {
      setTumoIndex(tumoLength - 1);
    } else {
      setTumoIndex(newTumoIndex);
    }

    // tumoIndex が変わってたら frameIndex を 0 に
    if (tumoIndex !== newTumoIndex) {
      setFrameIndex(0);
    }
  };

  const updateFrameIndex = (margin: number) => {
    const newFrameIndex = frameIndex + margin;
    const tumoLength = games[selectedIndex].json_decisions.length - 1;
    const frameLength =
      games[selectedIndex].json_decisions[tumoIndex].boards.length;

    if (newFrameIndex < 0) {
      if (frameIndex === 0 && tumoIndex !== 0) {
        updateTumoIndex(-1);
      } else {
        setFrameIndex(0);
      }
    } else if (newFrameIndex >= frameLength) {
      if (frameIndex === frameLength - 1 && tumoIndex !== tumoLength - 1) {
        updateTumoIndex(1);
      } else {
        setFrameIndex(frameLength - 1);
      }
    } else {
      setFrameIndex(newFrameIndex);
    }
  };

  // NOTE: since `trial` is `string | string[]`
  const trial_id = typeof trial === 'string' ? trial : trial.join('');
  const [pr_number, ...rest] = trial_id.split('_');
  const ai_name = rest.join('_');
  const tumo_loop = games[selectedIndex].tumos.length;

  return (
    <>
      <MyHead title={`Toko-Puyo Result of #${pr_number}`} />
      <div className={styles.container}>
        <h1 className={styles.title}>
          <a href="/">
            <code className={styles.linkname}>/</code>
          </a>
          {' > '}
          <a href="/1p">
            <code className={styles.linkname}>1p</code>
          </a>
          {' > '}
          <a href={`https://github.com/morioprog/ghoti/pull/${pr_number}`}>
            <code
              className={styles.linkname}
            >{`#${pr_number}: ${ai_name}`}</code>
          </a>
        </h1>
        <div className={styles.table}>
          <MaterialTable
            columns={[
              {
                title: '#',
                field: 'trial_index',
                type: 'numeric',
                width: '10%',
                cellStyle: { width: '10%' },
                headerStyle: { width: '10%' },
              },
              {
                title: 'Turns',
                field: 'turns',
                type: 'numeric',
                width: '15%',
                cellStyle: { width: '15%' },
                headerStyle: { width: '15%' },
              },
              {
                title: 'Visible',
                field: 'visible_tumos',
                type: 'numeric',
                hidden: width <= 600,
                width: '15%',
                cellStyle: { width: '15%' },
                headerStyle: { width: '15%' },
              },
              {
                title: 'Score',
                field: 'score',
                type: 'numeric',
                width: '30%',
                cellStyle: { width: '30%' },
                headerStyle: { width: '30%' },
              },
              {
                title: 'Average ms',
                field: 'average_think_ms',
                type: 'numeric',
                hidden: width <= 600,
                width: '30%',
                cellStyle: { width: '30%' },
                headerStyle: { width: '30%' },
              },
            ]}
            data={infos}
            title="Toko-Puyo Result"
            onRowClick={(_event, { trial_index }, _togglePanel) => {
              // https://material-table.com/#/docs/features/detail-panel
              setSelectedIndex(trial_index);
              setTumoIndex(0);
              setFrameIndex(0);
            }}
            options={{
              showTitle: false,
              search: false,
              toolbar: false,
              headerStyle: { fontWeight: 'bold' },
              rowStyle: ({ trial_index }) => ({
                backgroundColor:
                  trial_index === selectedIndex ? '#d6dde4' : 'white',
              }),
            }}
          />
        </div>
        <div className={styles.trial_container}>
          <div className={styles.board_container}>
            <div>
              <PuyoBoard
                board={
                  games[selectedIndex].json_decisions[tumoIndex].boards[
                    frameIndex
                  ]
                }
              />
              {/* TODO: キーボードで操作できるように */}
              <ButtonGroup className={styles.button_group}>
                <Tooltip title="Previous Turn">
                  <Button
                    variant="outlined"
                    className={styles.button}
                    onClick={() => updateTumoIndex(-1)}
                  >
                    <HiOutlineChevronDoubleLeft />
                  </Button>
                </Tooltip>
                <Tooltip title="Previous Frame">
                  <Button
                    variant="outlined"
                    className={styles.button}
                    onClick={() => updateFrameIndex(-1)}
                  >
                    <HiOutlineChevronLeft />
                  </Button>
                </Tooltip>
                <Tooltip
                  title={
                    autoPlayStatus === 'RUNNING'
                      ? 'Stop Auto Play'
                      : 'Auto Play (500ms/frame)'
                  }
                >
                  <Button
                    variant="outlined"
                    className={styles.button}
                    onClick={() =>
                      autoPlayStatus === 'RUNNING'
                        ? stopAutoPlay()
                        : startAutoPlay()
                    }
                  >
                    {autoPlayStatus === 'RUNNING' ? <FaStop /> : <FaPlay />}
                  </Button>
                </Tooltip>
                <Tooltip title="Next Frame">
                  <Button
                    variant="outlined"
                    className={styles.button}
                    onClick={() => updateFrameIndex(1)}
                  >
                    <HiOutlineChevronRight />
                  </Button>
                </Tooltip>
                <Tooltip title="Next Turn">
                  <Button
                    variant="outlined"
                    className={styles.button}
                    onClick={() => updateTumoIndex(1)}
                  >
                    <HiOutlineChevronDoubleRight />
                  </Button>
                </Tooltip>
              </ButtonGroup>
            </div>
            <PuyoNexts
              current={games[selectedIndex].tumos[tumoIndex % tumo_loop]}
              next={games[selectedIndex].tumos[(tumoIndex + 1) % tumo_loop]}
              next2={games[selectedIndex].tumos[(tumoIndex + 2) % tumo_loop]}
            />
          </div>
          {/* TODO: クリックしてフォームで値を変更可能に（普段は普通のラベル） */}
          <div className={styles.info_container}>
            <span className={classNames(styles.info_value, styles.tumo_index)}>
              {tumoIndex}
            </span>
            <span className={styles.info_label}>手目</span>
            <span className={classNames(styles.info_value, styles.frame_index)}>
              {frameIndex + 1}
            </span>
            <span className={styles.info_label}>フレーム目</span>
            <br />
            <span className={styles.info_label}>思考時間</span>
            <span className={classNames(styles.info_value, styles.think_ms)}>
              {tumoIndex === 0
                ? -1
                : games[selectedIndex].json_decisions[tumoIndex - 1].think_ms}
            </span>
            <span className={styles.info_label}>ミリ秒</span>
            <br />
            <span className={classNames(styles.info_value, styles.score_value)}>
              {`${games[selectedIndex].json_decisions[tumoIndex].scores[frameIndex][0]}`}
            </span>
            <span className={styles.info_label}>点</span>
            <span className={styles.score_diff}>
              {`(+${games[selectedIndex].json_decisions[tumoIndex].scores[frameIndex][1]})`}
            </span>
            <br />
            {/* TODO: think of better ways to show chain */}
            <div className={styles.log_output}>
              {`${
                frameIndex === 0 ? '\n' : ((frameIndex + 1) >> 1) + '連鎖!\n'
              }` +
                (tumoIndex === 0
                  ? ''
                  : games[selectedIndex].json_decisions[tumoIndex - 1]
                      .log_output)}
            </div>
            <br />
            <span className={styles.asset_info}>
              <a href={'https://puyo-camp.jp/posts/157768'}>
                薄力粉様のぷよ素材
              </a>
              を使わせて頂いております。
            </span>
            <br />
            <a
              href={games[selectedIndex].url}
              className={styles.simulator_link}
            >
              「ぷよぷよ連鎖シミュレータ」様へのリンク
            </a>
          </div>
        </div>
      </div>
      <footer className={styles.footer} />
    </>
  );
};

export const getStaticPaths: GetStaticPaths<PathParams> = async () => {
  const path = process.cwd() + '/../kifus/simulator_1p';
  const dirs = fs
    .readdirSync(path)
    .filter((f) => fs.statSync(`${path}/${f}`).isDirectory())
    .filter((f) => f != 'ga_tuning_1p'); // TODO

  return {
    paths: dirs.map((dir) => ({ params: { trial: dir } })),
    fallback: false,
  };
};

export const getStaticProps: GetStaticProps = async (context) => {
  const { trial } = context.params as PathParams;

  const trial_dir = `${process.cwd()}/../kifus/simulator_1p/${trial}`;
  const jsons = fs
    .readdirSync(trial_dir)
    .filter((f) => fs.statSync(`${trial_dir}/${f}`).isFile());

  const games = jsons.map((json) => {
    const json_path = `${trial_dir}/${json}`;
    const text = fs.readFileSync(json_path, 'utf-8');
    const json_data = JSON.parse(text) as JsonData1P;
    return json_data;
  });

  return {
    props: {
      infos: games.map(
        (game, index) =>
          ({
            trial_index: index,
            visible_tumos: game.visible_tumos,
            turns: game.json_decisions.length,
            score: game.score,
            average_think_ms:
              game.json_decisions.length === 0
                ? -1
                : (
                    game.json_decisions
                      .map((d) => d.think_ms)
                      .reduce((acc, cur) => acc + cur, 0) /
                    game.json_decisions.length
                  ).toFixed(2),
          } as TrialInfo),
      ),
      games,
    },
  };
};

export default TrialDetail;
