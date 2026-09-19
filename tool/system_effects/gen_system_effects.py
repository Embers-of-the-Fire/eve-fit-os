#!/usr/bin/env python3
"""Generate:
1. packages/eve-fit-os/data/patches/system_effects.yaml
   - wormhole system-effect buffs (-2001..-2032)
   - metaliminal/volatile storm buffs  (-2101..-2121)
2. tool/system_effects/out/environment_catalog.json
   - resolution rules for the app catalog (beacon type IDs + beacon
     attribute -> buff mappings / warfareBuff pairs / accepted-risk statics),
     consumed by gen_dart_catalog.py

Input: tool/system_effects/out/beacons.json (extract_wormhole.py), sourced
from our own per-server FSD dumps. Buff definitions carry no values; preset
values are resolved from the active snapshot at runtime by the app.
"""

import json
from pathlib import Path

import yaml

REPO_ROOT = Path(__file__).resolve().parents[2]

RAW = json.load(open(REPO_ROOT / "tool/system_effects/out/beacons.json"))
A = RAW["attributes"]
S = RAW["skills"]
G = RAW["groups"]
WH_CLASSES = RAW["wormhole_classes"]      # wtype -> class -> beacon type id
STORM_BEACONS = RAW["storm_beacons"]      # stype -> strength -> beacon type id
WEATHER_BEACONS = RAW["weather_beacons"]  # key -> tier -> beacon type id
HAZARD_BEACONS = RAW["hazard_beacons"]    # key -> beacon type id
BEACON_VALUES = RAW["beacon_values"]      # beacon type id -> attr id -> value

MLO = S["Missile Launcher Operation"]

# extra attribute IDs resolved from the engine dogma attributes
HULL_EM, HULL_THERM, HULL_KIN, HULL_EXP = 113, 110, 109, 111
WARP_SPEED = 600
SCAN_RES = 564
DURATION = 73
FA_AM_EXP_RADIUS = 2234  # fighterAbilityAttackMissileExplosionRadius
FA_M_EXP_RADIUS = 2125   # fighterAbilityMissilesExplosionRadius
MINING = 3386

# ---------------------------------------------------------------------------
# Buff definitions shared by the yaml patch and (for names) the catalog.
# ---------------------------------------------------------------------------

def buff(key, en, aggregate, penalized, op="PostMul", **mods):
    return {"key": key, "en": en, "aggregate": aggregate,
            "penalized": penalized, "op": op, "mods": mods}


AUTHORED = []          # (buff_id assigned sequentially)
def add(*args, **kwargs):
    AUTHORED.append(buff(*args, **kwargs))
    return AUTHORED[-1]


# --- Wormholes: id -2001..-2032; preset value = beacon attr per type/class
WH = []  # (defn, wormhole_type, beacon_attr_name)

def wh(*args, **kwargs):
    b = buff(*args, **kwargs)
    AUTHORED.append(b)
    return b

# Pulsar
WH.append((wh("pulsar_shield_hp", "Shield Hitpoint Bonus (Pulsar)", "Maximum", False,
              item=[A["shieldCapacity"]]), "Pulsar", "shieldCapacityMultiplier"))
WH.append((wh("pulsar_sig_radius", "Signature Radius Penalty (Pulsar)", "Maximum", True,
              item=[A["signatureRadius"]]), "Pulsar", "signatureRadiusMultiplier"))
WH.append((wh("pulsar_armor_res", "Armor Resistance Penalty (Pulsar)", "Maximum", True, "PostPercent",
              item=[A["armorEmDamageResonance"], A["armorThermalDamageResonance"],
                    A["armorKineticDamageResonance"], A["armorExplosiveDamageResonance"]]),
           "Pulsar", "armorEmDamageResistanceBonus"))
WH.append((wh("pulsar_cap_recharge", "Capacitor Recharge Time Bonus (Pulsar)", "Minimum", False,
              item=[A["rechargeRate"]]), "Pulsar", "rechargeRateMultiplier"))
WH.append((wh("pulsar_energy_warfare", "Energy Warfare Strength Bonus (Pulsar)", "Maximum", True,
              locationGroup=[(A["energyNeutralizerAmount"], G["Energy Neutralizer"]),
                             (A["powerTransferAmount"], G["Energy Nosferatu"])]),
           "Pulsar", "energyWarfareStrengthMultiplier"))
