import argparse
import os
from pathlib import Path

from dotenv import load_dotenv

from . import dbuffcollections


def main() -> None:
    load_dotenv()

    parser = argparse.ArgumentParser(
        prog="patching",
        description="Generate fsd-patches files from EVE static data.",
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    p_buffs = subparsers.add_parser(
        "dbuffcollections",
        help="Convert dbuffcollections.static (SQLite) to the fsd-patches YAML file.",
    )
    p_buffs.add_argument(
        "static_file",
        type=Path,
        help="Path to dbuffcollections.static from the EVE client.",
    )
    p_buffs.add_argument(
        "--localization",
        type=Path,
        default=os.environ.get("FSD_LOC_EN_DIR"),
        help="Path to localization_fsd_en-us.pickle "
        "(defaults to the FSD_LOC_EN_DIR environment variable).",
    )
    p_buffs.add_argument(
        "-o",
        "--output",
        type=Path,
        default=dbuffcollections.DEFAULT_OUTPUT,
        help="Output YAML file (defaults to %(default)s).",
    )

    args = parser.parse_args()

    if args.command == "dbuffcollections":
        localization = (
            Path(args.localization) if args.localization is not None else None
        )
        dbuffcollections.convert(args.static_file, args.output, localization)


if __name__ == "__main__":
    main()
