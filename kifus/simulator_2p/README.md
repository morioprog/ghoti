# `simulator_2p`

## JSON の形式

```json
{
    "date": "2022-08-03T15:57:41.265373700Z",
    "win_count_1p": 10,
    "win_count_2p": 30,
    "visible_tumos": 10,
    "json_matches": [
        {
            "won_1p": true,
            "tumos": [
                "RG",
                "YG",
                "GG",
                ...
            ],
            "json_events": [
                {
                    "frame": 376,
                    "json_state_1p": {
                        "tumo_index": 7,
                        "field": "gr///yr/yyry/gggbby/",
                        "score": 0,
                        "ojama_fixed": 0,
                        "ojama_ongoing": 0,
                        "current_chain": 0
                    },
                    "json_state_2p": {
                        "tumo_index": 7,
                        "field": "ggg/r/y/b/byy/gyrry/",
                        "score": 0,
                        "ojama_fixed": 0,
                        "ojama_ongoing": 0,
                        "current_chain": 0
                    }
                },
                ...
            ]
        },
        ...
    ]
}
```