# Black Hole
WH.append((wh("blackhole_agility", "Agility Penalty (Black Hole)", "Maximum", True,
              item=[A["agility"]]), "Black Hole", "agilityMultiplier"))
WH.append((wh("blackhole_target_range", "Targeting Range Bonus (Black Hole)", "Maximum", True,
              item=[A["maxTargetRange"]]), "Black Hole", "maxTargetRangeMultiplier"))
WH.append((wh("blackhole_missile_velocity", "Missile Velocity Bonus (Black Hole)", "Maximum", True,
              chargeSkill=[(A["maxVelocity"], MLO)]), "Black Hole", "missileVelocityMultiplier"))
WH.append((wh("blackhole_velocity", "Maximum Velocity Bonus (Black Hole)", "Maximum", True,
              item=[A["maxVelocity"]]), "Black Hole", "maxVelocityMultiplier"))
WH.append((wh("blackhole_explosion_velocity", "Missile Explosion Velocity Bonus (Black Hole)", "Maximum", True,
              chargeSkill=[(A["aoeVelocity"], MLO)],
              locationSkill=[(A["aoeVelocity"], S["Vorton Projector Operation"])]),
           "Black Hole", "aoeVelocityMultiplier"))
WH.append((wh("blackhole_web_strength", "Stasis Webifier Strength Penalty (Black Hole)", "Minimum", True,
              locationGroup=[(A["speedFactor"], G["Stasis Web"])]),
           "Black Hole", "stasisWebStrengthMultiplier"))
# Cataclysmic Variable
WH.append((wh("cataclysmic_armor_repair", "Armor Repair Amount Penalty (Cataclysmic Variable)", "Minimum", True,
              locationSkill=[(A["armorDamageAmount"], S["Repair Systems"]),
                             (A["armorDamageAmount"], S["Capital Repair Systems"])]),
           "Cataclysmic Variable", "armorDamageAmountMultiplier"))
WH.append((wh("cataclysmic_shield_boost", "Shield Boost Amount Penalty (Cataclysmic Variable)", "Minimum", True,
              locationSkill=[(A["shieldBonus"], S["Shield Operation"]),
                             (A["shieldBonus"], S["Capital Shield Operation"])]),
           "Cataclysmic Variable", "shieldBonusMultiplier"))
WH.append((wh("cataclysmic_remote_shield", "Remote Shield Repair Amount Bonus (Cataclysmic Variable)", "Maximum", True,
              locationSkill=[(A["shieldBonus"], S["Shield Emission Systems"]),
                             (A["shieldBonus"], S["Capital Shield Emission Systems"])]),
           "Cataclysmic Variable", "shieldBonusMultiplierRemote"))
WH.append((wh("cataclysmic_remote_armor", "Remote Armor Repair Amount Bonus (Cataclysmic Variable)", "Maximum", True,
              locationSkill=[(A["armorDamageAmount"], S["Remote Armor Repair Systems"]),
                             (A["armorDamageAmount"], S["Capital Remote Armor Repair Systems"])]),
           "Cataclysmic Variable", "armorDamageAmountMultiplierRemote"))
WH.append((wh("cataclysmic_cap_capacity", "Capacitor Capacity Bonus (Cataclysmic Variable)", "Maximum", False,
              item=[A["capacitorCapacity"]]), "Cataclysmic Variable", "capacitorCapacityMultiplierSystem"))
WH.append((wh("cataclysmic_cap_recharge", "Capacitor Recharge Time Penalty (Cataclysmic Variable)", "Maximum", False,
              item=[A["rechargeRate"]]), "Cataclysmic Variable", "rechargeRateMultiplier"))
WH.append((wh("cataclysmic_remote_cap", "Remote Capacitor Transmitter Amount Penalty (Cataclysmic Variable)", "Minimum", True,
              locationGroup=[(A["powerTransferAmount"], G["Remote Capacitor Transmitter"])]),
           "Cataclysmic Variable", "energyTransferAmountBonus"))
# Magnetar
WH.append((wh("magnetar_target_range", "Targeting Range Penalty (Magnetar)", "Minimum", True,
              item=[A["maxTargetRange"]]), "Magnetar", "maxTargetRangeMultiplier"))
