import json
from os import PathLike

import efos_pb2

from google.protobuf.json_format import MessageToJson


def convert(path: PathLike, loc: dict[int, str], out: PathLike, data):
    print("Loading types ...")

    with open(f"{path}/types.json", encoding="utf-8") as fp:
        types = json.load(fp)
        types = {int(k): v for k, v in types.items()}
        for ty in types.values():
            ty["name"] = loc[ty["typeNameID"]]

    data["types"] = types
    yield

    print("Converting types ...")

    pb2 = efos_pb2.Types()

    for id, entry in types.items():
        pb2.entries[id].groupID = entry["groupID"]
        pb2.entries[id].categoryID = data["groups"][entry["groupID"]]["categoryID"]
        pb2.entries[id].published = entry["published"]

        if "capacity" in entry and entry["capacity"] != 0.0:
            pb2.entries[id].capacity = entry["capacity"]
        if "mass" in entry and entry["mass"] != 0.0:
            pb2.entries[id].mass = entry["mass"]
        if "radius" in entry and entry["radius"] != 1.0:
            pb2.entries[id].radius = entry["radius"]
        if "volume" in entry and entry["volume"] != 0.0:
            pb2.entries[id].volume = entry["volume"]

    with open(f"{out}/pb2/types.pb2", "wb") as fp:
        fp.write(pb2.SerializeToString())

    with open(f"{out}/json/types.json", "w", encoding="utf-8") as fp:
        fp.write(MessageToJson(pb2, sort_keys=True))
