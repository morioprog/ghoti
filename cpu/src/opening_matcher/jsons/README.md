# `opening_matcher`

## JSON の形式

```json
{
  "visible_tumos": 2,
  "openings": [
    { // n 手目
      {
        "BBAC": { // ツモ
          // 操作前の盤面: 置く場所
          "A/A/B/C///": [4, 1],
          "A/ABA////": [1, 2],
          ...
        },
        ...
      },
      ...
    },
    ...
  ],
}
```
