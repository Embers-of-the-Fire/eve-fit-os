import json
from os import PathLike

import efos_pb2

from google.protobuf.json_format import MessageToJson


def convert(path: PathLike, loc: dict[int, str], out: PathLike, data):
    print("Loading marketGroups ...")

    with open(f"{path}/marketgroups.json", encoding="utf-8") as fp:
        marketGroups = json.load(fp)
        marketGroups = {int(k): v for k, v in marketGroups.items()}

    data["marketGroups"] = marketGroups
    yield

    print("Converting marketGroups ...")

    pb2 = efos_pb2.MarketGroups()

    for id, entry in marketGroups.items():
        if "parentGroupID" in entry:
            pb2.entries[id].parentGroupID = entry["parentGroupID"]
        if "iconID" in entry:
            pb2.entries[id].iconID = entry["iconID"]

    with open(f"{out}/pb2/marketGroups.pb2", "wb") as fp:
        fp.write(pb2.SerializeToString())

    with open(f"{out}/json/marketGroups.json", "w", encoding="utf-8") as fp:
        fp.write(MessageToJson(pb2, sort_keys=True))
