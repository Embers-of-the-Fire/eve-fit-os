import os
import pickle

from dotenv import load_dotenv


load_dotenv()

from .paths import FSD_BINARY_DIR, FSD_LOC_EN_DIR, FSD_PATCH_DIR, OUTPUT_DIR, PATCH_DIR  # noqa: E402
from .impl import (  # noqa: E402
    dogma_attributes,
    dogma_effects,
    types,
    type_dogma,
    dbuffcollections,
    categories,
    groups,
    market_groups,
)
from .patches import (  # noqa: E402
    dogma_attributes as patch_dogma_attributes,
    dogma_effects as patch_dogma_effects,
    type_dogma as patch_type_dogma,
)
from .patches.loader import load_patches  # noqa: E402

if FSD_BINARY_DIR is None or not FSD_BINARY_DIR.exists():
    print("No `FSD_BINARY_DIR` set, or it does not exist.")
    exit(1)
if FSD_LOC_EN_DIR is None or not FSD_LOC_EN_DIR.exists():
    print("No `FSD_LOC_EN_DIR` set, or it does not exist.")
    exit(1)

os.makedirs(f"{OUTPUT_DIR}/pb2", exist_ok=True)
os.makedirs(f"{OUTPUT_DIR}/json", exist_ok=True)

with open(FSD_LOC_EN_DIR, "rb") as fp:
    _, en = pickle.load(fp)
    loc = {key: value[0] for key, value in en.items()}

data = {}
gens = []

gens.append(types.convert(FSD_BINARY_DIR, loc, OUTPUT_DIR, data))
gens.append(dogma_attributes.convert(FSD_BINARY_DIR, OUTPUT_DIR, data))
gens.append(dogma_effects.convert(FSD_BINARY_DIR, OUTPUT_DIR, data))
gens.append(type_dogma.convert(FSD_BINARY_DIR, FSD_PATCH_DIR, OUTPUT_DIR, data))
gens.append(dbuffcollections.convert(FSD_PATCH_DIR, OUTPUT_DIR, data))
gens.append(groups.convert(FSD_BINARY_DIR, loc, OUTPUT_DIR, data))
gens.append(categories.convert(FSD_BINARY_DIR, loc, OUTPUT_DIR, data))
gens.append(market_groups.convert(FSD_BINARY_DIR, loc, OUTPUT_DIR, data))

# First iteration updates "data" with all the name -> ID mappings.
for gen in gens:
    next(gen)

patches = load_patches(PATCH_DIR)

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
