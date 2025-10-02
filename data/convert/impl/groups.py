from pathlib import Path

from data.convert.loader import loader


def convert(path: Path, loc: dict[int, str], out: Path, data):
    print("Loading groups ...")

    groups = loader(path / "groups")
    for g in groups.values():
        g["name"] = loc[g["groupNameID"]]

    data["groups"] = groups
    yield
