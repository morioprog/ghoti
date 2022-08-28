import { useEffect, useReducer, useRef } from 'react';

export type Action = { type: 'start' | 'stop'; payload?: unknown };

type Control = {
  start: () => void;
  stop: () => void;
};

type State = 'RUNNING' | 'STOPPED';

type OnUpdate = () => void;

type Options = {
  interval: number;
  autostart: boolean;
  onUpdate?: OnUpdate;
};

const reducer = (state: State, action: Action): State => {
  switch (action.type) {
    case 'start':
      return 'RUNNING';
    case 'stop':
      return 'STOPPED';
    default:
      return state;
  }
};

export const useInterval = ({
  interval = 1000,
  autostart = false,
  onUpdate,
}: Partial<Options>): [State, Control] => {
  const onUpdateRef = useRef<OnUpdate>(() => {});
  const [state, dispatch] = useReducer(reducer, 'STOPPED');

  const start = () => {
    dispatch({ type: 'start' });
  };
  const stop = () => {
    dispatch({ type: 'stop' });
  };

  useEffect(() => {
    onUpdateRef.current = onUpdate ?? (() => {});
  }, [onUpdate]);

  useEffect(() => {
    if (autostart) {
      dispatch({ type: 'start' });
    }
  }, [autostart]);

  useEffect(() => {
    let timerId: NodeJS.Timeout | undefined = undefined;
    if (state === 'RUNNING') {
      timerId = setInterval(() => {
        onUpdateRef.current();
      }, interval);
    } else {
      timerId && clearInterval(timerId);
    }
    return () => {
      timerId && clearInterval(timerId);
    };
  }, [interval, state]);

  return [state, { start, stop }];
};

export default useInterval;
