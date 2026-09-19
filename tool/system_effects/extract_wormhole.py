#!/usr/bin/env python3
"""Extract system-effect beacon data from our own per-server FSD dumps.

Reads `data/resources/<server>/fsd/*.msgpack` (plus the workspace en-US
localization pickle for skill/group names) for every server and emits
`tool/system_effects/out/beacons.json`:

  - attributes / skills / groups: name -> ID maps (resolved per server,
    verified identical across servers)
  - wormhole_classes: wormhole type -> class -> beacon type ID
  - beacon_values: canonical-server attribute table of every effect beacon
    (wormhole, storm, abyssal weather, abyssal hazard clouds)

A per-server value report is printed; any divergence between servers is a
WARNING (not an error): the app resolves preset values from the ACTIVE
snapshot at runtime, so per-server divergence is tolerated by construction.
"""

import argparse
import collections
import json
import pickle
import sys
from pathlib import Path

import msgpack

REPO_ROOT = Path(__file__).resolve().parents[4]
OUT = Path(__file__).resolve().parent / "out" / "beacons.json"

DEFAULT_SERVERS = ["tranquility", "serenity", "singularity"]

# Wormhole effect beacons: type ID -> (wormhole type, class).
BEACONS = {
    30844: ("Pulsar", 1), 30845: ("Black Hole", 1), 30846: ("Cataclysmic Variable", 1),
    30847: ("Magnetar", 1), 30848: ("Red Giant", 1), 30849: ("Wolf Rayet", 1),
    30850: ("Black Hole", 2), 30851: ("Black Hole", 3), 30852: ("Black Hole", 4),
    30853: ("Black Hole", 5), 30854: ("Black Hole", 6),
    30860: ("Magnetar", 2), 30861: ("Magnetar", 3), 30862: ("Magnetar", 4),
    30863: ("Magnetar", 5), 30864: ("Magnetar", 6),
    30865: ("Pulsar", 2), 30866: ("Pulsar", 3), 30867: ("Pulsar", 4),
    30868: ("Pulsar", 5), 30869: ("Pulsar", 6),
    30870: ("Red Giant", 2), 30871: ("Red Giant", 3), 30872: ("Red Giant", 4),
    30873: ("Red Giant", 5), 30874: ("Red Giant", 6),
    30875: ("Wolf Rayet", 2), 30876: ("Wolf Rayet", 3), 30877: ("Wolf Rayet", 4),
    30878: ("Wolf Rayet", 5), 30879: ("Wolf Rayet", 6),
    30880: ("Cataclysmic Variable", 2), 30881: ("Cataclysmic Variable", 3),
    30884: ("Cataclysmic Variable", 4), 30883: ("Cataclysmic Variable", 5),
    30882: ("Cataclysmic Variable", 6),
}

# Metaliminal/volatile storm beacons: storm key -> strength -> type ID.
STORM_BEACONS = {
    "electrical": {"strong": 56057, "weak": 56064},
    "exotic": {"strong": 56059, "weak": 56058},
    "gamma": {"strong": 56061, "weak": 56060},
    "plasma": {"strong": 56063, "weak": 56062},
    "icestorm": {"strong": 56967, "weak": 56997},
}

# Abyssal weather beacons: weather key -> tier -> type ID (warfareBuff pairs).
WEATHER_BEACONS = {
    "weather_darkness": {1: 47378, 2: 47379, 3: 47380},
    "weather_electrical": {1: 47381, 2: 47382, 3: 47383},
    "weather_caustic": {1: 47384, 2: 47385, 3: 47386},
    "weather_exotic": {1: 47387, 2: 47388, 3: 47389},
    "weather_infernal": {1: 47390, 2: 47391, 3: 47392},
}

# Abyssal hazard cloud beacons: hazard key -> type ID (warfareBuff pairs).
HAZARD_BEACONS = {
    "hazard_bioluminescence": 47439,
    "hazard_caustic": 47436,
    "hazard_filament_small": 47620,
    "hazard_filament_medium": 47472,
    "hazard_filament_large": 47473,
}

