import json
from os import PathLike
import yaml

import efos_pb2

from google.protobuf.json_format import MessageToJson


def convert(path: PathLike, out: PathLike, data):
    print("Loading marketGroups ...")

    try:
        with open(f"{path}/marketGroups.yaml", encoding="utf-8") as fp:
            marketGroups = yaml.load(fp, Loader=yaml.CSafeLoader)
    except FileNotFoundError:
        with open(f"{path}/marketgroups.json", encoding="utf-8") as fp:
            marketGroups = json.load(fp)
            marketGroups = {int(k): v for k, v in marketGroups.items()}

    data["marketGroups"] = marketGroups
    yield

    print("Converting marketGroups ...")

    pb2 = efos_pb2.MarketGroups()

    for id, entry in marketGroups.items():
        pb2.entries[id].name = entry["nameID"] if isinstance(entry["nameID"], str) else entry["nameID"]["en"]

        if "parentGroupID" in entry:
            pb2.entries[id].parentGroupID = entry["parentGroupID"]
        if "iconID" in entry:
            pb2.entries[id].iconID = entry["iconID"]

    with open(f"{out}/pb2/marketGroups.pb2", "wb") as fp:
        fp.write(pb2.SerializeToString())

    with open(f"{out}/json/marketGroups.json", "w", encoding="utf-8") as fp:
        fp.write(MessageToJson(pb2, sort_keys=True))
