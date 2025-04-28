import json
from os import PathLike

import efos_pb2

from google.protobuf.json_format import MessageToJson


def convert(path: PathLike, loc: dict[int, str], out: PathLike, data):
    print("Loading categories ...")

    with open(f"{path}/categories.json", encoding="utf-8") as fp:
        categories = json.load(fp)
        categories = {int(k): v for k, v in categories.items()}
        for c in categories.values():
            c["name"] = loc[c["categoryNameID"]]

    data["categories"] = categories
    yield

    print("Converting categories ...")

    pb2 = efos_pb2.Categories()

    for id, entry in categories.items():
        pb2.entries[id].published = entry["published"]

    with open(f"{out}/pb2/categories.pb2", "wb") as fp:
        fp.write(pb2.SerializeToString())

    with open(f"{out}/json/categories.json", "w", encoding="utf-8") as fp:
        fp.write(MessageToJson(pb2, sort_keys=True))