# Dogma attribute names referenced by the generators: ship/charge modifier
# targets, and the beacon-side "multiplier/bonus" attributes the preset
# values are read from.
ATTR_NAMES = [
    "shieldCapacity", "armorHP", "signatureRadius", "maxTargetRange", "maxVelocity",
    "agility", "rechargeRate", "capacitorCapacity", "damageMultiplier", "trackingSpeed",
    "emDamage", "thermalDamage", "kineticDamage", "explosiveDamage",
    "aoeVelocity", "aoeCloudSize", "speedFactor",
    "shieldEmDamageResonance", "shieldThermalDamageResonance",
    "shieldKineticDamageResonance", "shieldExplosiveDamageResonance",
    "armorEmDamageResonance", "armorThermalDamageResonance",
    "armorKineticDamageResonance", "armorExplosiveDamageResonance",
    "armorDamageAmount", "shieldBonus", "powerTransferAmount",
    "energyNeutralizerAmount", "signatureRadiusBonus", "heatDamage", "empFieldRange",
    "overloadArmorDamageAmount", "overloadDamageModifier", "overloadDurationBonus",
    "overloadECCMStrenghtBonus", "overloadECMStrengthBonus", "overloadHardeningBonus",
    "overloadRangeBonus", "overloadRofBonus", "overloadSelfDurationBonus",
    "overloadShieldBonus", "overloadSpeedFactorBonus",
    "scanGravimetricStrengthBonus", "scanLadarStrengthBonus",
    "scanMagnetometricStrengthBonus", "scanRadarStrengthBonus",
    # Beacon-side value attributes (wormholes and storms).
    "shieldCapacityMultiplier", "armorHPMultiplier", "signatureRadiusMultiplier",
    "maxTargetRangeMultiplier", "maxVelocityMultiplier", "agilityMultiplier",
    "trackingSpeedMultiplier", "damageMultiplierMultiplier", "aoeVelocityMultiplier",
    "aoeCloudSizeMultiplier", "smallWeaponDamageMultiplier", "heatDamageMultiplier",
    "overloadBonusMultiplier", "empFieldRangeMultiplier", "smartbombDamageMultiplier",
    "missileVelocityMultiplier", "armorDamageAmountMultiplier", "shieldBonusMultiplier",
    "shieldBonusMultiplierRemote", "armorDamageAmountMultiplierRemote",
    "capacitorCapacityMultiplierSystem", "rechargeRateMultiplier",
    "energyTransferAmountBonus", "energyWarfareStrengthMultiplier",
    "targetPainterStrengthMultiplier", "stasisWebStrengthMultiplier",
    "emDamageResistanceBonus", "explosiveDamageResistanceBonus",
    "kineticDamageResistanceBonus", "thermalDamageResistanceBonus",
    "armorEmDamageResistanceBonus", "armorKineticDamageResistanceBonus",
    "armorThermalDamageResistanceBonus", "armorExplosiveDamageResistanceBonus",
    "shieldEmDamageResistanceBonus", "shieldKineticDamageResistanceBonus",
    "shieldThermalDamageResistanceBonus", "shieldExplosiveDamageResistanceBonus",
    "miningDurationMultiplier", "warpSpeedBonus", "scanResolutionBonus",
    "thermodynamicsHeatDamage", "armorRepairDurationBonus", "shieldBoosterDurationBonus",
    "warfareBuff1ID", "warfareBuff1Value", "warfareBuff2ID", "warfareBuff2Value",
    "warfareBuff3ID", "warfareBuff3Value", "warfareBuff4ID", "warfareBuff4Value",
]

SKILL_NAMES = [
    "Gunnery", "Missile Launcher Operation", "Rockets", "Light Missiles",
    "Bomb Deployment", "Drones", "Fighters", "Repair Systems", "Capital Repair Systems",
    "Remote Armor Repair Systems", "Capital Remote Armor Repair Systems",
    "Shield Emission Systems", "Capital Shield Emission Systems",
    "Shield Operation", "Capital Shield Operation", "Target Painting",
    "Small Energy Turret", "Small Projectile Turret", "Small Hybrid Turret",
    "Small Precursor Weapon", "Small Vorton Projector", "Vorton Projector Operation",
]

GROUP_NAMES = [
    "Smart Bomb", "Energy Neutralizer", "Energy Nosferatu", "Stasis Web",
    "Remote Capacitor Transmitter",
]

WARFARE_PAIRS = [("warfareBuff1ID", "warfareBuff1Value"), ("warfareBuff2ID", "warfareBuff2Value"),
                 ("warfareBuff3ID", "warfareBuff3Value"), ("warfareBuff4ID", "warfareBuff4Value")]


def load_msgpack(path: Path) -> dict[int, dict]:
    with open(path, "rb") as fp:
        return {int(k): v for k, v in msgpack.unpack(fp, strict_map_key=False).items()}


def load_localization(path: Path) -> dict[int, str]:
    with open(path, "rb") as fp:
        _, loc = pickle.load(fp)
    return {key: value[0] for key, value in loc.items()}


