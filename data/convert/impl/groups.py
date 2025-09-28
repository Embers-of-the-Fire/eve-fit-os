import json
from os import PathLike


def convert(path: PathLike, loc: dict[int, str], out: PathLike, data):
    print("Loading groups ...")

    with open(f"{path}/groups.json", encoding="utf-8") as fp:
        groups = json.load(fp)
        groups = {int(k): v for k, v in groups.items()}
        for g in groups.values():
            g["name"] = loc[g["groupNameID"]]

    data["groups"] = groups
    yield
