#!/usr/bin/env python3
"""Generate the Dart system-effect catalog from tool/system_effects/out/environment_catalog.json.

Usage: gen_dart_catalog.py --output <path to system_effect_catalog.dart>

The catalog carries no buff values: wormhole/storm presets reference their
effect beacon plus a beacon-attribute -> buff mapping, abyssal weather and
hazard presets expand the beacon's warfareBuff1-4 pairs at runtime, and only
the sovereignty/Triglavian/insurgency presets (whose beacons carry no usable
dogma on any server) keep pre-expanded static entries.
"""

import argparse
import json
import subprocess
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--output", type=Path, required=True,
                    help="path of the generated system_effect_catalog.dart")
args = parser.parse_args()

CAT = json.load(open(REPO_ROOT / "tool/system_effects/out/environment_catalog.json"))

WH_ZH = {
    "Black Hole": "黑洞",
    "Cataclysmic Variable": "激变变星",
    "Magnetar": "磁星",
    "Pulsar": "脉冲星",
    "Red Giant": "红巨星",
    "Wolf Rayet": "沃尔夫-拉叶星",
}
ZH_CLASS = {1: "一类", 2: "二类", 3: "三类", 4: "四类", 5: "五类", 6: "六类"}

WEATHER = [
    ("weather_electrical", "电子风暴", "Electrical Storm"),
    ("weather_exotic", "异种粒子风暴", "Exotic Particle Storm"),
    ("weather_infernal", "等离子火焰风暴", "Plasma Firestorm"),
    ("weather_darkness", "暗物质场", "Dark Matter Field"),
    ("weather_caustic", "伽玛射线余波", "Gamma-Ray Afterglow"),
]

STORM_TYPES = [
    ("electrical", "超阈限电子风暴", "Metaliminal Electrical Storm"),
    ("exotic", "超阈限异种物质风暴", "Metaliminal Exotic Matter Storm"),
    ("gamma", "超阈限伽玛射线风暴", "Metaliminal Gamma Ray Storm"),
    ("plasma", "超阈限等离子火焰风暴", "Metaliminal Plasma Firestorm"),
    ("icestorm", "不稳定冰风暴", "Volatile Ice Storm"),
]