WH.append((wh("magnetar_tracking", "Tracking Speed Penalty (Magnetar)", "Minimum", True,
              locationSkill=[(A["trackingSpeed"], S["Gunnery"]),
                             (A["trackingSpeed"], S["Drones"])]), "Magnetar", "trackingSpeedMultiplier"))
WH.append((wh("magnetar_damage", "Damage Bonus (Magnetar)", "Maximum", True,
              locationSkill=[(A["damageMultiplier"], S["Gunnery"]),
                             (A["damageMultiplier"], S["Drones"]),
                             (A["damageMultiplier"], S["Fighters"]),
                             (A["damageMultiplier"], S["Vorton Projector Operation"])],
              chargeSkill=[(A["emDamage"], MLO), (A["thermalDamage"], MLO),
                           (A["kineticDamage"], MLO), (A["explosiveDamage"], MLO)]),
           "Magnetar", "damageMultiplierMultiplier"))
WH.append((wh("magnetar_explosion_radius", "Missile Explosion Radius Penalty (Magnetar)", "Maximum", True,
              chargeSkill=[(A["aoeCloudSize"], MLO)],
              locationSkill=[(A["aoeCloudSize"], S["Vorton Projector Operation"])]),
           "Magnetar", "aoeCloudSizeMultiplier"))
WH.append((wh("magnetar_target_painter", "Target Painter Strength Penalty (Magnetar)", "Minimum", True,
              locationSkill=[(A["signatureRadiusBonus"], S["Target Painting"])]),
           "Magnetar", "targetPainterStrengthMultiplier"))
# Red Giant
WH.append((wh("redgiant_heat_damage", "Module Heat Damage Penalty (Red Giant)", "Maximum", False,
              location=[A["heatDamage"]]), "Red Giant", "heatDamageMultiplier"))
WH.append((wh("redgiant_overload", "Overload Bonus (Red Giant)", "Maximum", False,
              location=[A["overloadArmorDamageAmount"], A["overloadDamageModifier"],
                        A["overloadDurationBonus"], A["overloadECCMStrenghtBonus"],
                        A["overloadECMStrengthBonus"], A["overloadHardeningBonus"],
                        A["overloadRangeBonus"], A["overloadRofBonus"],
                        A["overloadSelfDurationBonus"], A["overloadShieldBonus"],
                        A["overloadSpeedFactorBonus"]]), "Red Giant", "overloadBonusMultiplier"))
WH.append((wh("redgiant_smartbomb_range", "Smart Bomb Range Bonus (Red Giant)", "Maximum", False,
              locationGroup=[(A["empFieldRange"], G["Smart Bomb"])]), "Red Giant", "empFieldRangeMultiplier"))
WH.append((wh("redgiant_smartbomb_damage", "Smart Bomb Damage Bonus (Red Giant)", "Maximum", False,
              locationGroup=[(A["emDamage"], G["Smart Bomb"]),
                             (A["thermalDamage"], G["Smart Bomb"]),
                             (A["kineticDamage"], G["Smart Bomb"]),
                             (A["explosiveDamage"], G["Smart Bomb"])]), "Red Giant", "smartbombDamageMultiplier"))
WH.append((wh("redgiant_bomb_damage", "Bomb Damage Bonus (Red Giant)", "Maximum", True,
              chargeSkill=[(A["explosiveDamage"], S["Bomb Deployment"]),
                           (A["kineticDamage"], S["Bomb Deployment"]),
                           (A["thermalDamage"], S["Bomb Deployment"]),
                           (A["emDamage"], S["Bomb Deployment"]),
                           (A["energyNeutralizerAmount"], S["Bomb Deployment"]),
                           (A["scanGravimetricStrengthBonus"], S["Bomb Deployment"]),
                           (A["scanLadarStrengthBonus"], S["Bomb Deployment"]),
                           (A["scanMagnetometricStrengthBonus"], S["Bomb Deployment"]),
                           (A["scanRadarStrengthBonus"], S["Bomb Deployment"])]),
           "Red Giant", "smartbombDamageMultiplier"))
# Wolf Rayet
WH.append((wh("wolfrayet_armor_hp", "Armor Hitpoint Bonus (Wolf Rayet)", "Maximum", False,
              item=[A["armorHP"]]), "Wolf Rayet", "armorHPMultiplier"))
