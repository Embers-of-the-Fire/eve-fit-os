import json
from os import PathLike

import yaml

import efos_pb2

from google.protobuf.json_format import MessageToJson


def convert(path: PathLike, patch: PathLike, out: PathLike, data):
    print("Loading typeDogma ...")

    with open(f"{path}/typedogma.json", encoding="utf-8") as fp:
        typeDogma = json.load(fp)
        typeDogma = {int(k): v for k, v in typeDogma.items()}

    with open(f"{patch}/ignoredDogma.yaml", encoding="utf-8") as f:
        ignored_types = yaml.load(f, yaml.CLoader)

    data["typeDogma"] = typeDogma
    yield

    print("Converting typeDogma ...")

    pb2 = efos_pb2.TypeDogma()

    for id, entry in typeDogma.items():
        if id in ignored_types:
            continue
        for attribute in entry["dogmaAttributes"]:
            pbea = pb2.TypeDogmaEntry.DogmaAttributes(
                attributeID=attribute["attributeID"], value=attribute["value"]
            )

            pb2.entries[id].dogmaAttributes.append(pbea)

        for effect in entry["dogmaEffects"]:
            pbee = pb2.TypeDogmaEntry.DogmaEffects(
                effectID=effect["effectID"], isDefault=effect["isDefault"]
            )

            pb2.entries[id].dogmaEffects.append(pbee)

    with open(f"{out}/pb2/typeDogma.pb2", "wb") as fp:
        fp.write(pb2.SerializeToString())

    with open(f"{out}/json/typeDogma.json", "w", encoding="utf-8") as fp:
        fp.write(MessageToJson(pb2, sort_keys=True))
