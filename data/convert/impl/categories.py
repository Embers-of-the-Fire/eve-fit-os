import json
from os import PathLike
import yaml

import efos_pb2

from google.protobuf.json_format import MessageToJson


def convert(path: PathLike, out: PathLike, data):
    print("Loading categories ...")

    try:
        with open(f"{path}/categories.yaml", encoding="utf-8") as fp:
            categories = yaml.load(fp, Loader=yaml.CSafeLoader)
            for category in categories.values():
                category["name"] = category["name"]["en"]
    except FileNotFoundError:
        with open(f"{path}/categories.json", encoding="utf-8") as fp:
            categories = json.load(fp)
            categories = {int(k): v for k, v in categories.items()}
            for category in categories.values():
                category["name"] = category["categoryNameID"]

    data["categories"] = categories
    yield

    print("Converting categories ...")

    pb2 = efos_pb2.Categories()

    for id, entry in categories.items():
        pb2.entries[id].name = entry["name"]
        pb2.entries[id].published = entry["published"]

    with open(f"{out}/pb2/categories.pb2", "wb") as fp:
        fp.write(pb2.SerializeToString())

    with open(f"{out}/json/categories.json", "w", encoding="utf-8") as fp:
        fp.write(MessageToJson(pb2, sort_keys=True))