WH.append((wh("wolfrayet_sig_radius", "Signature Radius Bonus (Wolf Rayet)", "Minimum", True,
              item=[A["signatureRadius"]]), "Wolf Rayet", "signatureRadiusMultiplier"))
WH.append((wh("wolfrayet_shield_res", "Shield Resistance Penalty (Wolf Rayet)", "Maximum", True, "PostPercent",
              item=[A["shieldEmDamageResonance"], A["shieldThermalDamageResonance"],
                    A["shieldKineticDamageResonance"], A["shieldExplosiveDamageResonance"]]),
           "Wolf Rayet", "shieldEmDamageResistanceBonus"))
WH.append((wh("wolfrayet_small_weapon", "Small Weapon Damage Bonus (Wolf Rayet)", "Maximum", True,
              locationSkill=[(A["damageMultiplier"], S["Small Energy Turret"]),
                             (A["damageMultiplier"], S["Small Projectile Turret"]),
                             (A["damageMultiplier"], S["Small Hybrid Turret"]),
                             (A["damageMultiplier"], S["Small Precursor Weapon"]),
                             (A["damageMultiplier"], S["Small Vorton Projector"])],
              chargeSkill=[(A["emDamage"], S["Rockets"]),
                           (A["thermalDamage"], S["Rockets"]),
                           (A["kineticDamage"], S["Rockets"]),
                           (A["explosiveDamage"], S["Rockets"]),
                           (A["emDamage"], S["Light Missiles"]),
                           (A["thermalDamage"], S["Light Missiles"]),
                           (A["kineticDamage"], S["Light Missiles"]),
                           (A["explosiveDamage"], S["Light Missiles"])]),
           "Wolf Rayet", "smallWeaponDamageMultiplier"))

WH_FIRST_ID = -2001
WH_IDS = {b["key"]: WH_FIRST_ID - i for i, (b, _, _) in enumerate(WH)}

# --- Storms: id -2101..-2121; preset value = beacon attr per storm strength
STORM_FIRST_ID = -2101
STORM = []  # (defn, beacon_attr_name)

def storm(*args, **kwargs):
    beacon_attr = kwargs.pop("beacon_attr")
    b = buff(*args, **kwargs)
    AUTHORED.append(b)
    STORM.append((b, beacon_attr))
    return b

# Electrical storm (strong 56057 / weak 56064)
storm("storm_electrical_em_res", "EM Resistance Penalty (Electrical Storm)", "Maximum", True, "PostPercent",
      item=[HULL_EM, A["armorEmDamageResonance"], A["shieldEmDamageResonance"]],
      beacon_attr="emDamageResistanceBonus")
storm("storm_electrical_cap_recharge", "Capacitor Recharge Time Bonus (Electrical Storm)", "Minimum", False,
      item=[A["rechargeRate"]], beacon_attr="rechargeRateMultiplier")
# Exotic matter storm (strong 56059 / weak 56058)
storm("storm_exotic_kin_res", "Kinetic Resistance Penalty (Exotic Matter Storm)", "Maximum", True, "PostPercent",
      item=[HULL_KIN, A["armorKineticDamageResonance"], A["shieldKineticDamageResonance"]],
      beacon_attr="kineticDamageResistanceBonus")
storm("storm_exotic_armor_duration", "Armor Repair Duration Bonus (Exotic Matter Storm)", "Minimum", False, "PostPercent",
      locationSkill=[(DURATION, S["Repair Systems"]), (DURATION, S["Capital Repair Systems"])],
      beacon_attr="armorRepairDurationBonus")
storm("storm_exotic_shield_duration", "Shield Booster Duration Bonus (Exotic Matter Storm)", "Minimum", False, "PostPercent",
      locationSkill=[(DURATION, S["Shield Operation"]), (DURATION, S["Capital Shield Operation"])],
      beacon_attr="shieldBoosterDurationBonus")
storm("storm_exotic_mining_duration", "Mining Duration Bonus (Exotic Matter Storm)", "Minimum", False, "PostPercent",
      locationSkill=[(DURATION, MINING)], beacon_attr="miningDurationMultiplier")
