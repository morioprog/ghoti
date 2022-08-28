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
import { HiOutlineChevronLeft, HiOutlineChevronRight } from 'react-icons/hi';

import { useInterval } from '@hooks/useInterval';
import { useWindowSize } from '@src/hooks/useWindowSize';
import { JsonData2P, JsonEvent } from '@src/types';
import { pfenToBoard } from '@src/util';

import MyHead from '@components/MyHead';
import PuyoBoard from '@components/PuyoBoard';
import PuyoNexts from '@components/PuyoNexts';
import PuyoOjama from '@components/PuyoOjama';
// TODO: 別のCSSを用意する？
import styles_1p from '@styles/1P_trial.module.css';
import styles_2p from '@styles/2P_match.module.css';

type MatchInfo = {
  winner: string;
  match_index: number;
  frames: number;
  score_1p: number;
  score_2p: number;
};
type StaticProps = { infos: MatchInfo[]; json_data: JsonData2P };
type PathParams = { match: string };

const MatchDetail: NextPage<StaticProps> = ({ infos, json_data }) => {
  const router = useRouter();
  const { match } = router.query;
  const { height: _, width } = useWindowSize();

  // どのmatchを選んでいるか
  const [selectedIndex, setSelectedIndex] = useState(infos[0].match_index);
  // 何手目を表示しているか
  const [eventIndex, setEventIndex] = useState(0);
  // 自動でアニメーションを再生するか
  const [autoPlayStatus, { start: startAutoPlay, stop: stopAutoPlay }] =
    useInterval({
      interval: 500,
      autostart: false,
      onUpdate: () => updateEventIndex(1),
    });

  // 最後まで到達したらアニメーションを止める
  useEffect(() => {
    if (autoPlayStatus === 'STOPPED') {
      return;
    }

    const eventLength =
      json_data.json_matches[selectedIndex].json_events.length;

    if (eventIndex === eventLength - 1) {
      stopAutoPlay();
    }
  }, [selectedIndex, eventIndex]);

  json_data.json_matches = json_data.json_matches.map((json_match) => {
    // pfen文字列を盤面に変換する
    json_match.json_events = json_match.json_events.map((json_event) => {
      json_event.json_state_1p.board = pfenToBoard(
        json_event.json_state_1p.field,
      );
      json_event.json_state_2p.board = pfenToBoard(
        json_event.json_state_2p.field,
      );
      return json_event;
    });

    // フレーム or 盤面の状況が同じなら後の方のものを採用
    // TODO: そもそも JSON を吐き出すときにそういうケースを省く
    json_match.json_events = json_match.json_events.filter((json_event, i) => {
      // 最後のイベントは必ず使う
      if (i === json_match.json_events.length - 1) {
        return true;
      }

      const next_event = json_match.json_events[i + 1];
      if (
        json_event.frame === next_event.frame ||
        // `JsonEvent` の等価判定（なにこれ...）
        (JSON.stringify(Object.entries(json_event.json_state_1p).sort()) ===
          JSON.stringify(Object.entries(next_event.json_state_1p).sort()) &&
          JSON.stringify(Object.entries(json_event.json_state_2p).sort()) ===
            JSON.stringify(Object.entries(next_event.json_state_2p).sort()))
      ) {
        return false;
      }

      return true;
    });

    return json_match;
  });

  const updateEventIndex = (margin: number) => {
    const newEventIndex = eventIndex + margin;
    const eventLength =
      json_data.json_matches[selectedIndex].json_events.length;
    if (newEventIndex < 0) {
      setEventIndex(0);
    } else if (newEventIndex >= eventLength) {
      setEventIndex(eventLength - 1);
    } else {
      setEventIndex(newEventIndex);
    }
  };

  // NOTE: since `match` is `string | string[]`
  const match_id = typeof match === 'string' ? match : match.join('');
  const [pr_number, ai_name_1p, _vs, ai_name_2p] = match_id.split('_');
  const tumo_loop = json_data.json_matches[selectedIndex].tumos.length;

  return (
    <>
      <MyHead title={`Battle Result of #${pr_number}`} />
      <div className={styles_1p.container}>
        <h1 className={styles_1p.title}>
          <a href="/">
            <code className={styles_1p.linkname}>/</code>
          </a>
          {' > '}
          <a href="/2p">
            <code className={styles_1p.linkname}>2p</code>
          </a>
          {' > '}
          <a href={`https://github.com/morioprog/ghoti/pull/${pr_number}`}>
            <code
              className={styles_1p.linkname}
            >{`#${pr_number}: ${ai_name_1p} vs ${ai_name_2p}`}</code>
          </a>
        </h1>
        <div className={styles_1p.table}>
          <MaterialTable
            columns={[
              {
                title: '#',
                field: 'match_index',
                type: 'numeric',
                width: '10%',
                cellStyle: { width: '10%' },
                headerStyle: { width: '10%' },
              },
              {
                title: 'Winner',
                field: 'winner',
                type: 'numeric',
                width: '10%',
                cellStyle: { width: '10%' },
                headerStyle: { width: '10%' },
              },
              {
                title: 'Frames',
                field: 'frames',
                type: 'numeric',
                hidden: width <= 600,
                width: '15%',
                cellStyle: { width: '15%' },
                headerStyle: { width: '15%' },
              },
              {
                title: '1P Score',
                field: 'score_1p',
                type: 'numeric',
                width: '30%',
                cellStyle: { width: '30%' },
                headerStyle: { width: '30%' },
              },
              {
                title: '2P Score',
                field: 'score_2p',
                type: 'numeric',
                width: '30%',
                cellStyle: { width: '30%' },
                headerStyle: { width: '30%' },
              },
            ]}
            data={infos}
            title="Battle Result"
            onRowClick={(_event, { match_index }, _togglePanel) => {
              // https://material-table.com/#/docs/features/detail-panel
              setSelectedIndex(match_index);
              setEventIndex(0);
            }}
            options={{
              showTitle: false,
              search: false,
              toolbar: false,
              headerStyle: { fontWeight: 'bold' },
              rowStyle: ({ match_index }) => ({
                backgroundColor:
                  match_index === selectedIndex ? '#d6dde4' : 'white',
              }),
            }}
          />
        </div>
        <div className={styles_2p.match_container}>
          <div className={styles_1p.board_container}>
            <div>
              <PuyoOjama
                ojama={
                  json_data.json_matches[selectedIndex].json_events[eventIndex]
                    .json_state_1p.ojama_fixed +
                  json_data.json_matches[selectedIndex].json_events[eventIndex]
                    .json_state_1p.ojama_ongoing
                }
              />
              <PuyoBoard
                board={
                  json_data.json_matches[selectedIndex].json_events[eventIndex]
                    .json_state_1p.board
                }
              />
              <span className={styles_2p.score_label}>
                {`${(
                  '00000000' +
                  json_data.json_matches[selectedIndex].json_events[eventIndex]
                    .json_state_1p.score
                ).slice(-8)}`}
              </span>
            </div>
            <PuyoNexts
              current={
                json_data.json_matches[selectedIndex].tumos[
                  json_data.json_matches[selectedIndex].json_events[eventIndex]
                    .json_state_1p.tumo_index % tumo_loop
                ]
              }
              next={
                json_data.json_matches[selectedIndex].tumos[
                  (json_data.json_matches[selectedIndex].json_events[eventIndex]
                    .json_state_1p.tumo_index +
                    1) %
                    tumo_loop
                ]
              }
              next2={
                json_data.json_matches[selectedIndex].tumos[
                  (json_data.json_matches[selectedIndex].json_events[eventIndex]
                    .json_state_1p.tumo_index +
                    2) %
                    tumo_loop
                ]
              }
              margin={1}
            />
          </div>
          <div className={styles_1p.board_container}>
            <PuyoNexts
              current={
                json_data.json_matches[selectedIndex].tumos[
                  json_data.json_matches[selectedIndex].json_events[eventIndex]
                    .json_state_2p.tumo_index % tumo_loop
                ]
              }
              next={
                json_data.json_matches[selectedIndex].tumos[
                  (json_data.json_matches[selectedIndex].json_events[eventIndex]
                    .json_state_2p.tumo_index +
                    1) %
                    tumo_loop
                ]
              }
              next2={
                json_data.json_matches[selectedIndex].tumos[
                  (json_data.json_matches[selectedIndex].json_events[eventIndex]
                    .json_state_2p.tumo_index +
                    2) %
                    tumo_loop
                ]
              }
              margin={1}
            />
            <div>
              <PuyoOjama
                ojama={
                  json_data.json_matches[selectedIndex].json_events[eventIndex]
                    .json_state_2p.ojama_fixed +
                  json_data.json_matches[selectedIndex].json_events[eventIndex]
                    .json_state_2p.ojama_ongoing
                }
              />
              <PuyoBoard
                board={
                  json_data.json_matches[selectedIndex].json_events[eventIndex]
                    .json_state_2p.board
                }
              />
              <span className={styles_2p.score_label}>
                {`${(
                  '00000000' +
                  json_data.json_matches[selectedIndex].json_events[eventIndex]
                    .json_state_2p.score
                ).slice(-8)}`}
              </span>
            </div>
          </div>
        </div>
        <div className={styles_1p.info_container}>
          <span
            className={classNames(styles_1p.info_value, styles_1p.think_ms)}
          >
            {`${json_data.json_matches[selectedIndex].json_events[eventIndex].frame}`}
          </span>
          <span className={styles_1p.info_label}> F</span>
          <br />
          {/* TODO: キーボードで操作できるように */}
          <ButtonGroup className={styles_2p.button_group}>
            <Tooltip title="Previous Event">
              <Button
                variant="outlined"
                className={styles_1p.button}
                onClick={() => updateEventIndex(-1)}
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
                className={styles_1p.button}
                onClick={() =>
                  autoPlayStatus === 'RUNNING'
                    ? stopAutoPlay()
                    : startAutoPlay()
                }
              >
                {autoPlayStatus === 'RUNNING' ? <FaStop /> : <FaPlay />}
              </Button>
            </Tooltip>
            <Tooltip title="Next Event">
              <Button
                variant="outlined"
                className={styles_1p.button}
                onClick={() => updateEventIndex(1)}
              >
                <HiOutlineChevronRight />
              </Button>
            </Tooltip>
          </ButtonGroup>
          <br />
          <br />
          <span className={styles_1p.asset_info}>
            <a href={'https://puyo-camp.jp/posts/157768'}>薄力粉様のぷよ素材</a>
            を使わせて頂いております。
          </span>
        </div>
      </div>
      <footer className={styles_1p.footer} />
    </>
  );
};

