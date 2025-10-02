import os
from pathlib import Path
from typing import Any

import json
import msgpack

def __json_loader(source: Path) -> dict[int, Any]:
    with open(source.with_suffix(".json"), "r", encoding="utf-8") as fp:
        data = json.load(fp)
        data = {int(k): v for k, v in data.items()}
    
    return data


def __msgpack_loader(source: Path) -> dict[int, Any]:
    with open(source.with_suffix(".msgpack"), "rb") as fp:
        data = msgpack.unpack(fp, strict_map_key=False)
        data = {int(k): v for k, v in data.items()}
    
    return data


loader = __json_loader if "FSD_FORMAT" in os.environ and os.environ["FSD_FORMAT"] == "json" else __msgpack_loader
