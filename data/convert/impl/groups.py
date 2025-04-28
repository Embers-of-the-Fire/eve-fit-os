import json
from os import PathLike

import efos_pb2

from google.protobuf.json_format import MessageToJson


def convert(path: PathLike, loc: dict[int, str], out: PathLike, data):
    print("Loading groups ...")

    with open(f"{path}/groups.json", encoding="utf-8") as fp:
        groups = json.load(fp)
        groups = {int(k): v for k, v in groups.items()}
        for g in groups.values():
            g["name"] = loc[g["groupNameID"]]

    data["groups"] = groups
    yield

    print("Converting groups ...")

    pb2 = efos_pb2.Groups()

    for id, entry in groups.items():
        pb2.entries[id].categoryID = entry["categoryID"]
        pb2.entries[id].published = entry["published"]

    with open(f"{out}/pb2/groups.pb2", "wb") as fp:
        fp.write(pb2.SerializeToString())

    with open(f"{out}/json/groups.json", "w", encoding="utf-8") as fp:
        fp.write(MessageToJson(pb2, sort_keys=True))