export const getStaticPaths: GetStaticPaths<PathParams> = async () => {
  const path = process.cwd() + '/../kifus/simulator_2p';
  const dirs = fs
    .readdirSync(path)
    .filter((f) => fs.statSync(`${path}/${f}`).isDirectory())
    .filter((f) => f != 'ga_tuning_2p'); // TODO

  return {
    paths: dirs.map((dir) => ({ params: { match: dir } })),
    fallback: false,
  };
};

export const getStaticProps: GetStaticProps = async (context) => {
  const { match } = context.params as PathParams;

  const trial_dir = `${process.cwd()}/../kifus/simulator_2p/${match}`;
  const jsons = fs
    .readdirSync(trial_dir)
    .filter((f) => fs.statSync(`${trial_dir}/${f}`).isFile());

  // NOTE: フォルダの中に複数ファイルあっても、その先頭のものしか使わないようになってる
  const json_data = jsons.map((json) => {
    const json_path = `${trial_dir}/${json}`;
    const text = fs.readFileSync(json_path, 'utf-8');
    const json_data = JSON.parse(text) as JsonData2P;
    return json_data;
  })[0];

  // NOTE: 最大5試合に制限している（現状多すぎると重いので）
  if (json_data.json_matches.length > 5) {
    json_data.json_matches = json_data.json_matches.slice(0, 5);
  }

  return {
    props: {
      infos: json_data.json_matches.map((game, index) => {
        let last_json_event = game.json_events.slice(-1)[0];
        return {
          winner: game.won_1p ? '1P' : '2P',
          match_index: index,
          frames: last_json_event.frame,
          score_1p: last_json_event.json_state_1p.score,
          score_2p: last_json_event.json_state_2p.score,
        } as MatchInfo;
      }),
      json_data,
    },
  };
};

export default MatchDetail;
