import os
import sys

from .impl import (
    categories,
    dogma_attributes,
    dogma_effects,
    groups,
    market_groups,
    types,
    type_dogma,
    dbuffcollections,
)
from .patches import (
    dogma_attributes as patch_dogma_attributes,
    dogma_effects as patch_dogma_effects,
    type_dogma as patch_type_dogma,
)
from .patches.loader import load_patches


if len(sys.argv) != 5:
    print(
        "Usage: python3 convert.py <path/to/eve-sde/fsd> <path/to/eve-sde/fsd-patches> <path/to/patches> <path/to/output>"
    )
    exit(1)

fsd_dir = sys.argv[1]
fsd_patch_dir = sys.argv[2]
patch_dir = sys.argv[3]
out_dir = sys.argv[4]

os.makedirs(f"{out_dir}/pb2", exist_ok=True)
os.makedirs(f"{out_dir}/json", exist_ok=True)

data = {}
gens = []

gens.append(categories.convert(fsd_dir, out_dir, data))
gens.append(groups.convert(fsd_dir, out_dir, data))
gens.append(types.convert(fsd_dir, out_dir, data))
gens.append(market_groups.convert(fsd_dir, out_dir, data))
gens.append(dogma_attributes.convert(fsd_dir, out_dir, data))
gens.append(dogma_effects.convert(fsd_dir, out_dir, data))
gens.append(type_dogma.convert(fsd_dir, out_dir, data))
gens.append(dbuffcollections.convert(fsd_patch_dir, out_dir, data))

# First iteration updates "data" with all the name -> ID mappings.
for gen in gens:
    next(gen)

patches = load_patches(patch_dir)

# Patch all data.
patch_dogma_attributes.patch(data["dogmaAttributes"], patches["attributes"], data)
patch_dogma_effects.patch(data["dogmaEffects"], patches["effects"], data)
patch_type_dogma.patch(data["typeDogma"], patches["typeDogma"], data)

# Second iteration actually writes all the information.
for gen in gens:
    try:
        next(gen)
    except StopIteration:
        pass
