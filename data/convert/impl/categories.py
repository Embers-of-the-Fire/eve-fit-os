from pathlib import Path

from ..loader import loader


def convert(path: Path, loc: dict[int, str], out: Path, data):
    print("Loading categories ...")

    categories = loader(path / "categories")
    for c in categories.values():
        c["name"] = loc[c["categoryNameID"]]

    data["categories"] = categories
    yield