def all_beacon_ids() -> list[int]:
    ids = set(BEACONS)
    for m in STORM_BEACONS.values():
        ids.update(m.values())
    for m in WEATHER_BEACONS.values():
        ids.update(m.values())
    ids.update(HAZARD_BEACONS.values())
    return sorted(ids)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--servers", nargs="+", default=DEFAULT_SERVERS,
                        help="servers to extract (first one is the canonical value source)")
    parser.add_argument("--resources-root", type=Path, default=REPO_ROOT / "data" / "resources")
    parser.add_argument("--workspaces-root", type=Path, default=REPO_ROOT / "cache" / "workspaces",
                        help="workspace root holding the per-server localization pickles")
    parser.add_argument("--out", type=Path, default=OUT)
    args = parser.parse_args()

    servers: dict[str, dict] = {}
    for server in args.servers:
        fsd = args.resources_root / server / "fsd"
        loc_path = (args.workspaces_root / server / "res" / "localizationfsd"
                    / "localization_fsd_en-us.pickle")
        for p in (fsd / "typedogma.msgpack", fsd / "dogmaattributes.msgpack",
                  fsd / "types.msgpack", fsd / "groups.msgpack", loc_path):
            if not p.exists():
                raise SystemExit(f"missing input: {p} (run ./x build data {server} first)")
        loc = load_localization(loc_path)
        servers[server] = {
            "typedogma": load_msgpack(fsd / "typedogma.msgpack"),
            "attributes": load_msgpack(fsd / "dogmaattributes.msgpack"),
            "types": load_msgpack(fsd / "types.msgpack"),
            "groups": load_msgpack(fsd / "groups.msgpack"),
            "loc": loc,
        }

    canonical = args.servers[0]
    warnings = 0

    def resolve_names(server: str) -> tuple[dict, dict, dict]:
        data = servers[server]
        attr_by_name = {}
        for aid, v in data["attributes"].items():
            attr_by_name.setdefault(v.get("name"), aid)
        skill_by_name = {}
        for tid, v in data["types"].items():
            name = data["loc"].get(v.get("typeNameID"))
            if name in SKILL_NAMES:
                skill_by_name[name] = tid
        group_by_name = {}
        for gid, v in data["groups"].items():
            name = data["loc"].get(v.get("groupNameID"))
            if name in GROUP_NAMES:
                group_by_name[name] = gid
        return (
            {n: attr_by_name.get(n) for n in ATTR_NAMES},
            {n: skill_by_name.get(n) for n in SKILL_NAMES},
            {n: group_by_name.get(n) for n in GROUP_NAMES},
        )

    def check_maps():
        nonlocal warnings
        ref = None
        for server in args.servers:
            maps = resolve_names(server)
            missing = [n for m in maps for n, v in m.items() if v is None]
            if missing:
                raise SystemExit(f"{server}: could not resolve names: {missing}")
            if ref is None:
                ref = maps
                continue
            for label, got, want in zip(("attributes", "skills", "groups"), maps, ref):
                for name, v in got.items():
                    if v != want[name]:
                        print(f"WARNING: {server}: {label}[{name!r}] = {v}, "
                              f"{canonical} has {want[name]}")
                        warnings += 1
        assert ref is not None
        return ref

    attributes, skills, groups = check_maps()

    def beacon_attrs(server: str, type_id: int) -> dict[int, float] | None:
        entry = servers[server]["typedogma"].get(type_id)
        if entry is None:
            return None
        return {a["attributeID"]: a["value"] for a in entry.get("dogmaAttributes", [])}

    beacon_values = {}
    print(f"per-server beacon report (canonical: {canonical})")
    for tid in all_beacon_ids():
        per_server = {s: beacon_attrs(s, tid) for s in args.servers}
        ref = per_server[canonical]
        if ref is None:
            raise SystemExit(f"{canonical}: beacon {tid} missing from typedogma")
        beacon_values[tid] = ref
        for server, attrs in per_server.items():
            if server == canonical:
                continue
            if attrs != ref:
                print(f"WARNING: beacon {tid}: {server} dogma differs from {canonical}: "
                      f"{attrs} != {ref}")
                warnings += 1
        for s, a in per_server.items():
            if not a:
                print(f"WARNING: beacon {tid}: no dogma on {s}")
                warnings += 1

    wormhole_classes: dict[str, dict[int, int]] = collections.defaultdict(dict)
    for tid, (wtype, cls) in BEACONS.items():
        wormhole_classes[wtype][cls] = tid

    out = {
        "server": canonical,
        "attributes": attributes,
        "skills": skills,
        "groups": groups,
        "wormhole_classes": {k: dict(v) for k, v in wormhole_classes.items()},
        "storm_beacons": STORM_BEACONS,
        "weather_beacons": WEATHER_BEACONS,
        "hazard_beacons": HAZARD_BEACONS,
        "beacon_values": {str(k): {str(a): v for a, v in vals.items()}
                          for k, vals in beacon_values.items()},
    }
    args.out.parent.mkdir(parents=True, exist_ok=True)
    with open(args.out, "w", encoding="utf-8") as fp:
        json.dump(out, fp, indent=1)

    print(f"wrote {args.out}: {len(beacon_values)} beacons, "
          f"{len(attributes)} attributes, {len(skills)} skills, {len(groups)} groups")
    if warnings:
        print(f"{warnings} warning(s) (cross-server divergence is tolerated, not fatal)")
    sys.exit(0)


if __name__ == "__main__":
    main()
