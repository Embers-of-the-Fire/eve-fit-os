from os import PathLike
import os
import yaml


def load_patches(root: PathLike):
    effects = []
    attributes = []
    typeDogma = []

    for patch in sorted(os.listdir(root)):
        if not patch.endswith(".yaml"):
            continue
        
        with open(f"{root}/{patch}", 'r', encoding='utf-8') as fp:
            patch = yaml.load(fp, Loader=yaml.CSafeLoader)

        for attribute in patch.get("attributes", []):
            attributes.append(attribute)

        for effect in patch.get("effects", []):
            effects.append(effect)

        for entry in patch.get("typeDogma", []):
            typeDogma.append(entry)

    return {
        "effects": effects,
        "attributes": attributes,
        "typeDogma": typeDogma,
    }
