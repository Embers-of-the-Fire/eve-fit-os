from os import PathLike
import os
import yaml


def load_patches(root: PathLike):
    effects = []
    attributes = []
    typeDogma = []
    buffs = []

    for patch in sorted(os.listdir(root)):
        if not patch.endswith(".yaml"):
            continue
        
        with open(f"{root}/{patch}", 'r', encoding='utf-8') as fp:
            # Fall back to the pure-Python loader when PyYAML lacks the
            # libyaml extension (no prebuilt wheel for the running CPython).
            patch = yaml.load(fp, Loader=getattr(yaml, "CSafeLoader", yaml.SafeLoader))

        for attribute in patch.get("attributes", []):
            attributes.append(attribute)

        for effect in patch.get("effects", []):
            effects.append(effect)

        for entry in patch.get("typeDogma", []):
            typeDogma.append(entry)

        for entry in patch.get("buffs", []):
            buffs.append(entry)

    return {
        "effects": effects,
        "attributes": attributes,
        "typeDogma": typeDogma,
        "buffs": buffs,
    }