storm("storm_exotic_warp_speed", "Warp Speed Bonus (Exotic Matter Storm)", "Maximum", True, "PostPercent",
      item=[WARP_SPEED], beacon_attr="warpSpeedBonus")
storm("storm_exotic_scan_res", "Scan Resolution Bonus (Exotic Matter Storm)", "Maximum", True, "PostPercent",
      item=[SCAN_RES], beacon_attr="scanResolutionBonus")
# Gamma ray storm (strong 56061 / weak 56060)
storm("storm_gamma_exp_res", "Explosive Resistance Penalty (Gamma Ray Storm)", "Maximum", True, "PostPercent",
      item=[HULL_EXP, A["armorExplosiveDamageResonance"], A["shieldExplosiveDamageResonance"]],
      beacon_attr="explosiveDamageResistanceBonus")
storm("storm_gamma_remote_shield", "Remote Shield Repair Amount Penalty (Gamma Ray Storm)", "Minimum", True,
      locationSkill=[(A["shieldBonus"], S["Shield Emission Systems"]),
                     (A["shieldBonus"], S["Capital Shield Emission Systems"])],
      beacon_attr="shieldBonusMultiplierRemote")
storm("storm_gamma_remote_armor", "Remote Armor Repair Amount Penalty (Gamma Ray Storm)", "Minimum", True,
      locationSkill=[(A["armorDamageAmount"], S["Remote Armor Repair Systems"]),
                     (A["armorDamageAmount"], S["Capital Remote Armor Repair Systems"])],
      beacon_attr="armorDamageAmountMultiplierRemote")
storm("storm_gamma_shield_hp", "Shield Hitpoint Bonus (Gamma Ray Storm)", "Maximum", False,
      item=[A["shieldCapacity"]], beacon_attr="shieldCapacityMultiplier")
storm("storm_gamma_cap_capacity", "Capacitor Capacity Bonus (Gamma Ray Storm)", "Maximum", False,
      item=[A["capacitorCapacity"]], beacon_attr="capacitorCapacityMultiplierSystem")
storm("storm_gamma_sig_radius", "Signature Radius Bonus (Gamma Ray Storm)", "Minimum", True,
      item=[A["signatureRadius"]], beacon_attr="signatureRadiusMultiplier")
# Plasma firestorm (strong 56063 / weak 56062)
storm("storm_plasma_therm_res", "Thermal Resistance Penalty (Plasma Firestorm)", "Maximum", True, "PostPercent",
      item=[HULL_THERM, A["armorThermalDamageResonance"], A["shieldThermalDamageResonance"]],
      beacon_attr="thermalDamageResistanceBonus")
storm("storm_plasma_damage", "Damage Bonus (Plasma Firestorm)", "Maximum", True,
      locationSkill=[(A["damageMultiplier"], S["Gunnery"]),
                     (A["damageMultiplier"], S["Drones"]),
                     (A["damageMultiplier"], S["Fighters"])],
      chargeSkill=[(A["emDamage"], MLO), (A["thermalDamage"], MLO),
                   (A["kineticDamage"], MLO), (A["explosiveDamage"], MLO)],
      beacon_attr="damageMultiplierMultiplier")
storm("storm_plasma_tracking", "Tracking Speed Penalty (Plasma Firestorm)", "Minimum", True,
      locationSkill=[(A["trackingSpeed"], S["Gunnery"]),
                     (A["trackingSpeed"], S["Drones"])],
      beacon_attr="trackingSpeedMultiplier")
storm("storm_plasma_explosion_radius", "Explosion Radius Penalty (Plasma Firestorm)", "Maximum", True,
      chargeSkill=[(A["aoeCloudSize"], MLO)],
      locationSkill=[(FA_AM_EXP_RADIUS, S["Fighters"]), (FA_M_EXP_RADIUS, S["Fighters"])],
      beacon_attr="aoeCloudSizeMultiplier")
storm("storm_plasma_armor_hp", "Armor Hitpoint Bonus (Plasma Firestorm)", "Maximum", False,
      item=[A["armorHP"]], beacon_attr="armorHPMultiplier")
# Volatile ice storm (strong 56967 / weak 56997)
storm("storm_icestorm_therm_res", "Thermal Resistance Bonus (Volatile Ice Storm)", "Minimum", True, "PostPercent",
      item=[HULL_THERM, A["armorThermalDamageResonance"], A["shieldThermalDamageResonance"]],
      beacon_attr="thermalDamageResistanceBonus")