BUFF_ZH = {
    # wormhole buffs
    "Shield Hitpoint Bonus (Pulsar)": "护盾值加成（脉冲星）",
    "Signature Radius Penalty (Pulsar)": "信号半径惩罚（脉冲星）",
    "Armor Resistance Penalty (Pulsar)": "装甲抗性惩罚（脉冲星）",
    "Capacitor Recharge Time Bonus (Pulsar)": "电容回充时间加成（脉冲星）",
    "Energy Warfare Strength Bonus (Pulsar)": "能量战强度加成（脉冲星）",
    "Agility Penalty (Black Hole)": "惯性惩罚（黑洞）",
    "Targeting Range Bonus (Black Hole)": "锁定距离加成（黑洞）",
    "Missile Velocity Bonus (Black Hole)": "导弹速度加成（黑洞）",
    "Maximum Velocity Bonus (Black Hole)": "最大速度加成（黑洞）",
    "Missile Explosion Velocity Bonus (Black Hole)": "导弹爆炸速度加成（黑洞）",
    "Stasis Webifier Strength Penalty (Black Hole)": "停滞缠绕强度惩罚（黑洞）",
    "Armor Repair Amount Penalty (Cataclysmic Variable)": "装甲维修量惩罚（激变变星）",
    "Shield Boost Amount Penalty (Cataclysmic Variable)": "护盾回充量惩罚（激变变星）",
    "Remote Shield Repair Amount Bonus (Cataclysmic Variable)": "远程护盾维修量加成（激变变星）",
    "Remote Armor Repair Amount Bonus (Cataclysmic Variable)": "远程装甲维修量加成（激变变星）",
    "Capacitor Capacity Bonus (Cataclysmic Variable)": "电容容量加成（激变变星）",
    "Capacitor Recharge Time Penalty (Cataclysmic Variable)": "电容回充时间惩罚（激变变星）",
    "Remote Capacitor Transmitter Amount Penalty (Cataclysmic Variable)": "远程电容传输量惩罚（激变变星）",
    "Targeting Range Penalty (Magnetar)": "锁定距离惩罚（磁星）",
    "Tracking Speed Penalty (Magnetar)": "跟踪速度惩罚（磁星）",
    "Damage Bonus (Magnetar)": "伤害加成（磁星）",
    "Missile Explosion Radius Penalty (Magnetar)": "导弹爆炸半径惩罚（磁星）",
    "Target Painter Strength Penalty (Magnetar)": "目标标记强度惩罚（磁星）",
    "Module Heat Damage Penalty (Red Giant)": "装备热量伤害惩罚（红巨星）",
    "Overload Bonus (Red Giant)": "超载加成（红巨星）",
    "Smart Bomb Range Bonus (Red Giant)": "立体炸弹范围加成（红巨星）",
    "Smart Bomb Damage Bonus (Red Giant)": "立体炸弹伤害加成（红巨星）",
    "Bomb Damage Bonus (Red Giant)": "炸弹伤害加成（红巨星）",
    "Armor Hitpoint Bonus (Wolf Rayet)": "装甲值加成（沃尔夫-拉叶星）",
    "Signature Radius Bonus (Wolf Rayet)": "信号半径加成（沃尔夫-拉叶星）",
    "Shield Resistance Penalty (Wolf Rayet)": "护盾抗性惩罚（沃尔夫-拉叶星）",
    "Small Weapon Damage Bonus (Wolf Rayet)": "小型武器伤害加成（沃尔夫-拉叶星）",
    # storm buffs
    "EM Resistance Penalty (Electrical Storm)": "电磁抗性惩罚（电子风暴）",
    "Capacitor Recharge Time Bonus (Electrical Storm)": "电容回充时间加成（电子风暴）",
    "Kinetic Resistance Penalty (Exotic Matter Storm)": "动能抗性惩罚（异种物质风暴）",
    "Armor Repair Duration Bonus (Exotic Matter Storm)": "装甲维修周期加成（异种物质风暴）",
    "Shield Booster Duration Bonus (Exotic Matter Storm)": "护盾回充周期加成（异种物质风暴）",
    "Mining Duration Bonus (Exotic Matter Storm)": "采矿周期加成（异种物质风暴）",
    "Warp Speed Bonus (Exotic Matter Storm)": "跃迁速度加成（异种物质风暴）",
    "Scan Resolution Bonus (Exotic Matter Storm)": "扫描分辨率加成（异种物质风暴）",
    "Explosive Resistance Penalty (Gamma Ray Storm)": "爆炸抗性惩罚（伽玛射线风暴）",
    "Remote Shield Repair Amount Penalty (Gamma Ray Storm)": "远程护盾维修量惩罚（伽玛射线风暴）",
    "Remote Armor Repair Amount Penalty (Gamma Ray Storm)": "远程装甲维修量惩罚（伽玛射线风暴）",
    "Shield Hitpoint Bonus (Gamma Ray Storm)": "护盾值加成（伽玛射线风暴）",
    "Capacitor Capacity Bonus (Gamma Ray Storm)": "电容容量加成（伽玛射线风暴）",
    "Signature Radius Bonus (Gamma Ray Storm)": "信号半径加成（伽玛射线风暴）",
    "Thermal Resistance Penalty (Plasma Firestorm)": "热能抗性惩罚（等离子火焰风暴）",
    "Damage Bonus (Plasma Firestorm)": "伤害加成（等离子火焰风暴）",
    "Tracking Speed Penalty (Plasma Firestorm)": "跟踪速度惩罚（等离子火焰风暴）",
    "Explosion Radius Penalty (Plasma Firestorm)": "爆炸半径惩罚（等离子火焰风暴）",
    "Armor Hitpoint Bonus (Plasma Firestorm)": "装甲值加成（等离子火焰风暴）",
    "Thermal Resistance Bonus (Volatile Ice Storm)": "热能抗性加成（不稳定冰风暴）",
    "Module Heat Damage Bonus (Volatile Ice Storm)": "装备热量伤害加成（不稳定冰风暴）",
}

