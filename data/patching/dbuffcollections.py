import json
import pickle
import sqlite3
from pathlib import Path

import yaml

PATCHING_ROOT = Path(__file__).parent
DATA_ROOT = PATCHING_ROOT.parent

DEFAULT_OUTPUT = DATA_ROOT / "fsd-patches" / "dbuffcollections.yaml"


def load_localization(path: Path) -> dict:
    with open(path, "rb") as fp:
        _, loc = pickle.load(fp)
    return loc


def convert(static_file: Path, output: Path, localization: Path | None) -> None:
    loc = load_localization(localization) if localization else None

    db = sqlite3.connect(static_file)
    cursor = db.cursor()

    data = {}
    for row in cursor.execute("SELECT * FROM cache"):
        buff_id = int(row[0])
        content = json.loads(row[1])
        if loc is not None and "displayNameID" in content:
            content["displayName"] = loc[content["displayNameID"]][0]
        content["buffID"] = buff_id
        data[buff_id] = content
    db.close()

    output.parent.mkdir(parents=True, exist_ok=True)
    with open(output, "w+", encoding="utf-8") as fp:
        yaml.dump(data, fp, allow_unicode=True, indent=2, sort_keys=True)

    print(f"Wrote {len(data)} buff entries to {output}")
