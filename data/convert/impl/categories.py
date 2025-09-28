import json
from os import PathLike


def convert(path: PathLike, loc: dict[int, str], out: PathLike, data):
    print("Loading categories ...")

    with open(f"{path}/categories.json", encoding="utf-8") as fp:
        categories = json.load(fp)
        categories = {int(k): v for k, v in categories.items()}
        for c in categories.values():
            c["name"] = loc[c["categoryNameID"]]

    data["categories"] = categories
    yield