NATIVE_BUFF_NAMES = {
    90: ("电磁抗性惩罚", "EM Resistance Penalty"),
    92: ("电容回充加成", "Capacitor Recharge Bonus"),
    93: ("爆炸抗性惩罚", "Explosive Resistance Penalty"),
    94: ("护盾值加成", "Shield HP Bonus"),
    95: ("热能抗性惩罚", "Thermal Resistance Penalty"),
    96: ("装甲值加成", "Armor HP Bonus"),
    97: ("炮台射程惩罚", "Turret Optimal and Falloff Range Penalty"),
    98: ("速度加成", "Velocity Bonus"),
    99: ("动能抗性惩罚", "Kinetic Resistance Penalty"),
    100: ("扫描分辨率加成", "Scan Resolution Bonus"),
    2433: ("护盾值加成", "Shield Hitpoint Bonus"),
    2434: ("电容容量加成", "Capacitor Capacity Bonus"),
    2441: ("本地护盾回充加成", "Local Shield Booster Bonus"),
    2442: ("本地装甲维修加成", "Local Armor Repairer Bonus"),
    2435: ("装甲值加成", "Armor Hitpoint Bonus"),
    2436: ("装备超载加成", "Module Overload Bonus"),
    2437: ("电容回充加成", "Capacitor Recharge Bonus"),
    2438: ("锁定与定向扫描距离加成", "Targeting and DScan Range Bonus"),
    2440: ("基础跃迁速度增加", "Base Warp Speed Addition"),
    2439: ("扫描分辨率加成", "Scan Resolution Bonus"),
    2534: ("最大锁定数惩罚", "Maximum Locked Targets Penalty"),
    2535: ("跃迁速度惩罚", "Warp Speed Penalty"),
    2538: ("远程装甲维修加成", "Remote Armor Repair Bonus"),
    2539: ("远程护盾维修加成", "Remote Shield Boost Bonus"),
    2405: ("停滞缠绕与跃迁扰断距离加成", "Webifier and Scrambler Range Bonus (Anti-Pirate)"),
    79: ("信号半径惩罚", "Signature Radius Penalty"),
    80: ("惯性加成", "Inertia Bonus"),
    81: ("最大速度加成", "Max Velocity Bonus"),
    88: ("护盾回充量惩罚", "Shield Booster Amount Penalty"),
    89: ("护盾回充周期加成", "Shield Booster Duration Bonus"),
}

SOV = [
    ("sov_gamma", "伽玛", "Gamma"),
    ("sov_plasma", "等离子", "Plasma"),
    ("sov_electric", "电子", "Electric"),
    ("sov_exotic", "异种", "Exotic"),
]

STATIC_MISC = [
    ("trig_pochven", "trig.pochven", "triglavian", "波赫文（最终林隙）", "Pochven (Final Liminality)"),
    ("trig_minor_victory", "trig.minor_victory", "triglavian", "三神裔局部胜利", "Triglavian Minor Victory"),
    ("insurgency_suppression", "insurgency.suppression", "insurgency", "叛乱镇压", "Insurgency Suppression"),
]

HAZARDS = [
    ("hazard_bioluminescence", "生物荧光云", "Bioluminescence Cloud"),
    ("hazard_caustic", "腐蚀云", "Caustic Cloud"),
    ("hazard_filament_small", "纤维云（小）", "Filament Cloud (Small)"),
    ("hazard_filament_medium", "纤维云（中）", "Filament Cloud (Medium)"),
    ("hazard_filament_large", "纤维云（大）", "Filament Cloud (Large)"),
]


def fmt_num(v):
    return f"{v:g}"


def preset_head(pid, category, zh, en):
    return [f'  SystemEffectPreset(id: "{pid}", category: SystemEffectCategory.{category},',
            f'      zh: "{zh}", en: "{en}",']


out = []
out.append("// GENERATED — see packages/eve-fit-os/data/patches/system_effects.yaml for the")
out.append("// hand-authored buff definitions and the generation tooling under")
out.append("// packages/eve-fit-os/tool/system_effects/. Regenerate instead of hand-editing.")
out.append("//")
out.append("// The catalog carries no buff values: presets describe how to resolve values")
out.append("// from the active snapshot's effect-beacon dogma at runtime; see")
out.append("// system_effect_resolver.dart (hand-written) for the expansion logic.")
out.append("")
out.append('import "package:eve_fit_assistant/storage/fit/schema.dart";')
out.append("")
out.append("/// Category of a system effect preset, driving the picker grouping.")
out.append("enum SystemEffectCategory {")
out.append('  wormhole("虫洞", "Wormhole"),')
out.append('  abyssalWeather("深渊天气", "Abyssal Weather"),')
out.append('  storm("风暴", "Storm"),')
out.append('  sovUpgrade("主权升级", "Sovereignty Upgrade"),')
out.append('  triglavian("三神裔", "Triglavian"),')
out.append('  insurgency("海盗叛乱", "Pirate Insurgency"),')
out.append('  abyssalHazard("深渊危害", "Abyssal Hazard");')
out.append("")
out.append("  const SystemEffectCategory(this.zh, this.en);")
out.append("")
out.append("  final String zh;")
out.append("  final String en;")
out.append("}")
out.append("")
out.append("/// A system-wide environment preset: bilingual display metadata plus a")
out.append("/// rule for expanding the preset into concrete [FitSystemBuff] entries")
out.append("/// from the active snapshot's effect-beacon dogma.")
out.append("class SystemEffectPreset {")
out.append("  const SystemEffectPreset({")
out.append("    required this.id,")
out.append("    required this.category,")
out.append("    required this.zh,")
out.append("    required this.en,")
out.append("    this.beaconTypeId,")
out.append("    this.buffAttrs,")
out.append("    this.warfareFromBeacon = false,")
out.append("    this.staticBuffs,")
out.append("  });")
out.append("")
out.append("  final String id;")
out.append("  final SystemEffectCategory category;")
out.append("  final String zh;")
out.append("  final String en;")
out.append("")
out.append("  /// Effect beacon type carrying this preset's values in the active")
out.append("  /// snapshot's engine data; null for [staticBuffs] presets.")
out.append("  final int? beaconTypeId;")
out.append("")
out.append("  /// Buff ID -> beacon attribute ID: each buff's value is the beacon's")
out.append("  /// attribute value (wormholes and metaliminal/volatile storms).")
out.append("  final Map<int, int>? buffAttrs;")
out.append("")
out.append("  /// Expand the beacon's `warfareBuff1-4ID/Value` attribute pairs")
out.append("  /// directly (abyssal weather and hazard clouds).")
out.append("  final bool warfareFromBeacon;")
out.append("")
out.append("  /// Pre-expanded entries for effects whose beacons carry no usable")
out.append("  /// dogma on any server (sovereignty upgrades, Triglavian space,")
out.append("  /// insurgency suppression); accepted-risk hardcodes.")
out.append("  final List<FitSystemBuff>? staticBuffs;")
out.append("}")
out.append("")
out.append("/// All environment presets, in picker display order.")
out.append("const systemEffectCatalog = <SystemEffectPreset>[")

