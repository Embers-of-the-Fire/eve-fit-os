import json
from os import PathLike

import efos_pb2

from google.protobuf.json_format import MessageToJson


def convert(path: PathLike, loc: dict[int, str], out: PathLike, data):
    print("Loading dogmaAttributes ...")

    with open(f"{path}/dogmaattributes.json", encoding="utf-8") as fp:
        dogmaAttributes = json.load(fp)
        dogmaAttributes = {int(k): v for k, v in dogmaAttributes.items()}

    data["dogmaAttributes"] = dogmaAttributes
    yield

    print("Converting dogmaAttributes ...")

    pb2 = efos_pb2.DogmaAttributes()

    for id, entry in dogmaAttributes.items():
        pb2.entries[id].name = entry["name"]
        pb2.entries[id].published = entry["published"]
        pb2.entries[id].defaultValue = entry["defaultValue"]
        pb2.entries[id].highIsGood = entry["highIsGood"]
        pb2.entries[id].stackable = entry["stackable"]

    with open(f"{out}/pb2/dogmaAttributes.pb2", "wb") as fp:
        fp.write(pb2.SerializeToString())

    with open(f"{out}/json/dogmaAttributes.json", "w", encoding="utf-8") as fp:
        fp.write(MessageToJson(pb2, sort_keys=True))
