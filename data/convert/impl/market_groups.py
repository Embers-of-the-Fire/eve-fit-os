from pathlib import Path

from ..loader import loader


def convert(path: Path, loc: dict[int, str], out: Path, data):
    print("Loading marketGroups ...")

    marketGroups = loader(path / "marketgroups")

    data["marketGroups"] = marketGroups
    yield

    print("Converting marketGroups ...")