storm("storm_icestorm_heat_damage", "Module Heat Damage Bonus (Volatile Ice Storm)", "Minimum", False, "PostPercent",
      location=[A["heatDamage"]], beacon_attr="thermodynamicsHeatDamage")

STORM_IDS = {b["key"]: STORM_FIRST_ID - i for i, (b, _) in enumerate(STORM)}

# ---------------------------------------------------------------------------
# Verify the resolution rules against the extracted beacon data.
# ---------------------------------------------------------------------------

def beacon_attrs(type_id):
    attrs = BEACON_VALUES.get(str(type_id))
    if attrs is None:
        raise SystemExit(f"beacon {type_id} missing from extracted data")
    return attrs


for b, wtype, attr_name in WH:
    attr_id = str(A[attr_name])
    for cls, beacon in WH_CLASSES[wtype].items():
        if attr_id not in beacon_attrs(beacon):
            raise SystemExit(
                f"{b['key']}: attribute {attr_name} ({attr_id}) missing on "
                f"{wtype} class-{cls} beacon {beacon}")

for b, attr_name in STORM:
    attr_id = str(A[attr_name])
    stype = b["key"].split("_")[1]
    for strength, beacon in STORM_BEACONS[stype].items():
        if attr_id not in beacon_attrs(beacon):
            raise SystemExit(
                f"{b['key']}: attribute {attr_name} ({attr_id}) missing on "
                f"{stype} {strength} beacon {beacon}")

WARFARE_PAIRS = [("warfareBuff1ID", "warfareBuff1Value"), ("warfareBuff2ID", "warfareBuff2Value"),
                 ("warfareBuff3ID", "warfareBuff3Value"), ("warfareBuff4ID", "warfareBuff4Value")]
EXPECTED_WARFARE_BUFFS = {
    "weather_darkness": {97, 98},
    "weather_electrical": {90, 92},
    "weather_caustic": {99, 100},
    "weather_exotic": {93, 94},
    "weather_infernal": {95, 96},
    "hazard_bioluminescence": {79},
    "hazard_caustic": {80, 81},
    "hazard_filament_small": {88, 89},
    "hazard_filament_medium": {88, 89},
    "hazard_filament_large": {88, 89},
}


def warfare_buff_ids(type_id):
    attrs = beacon_attrs(type_id)
    ids = []
    for id_name, _ in WARFARE_PAIRS:
        buff_id = int(attrs.get(str(A[id_name]), 0))
        if buff_id:
            ids.append(buff_id)
    return ids


for key, expected in EXPECTED_WARFARE_BUFFS.items():
    beacons = (WEATHER_BEACONS.get(key) or {0: HAZARD_BEACONS[key]})
    for label, beacon in beacons.items():
        found = set(warfare_buff_ids(beacon))
        if found != expected:
            raise SystemExit(
                f"{key} (beacon {beacon}): warfare buffs {sorted(found)} != "
                f"expected {sorted(expected)}")

# ---------------------------------------------------------------------------
# 1. Emit the yaml patch
# ---------------------------------------------------------------------------

MOD_KEY = {
    "item": "itemModifiers",
    "location": "locationModifiers",
    "locationGroup": "locationGroupModifiers",
    "locationSkill": "locationRequiredSkillModifiers",
    "chargeSkill": "chargeRequiredSkillModifiers",
}


def yaml_entry(buff_id, b):
    entry = {
        "id": buff_id,
        "aggregateMode": b["aggregate"],
        "developerDescription": f"System-wide effect - {b['en']}",
        "displayName": b["en"],
        "penalized": b["penalized"],
        "operationName": b["op"],
        "showOutputValueInUI": "ShowNormal",
    }
    for mod_kind, yaml_key in MOD_KEY.items():
        values = b["mods"].get(mod_kind)
        if not values:
            continue
        out = []
        for v in values:
            if mod_kind in ("item", "location"):
                out.append({"dogmaAttributeID": v})
            elif mod_kind == "locationGroup":
                out.append({"dogmaAttributeID": v[0], "groupID": v[1]})
            else:
                out.append({"dogmaAttributeID": v[0], "skillID": v[1]})
        entry[yaml_key] = out
    return entry