for wtype in ["Black Hole", "Cataclysmic Variable", "Magnetar", "Pulsar", "Red Giant", "Wolf Rayet"]:
    slug = wtype.lower().replace(" ", "_")
    rule = CAT["wormholes"][wtype]
    buff_attrs = ", ".join(f"{buff}: {attr}" for buff, attr in rule["buff_attrs"].items())
    for cls in range(1, 7):
        pid = f"wormhole.{slug}.{cls}"
        beacon = rule["beacons"][str(cls)]
        out += preset_head(pid, "wormhole", f"{WH_ZH[wtype]} — {ZH_CLASS[cls]}", f"{wtype} — Class {cls}")
        out.append(f"      beaconTypeId: {beacon}, buffAttrs: {{{buff_attrs}}}),")

for key, zh, en in WEATHER:
    slug = key.split("_", 1)[1]
    beacons = CAT["warfare"][key]["beacons"]
    for tier in (1, 2, 3):
        pid = f"weather.{slug}.{tier}"
        out += preset_head(pid, "abyssalWeather", f"{zh} — {tier} 级", f"{en} — Tier {tier}")
        out.append(f"      beaconTypeId: {beacons[str(tier)]}, warfareFromBeacon: true),")

for key, zh, en in STORM_TYPES:
    rule = CAT["storms"][key]
    buff_attrs = ", ".join(f"{buff}: {attr}" for buff, attr in rule["buff_attrs"].items())
    for strength, strength_zh, strength_en in [("strong", "强", "Strong"), ("weak", "弱", "Weak")]:
        pid = f"storm.{key}.{strength}"
        out += preset_head(pid, "storm", f"{strength_zh}{zh}", f"{strength_en} {en}")
        out.append(f"      beaconTypeId: {rule['beacons'][strength]}, buffAttrs: {{{buff_attrs}}}),")


def emit_static(pid, category, zh, en, entries):
    buffs = ", ".join(
        f'FitSystemBuff(buffId: {b}, value: {fmt_num(v)}, presetId: "{pid}")' for b, v in entries)
    out_head = preset_head(pid, category, zh, en)
    out_head.append(f"      staticBuffs: [{buffs}]),")
    return out_head


for key, zh, en in SOV:
    pid = f"sov.{key.split('_', 1)[1]}"
    out += emit_static(pid, "sovUpgrade", f"{zh}主权升级", f"{en} Sovereignty Upgrade",
                       CAT["static"][key])

for key, pid, category, zh, en in STATIC_MISC:
    out += emit_static(pid, category, zh, en, CAT["static"][key])

for key, zh, en in HAZARDS:
    pid = f"hazard.{key.split('_', 1)[1]}"
    beacon = CAT["warfare"][key]["beacons"]["0"]
    out += preset_head(pid, "abyssalHazard", zh, en)
    out.append(f"      beaconTypeId: {beacon}, warfareFromBeacon: true),")

