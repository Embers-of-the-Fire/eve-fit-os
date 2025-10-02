from pathlib import Path

from ..loader import loader
import efos_pb2

from google.protobuf.json_format import MessageToJson


def convert(path: Path, out: Path, data):
    print("Loading dogmaEffects ...")

    dogmaEffects = loader(path / "dogmaeffects")

    data["dogmaEffects"] = dogmaEffects
    yield

    print("Converting dogmaEffects ...")

    pb2 = efos_pb2.DogmaEffects()
    pbmi = pb2.DogmaEffect.ModifierInfo()

    for id, entry in dogmaEffects.items():
        pb2.entries[id].name = entry["effectName"]
        pb2.entries[id].effectCategory = entry["effectCategory"]
        if "modifierInfo" in entry:
            for modifier_info in entry["modifierInfo"]:
                pbmi = pb2.DogmaEffect.ModifierInfo()

                match modifier_info["domain"]:
                    case "itemID":
                        pbmi.domain = pbmi.Domain.itemID
                    case "shipID":
                        pbmi.domain = pbmi.Domain.shipID
                    case "charID":
                        pbmi.domain = pbmi.Domain.charID
                    case "otherID":
                        pbmi.domain = pbmi.Domain.otherID
                    case "structureID":
                        pbmi.domain = pbmi.Domain.structureID
                    case "target":
                        pbmi.domain = pbmi.Domain.target
                    case "targetID":
                        pbmi.domain = pbmi.Domain.targetID
                    case _:
                        raise ValueError(f"Unknown domain: {modifier_info['domain']}")

                match modifier_info["func"]:
                    case "ItemModifier":
                        pbmi.func = pbmi.Func.ItemModifier
                    case "LocationGroupModifier":
                        pbmi.func = pbmi.Func.LocationGroupModifier
                    case "LocationModifier":
                        pbmi.func = pbmi.Func.LocationModifier
                    case "LocationRequiredSkillModifier":
                        pbmi.func = pbmi.Func.LocationRequiredSkillModifier
                    case "OwnerRequiredSkillModifier":
                        pbmi.func = pbmi.Func.OwnerRequiredSkillModifier
                    case "EffectStopper":
                        pbmi.func = pbmi.Func.EffectStopper
                    case _:
                        raise ValueError(f"Unknown func: {modifier_info['func']}")

                if "modifiedAttributeID" in modifier_info:
                    pbmi.modifiedAttributeID = modifier_info["modifiedAttributeID"]
                if "modifyingAttributeID" in modifier_info:
                    pbmi.modifyingAttributeID = modifier_info["modifyingAttributeID"]
                if "operation" in modifier_info:
                    pbmi.operation = modifier_info["operation"]
                if "groupID" in modifier_info:
                    pbmi.groupID = modifier_info["groupID"]
                if "skillTypeID" in modifier_info:
                    pbmi.skillTypeID = modifier_info["skillTypeID"]

                pb2.entries[id].modifierInfo.append(pbmi)

    with open(f"{out}/pb2/dogmaEffects.pb2", "wb") as fp:
        fp.write(pb2.SerializeToString())

    with open(f"{out}/json/dogmaEffects.json", "w", encoding="utf-8") as fp:
        fp.write(MessageToJson(pb2, sort_keys=True))