patch_buffs = [yaml_entry(WH_IDS[b["key"]], b) for b, _, _ in WH]
patch_buffs += [yaml_entry(STORM_IDS[b["key"]], b) for b, _ in STORM]

doc = {
    "description": """\
  Patch in system-wide effect buffs: wormhole system effects (Black Hole,
  Cataclysmic Variable, Magnetar, Pulsar, Red Giant, Wolf Rayet; classes
  1-6) and metaliminal/volatile storms (Electrical, Exotic Matter, Gamma
  Ray, Plasma Firestorm, Volatile Ice; weak/strong).

  In EVE Online these effects are delivered as System-category dogma effects
  on in-space "effect beacon" items. This patch models them as buff
  collections, mirroring how the client delivers other system-wide effects
  (weather, sovereignty upgrades) via dbuffcollections. The buff strength
  per wormhole class / storm strength is supplied by the fit input,
  resolved from the active snapshot's beacon dogma at runtime
  (tool/system_effects/extract_wormhole.py).

  `chargeRequiredSkillModifiers` and `penalized: false` are hand-authored
  extensions: charge modifiers cover effects that target charge attributes
  (missile/bomb damage, velocity, explosion radius/velocity), and
  `penalized: false` marks effects that are not stacking-penalized in-game
  (hull HP, capacitor, heat damage, overload, smart bomb and rep duration
  bonuses).
""",
    "buffs": patch_buffs,
}

with open(REPO_ROOT / "data/patches/system_effects.yaml", "w", encoding="utf-8") as fp:
    yaml.dump(doc, fp, allow_unicode=True, indent=2, sort_keys=False, width=120)

# ---------------------------------------------------------------------------
# 2. Emit the catalog resolution rules for gen_dart_catalog.py
# ---------------------------------------------------------------------------

wormholes = {}
for b, wtype, attr_name in WH:
    entry = wormholes.setdefault(wtype, {"beacons": WH_CLASSES[wtype], "buff_attrs": {}})
    entry["buff_attrs"][str(WH_IDS[b["key"]])] = A[attr_name]

storms = {}
for b, attr_name in STORM:
    stype = b["key"].split("_")[1]
    entry = storms.setdefault(stype, {"beacons": STORM_BEACONS[stype], "buff_attrs": {}})
    entry["buff_attrs"][str(STORM_IDS[b["key"]])] = A[attr_name]

warfare = {}
for key, beacons in WEATHER_BEACONS.items():
    tier1 = beacons.get(1) or beacons["1"]
    warfare[key] = {"beacons": beacons, "buffs": warfare_buff_ids(tier1)}
for key, beacon in HAZARD_BEACONS.items():
    warfare[key] = {"beacons": {0: beacon}, "buffs": warfare_buff_ids(beacon)}

# Accepted-risk hardcodes: these beacons carry no usable attributes on ANY
# server (pyfa hardcodes them too).
static = {
    # Sovereignty upgrades (iHub)
    "sov_gamma": [[2433, 5.0], [2434, 10.0], [2441, 5.0]],
    "sov_plasma": [[2442, 5.0], [2435, 5.0], [2436, 10.0]],
    "sov_electric": [[2437, -25.0], [2438, 25.0]],
    "sov_exotic": [[2440, 2.0], [2439, 25.0]],
    # Triglavian
    "trig_pochven": [[2534, -50.0], [2535, -30.0], [2538, 30.0], [2539, 30.0]],
    "trig_minor_victory": [[2534, -50.0], [2538, 15.0], [2539, 15.0]],
    # Pirate insurgency suppression
    "insurgency_suppression": [[2405, 10.0]],
}

json.dump(
    {"wormholes": wormholes, "storms": storms, "warfare": warfare, "static": static,
     "wh_buff_names": {str(WH_IDS[b["key"]]): b["en"] for b, _, _ in WH},
     "storm_buff_names": {str(STORM_IDS[b["key"]]): b["en"] for b, _ in STORM}},
    open(REPO_ROOT / "tool/system_effects/out/environment_catalog.json", "w"), indent=1)

print(f"patch: {len(patch_buffs)} buffs "
      f"(wormhole {len(WH)}, storm {len(STORM)}); catalog rules written")