out.append("];")
out.append("")
out.append("/// Preset lookup by id.")
out.append("final systemEffectPresetById = <String, SystemEffectPreset>{")
out.append("  for (final preset in systemEffectCatalog) preset.id: preset,")
out.append("};")
out.append("")
out.append("/// A pickable buff entry of the custom-buff library.")
out.append("class SystemBuffLibraryEntry {")
out.append("  const SystemBuffLibraryEntry({")
out.append("    required this.buffId,")
out.append("    required this.zh,")
out.append("    required this.en,")
out.append("    this.defaultBeaconTypeId,")
out.append("    this.defaultAttrId,")
out.append("    this.staticDefault,")
out.append("  });")
out.append("")
out.append("  final int buffId;")
out.append("  final String zh;")
out.append("  final String en;")
out.append("")
out.append("  /// Beacon whose dogma provides the pre-filled strength in the")
out.append("  /// active snapshot (weakest tier/class of the effect); the user can")
out.append("  /// edit the value after adding.")
out.append("  final int? defaultBeaconTypeId;")
out.append("")
out.append("  /// Beacon attribute holding the default strength; null means the")
out.append("  /// default comes from the beacon's `warfareBuff1-4` pair for")
out.append("  /// [buffId] (native dbuff environments).")
out.append("  final int? defaultAttrId;")
out.append("")
out.append("  /// Static fallback default for buffs without a resolvable beacon")
out.append("  /// (sovereignty upgrades, Triglavian, insurgency suppression).")
out.append("  final double? staticDefault;")
out.append("}")
out.append("")
out.append("/// Buffs offered for free-form (custom) selection, in display order.")
out.append("const systemBuffLibrary = <SystemBuffLibraryEntry>[")

# Authored buffs: weakest-tier beacon default (wormhole class 1, storm weak).
authored_names = {}
authored_names.update({int(k): v for k, v in CAT["wh_buff_names"].items()})
authored_names.update({int(k): v for k, v in CAT["storm_buff_names"].items()})

authored_sources = {}
for wtype, rule in CAT["wormholes"].items():
    beacon = rule["beacons"]["1"]
    for buff_id, attr in rule["buff_attrs"].items():
        authored_sources[int(buff_id)] = (beacon, int(attr))
for stype, rule in CAT["storms"].items():
    beacon = rule["beacons"]["weak"]
    for buff_id, attr in rule["buff_attrs"].items():
        authored_sources[int(buff_id)] = (beacon, int(attr))

for buff_id in sorted(authored_names, key=lambda b: -b):
    en = authored_names[buff_id]
    zh = BUFF_ZH[en]
    beacon, attr = authored_sources[buff_id]
    out.append(f'  SystemBuffLibraryEntry(buffId: {buff_id}, zh: "{zh}", en: "{en}",')
    out.append(f'      defaultBeaconTypeId: {beacon}, defaultAttrId: {attr}),')

# Native buffs: weather/hazard entries resolve their default from the
# weakest-tier beacon's warfareBuff pairs; sov/trig/insurgency entries fall
# back to a static default (their beacons carry no usable dogma).
seen_native = set()


def emit_native(buff_id, tail):
    if buff_id in seen_native:
        return
    seen_native.add(buff_id)
    zh, en = NATIVE_BUFF_NAMES[buff_id]
    out.append(f'  SystemBuffLibraryEntry(buffId: {buff_id}, zh: "{zh}", en: "{en}",')
    out.append(f"      {tail}),")


for key, _, _ in WEATHER:
    beacon = CAT["warfare"][key]["beacons"]["1"]
    for buff_id in CAT["warfare"][key]["buffs"]:
        emit_native(buff_id, f"defaultBeaconTypeId: {beacon}")

for key, _, _ in SOV + [(k, None, None) for k, _, _, _, _ in STATIC_MISC]:
    for buff_id, value in CAT["static"][key]:
        emit_native(buff_id, f"staticDefault: {fmt_num(value)}")

for key, _, _ in HAZARDS:
    beacon = CAT["warfare"][key]["beacons"]["0"]
    for buff_id in CAT["warfare"][key]["buffs"]:
        emit_native(buff_id, f"defaultBeaconTypeId: {beacon}")

out.append("];")
out.append("")
out.append("/// Custom-buff library lookup by buff id.")
out.append("final systemBuffLibraryById = <int, SystemBuffLibraryEntry>{")
out.append("  for (final entry in systemBuffLibrary) entry.buffId: entry,")
out.append("};")

path = args.output
path.parent.mkdir(parents=True, exist_ok=True)
with open(path, "w", encoding="utf-8") as fp:
    fp.write("\n".join(out) + "\n")

subprocess.run(["dart", "format", str(path)], check=True)

print(f"wrote {path} ({len(out)} lines)")
