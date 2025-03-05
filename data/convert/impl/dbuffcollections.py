from os import PathLike
import yaml

import efos_pb2

from google.protobuf.json_format import MessageToJson


def convert(path: PathLike, out: PathLike, data):
    print("Loading DBuffCollections ...")

    with open(f"{path}/dbuffcollections.yaml", encoding="utf-8") as fp:
        buffs = yaml.load(fp, Loader=yaml.CSafeLoader)

    data["dbuffcollections"] = buffs
    yield

    print("Converting DBuffCollections ...")

    pb2 = efos_pb2.BuffCollections()

    for id, entry in buffs.items():
        pb2.entries[id].aggregateMode = {
            "Maximum": efos_pb2.BuffCollections.Buff.AggregateMode.MAXIMUM,
            "Minimum": efos_pb2.BuffCollections.Buff.AggregateMode.MINIMUM,
        }[entry["aggregateMode"]]
        pb2.entries[id].buffID = id
        pb2.entries[id].developerDescription = entry["developerDescription"]

        for x in entry.get("itemModifiers", []):
            cache = efos_pb2.BuffCollections.Buff.ItemModifier()
            cache.dogmaAttributeID = x["dogmaAttributeID"]
            pb2.entries[id].itemModifiers.append(cache)

        for x in entry.get("locationModifiers", []):
            cache = efos_pb2.BuffCollections.Buff.ItemModifier()
            cache.dogmaAttributeID = x["dogmaAttributeID"]
            pb2.entries[id].locationModifiers.append(cache)

        for x in entry.get("locationGroupModifiers", []):
            cache = efos_pb2.BuffCollections.Buff.LocationGroupModifier()
            cache.dogmaAttributeID = x["dogmaAttributeID"]
            cache.groupID = x["groupID"]
            pb2.entries[id].locationGroupModifiers.append(cache)

        for x in entry.get("locationRequiredSkillModifiers", []):
            cache = efos_pb2.BuffCollections.Buff.LocationRequiredSkillModifier()
            cache.dogmaAttributeID = x["dogmaAttributeID"]
            cache.skillID = x["skillID"]
            pb2.entries[id].locationRequiredSkillModifiers.append(cache)

        pb2.entries[id].operationName = {
            "PostMul": efos_pb2.BuffCollections.Buff.OperationName.POST_MUL,
            "PostPercent": efos_pb2.BuffCollections.Buff.OperationName.POST_PERCENT,
            "ModAdd": efos_pb2.BuffCollections.Buff.OperationName.MOD_ADD,
            "PostAssignment": efos_pb2.BuffCollections.Buff.OperationName.POST_ASSIGNMENT,
        }[entry["operationName"]]
        pb2.entries[id].showOutputValueInUI = {
            "ShowNormal": efos_pb2.BuffCollections.Buff.ShowOutputValueInUI.SHOW_NORMAL,
            "ShowInverted": efos_pb2.BuffCollections.Buff.ShowOutputValueInUI.SHOW_INVERTED,
            "Hide": efos_pb2.BuffCollections.Buff.ShowOutputValueInUI.HIDE,
        }[entry["showOutputValueInUI"]]

        if "displayName" in entry.keys():
            pb2.entries[id].displayName = entry["displayName"]

    with open(f"{out}/pb2/dbuffcollections.pb2", "wb") as fp:
        fp.write(pb2.SerializeToString())

    with open(f"{out}/json/dbuffcollections.json", "w", encoding="utf-8") as fp:
        fp.write(MessageToJson(pb2, sort_keys=True))
