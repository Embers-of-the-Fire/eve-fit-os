from os import environ
from pathlib import Path


CONVERT_ROOT = Path(__file__).parent

FSD_BINARY_DIR = Path(environ.get("FSD_BINARY_DIR"))
FSD_LOC_EN_DIR = Path(environ.get("FSD_LOC_EN_DIR"))

DATA_ROOT = CONVERT_ROOT.parent
FSD_PATCH_DIR = DATA_ROOT / "fsd-patches"
PATCH_DIR = DATA_ROOT / "patches"
OUTPUT_DIR = environ.get("OUTPUT_DIR", DATA_ROOT / "out")
