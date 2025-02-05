import json
from os import PathLike
import yaml

import efos_pb2

from google.protobuf.json_format import MessageToJson


def convert(path: PathLike, out: PathLike, data):
    print("Loading groups ...")

    try:
        with open(f"{path}/groups.yaml", encoding="utf-8") as fp:
            groups = yaml.load(fp, Loader=yaml.CSafeLoader)
            for group in groups.values():
                group["name"] = group["name"]["en"]
    except FileNotFoundError:
        with open(f"{path}/groups.json", encoding="utf-8") as fp:
            groups = json.load(fp)
            groups = {int(k): v for k, v in groups.items()}
            for group in groups.values():
                group["name"] = group["groupNameID"]

    data["groups"] = groups
    yield

    print("Converting groups ...")

    pb2 = efos_pb2.Groups()

    for id, entry in groups.items():
        pb2.entries[id].name = entry["name"]
        pb2.entries[id].categoryID = entry["categoryID"]
        pb2.entries[id].published = entry["published"]

    with open(f"{out}/pb2/groups.pb2", "wb") as fp:
        fp.write(pb2.SerializeToString())

    with open(f"{out}/json/groups.json", "w", encoding="utf-8") as fp:
        fp.write(MessageToJson(pb2, sort_keys=True))
