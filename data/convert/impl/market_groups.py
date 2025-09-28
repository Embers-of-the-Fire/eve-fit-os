import json
from os import PathLike


def convert(path: PathLike, loc: dict[int, str], out: PathLike, data):
    print("Loading marketGroups ...")

    with open(f"{path}/marketgroups.json", encoding="utf-8") as fp:
        marketGroups = json.load(fp)
        marketGroups = {int(k): v for k, v in marketGroups.items()}

    data["marketGroups"] = marketGroups
    yield

    print("Converting marketGroups ...")
