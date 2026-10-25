"""Read STEP files with Open CASCADE (the kernel of FreeCAD and of many CAD
tools) and report what a CAD tool gets: the solids, whether each is a
valid closed B-rep, their volumes and the bounding box of the assembly.
Exit code 0 when every file transfers and every solid is valid.

Setup (once):
    conda create -n occ -c conda-forge pythonocc-core -y
Usage:
    python tools/step_validate.py model.step [--expect-solids N] [--expect-volume M3 ...]

`--expect-volume` lists the volumes the solids must have, in cubic metres,
in any order (each within 0.1 %).
"""
import argparse
import sys

from OCC.Core.STEPControl import STEPControl_Reader
from OCC.Core.IFSelect import IFSelect_RetDone
from OCC.Core.TopExp import TopExp_Explorer
from OCC.Core.TopAbs import TopAbs_SOLID
from OCC.Core.BRepCheck import BRepCheck_Analyzer
from OCC.Core.GProp import GProp_GProps
from OCC.Core.BRepGProp import brepgprop
from OCC.Core.Bnd import Bnd_Box
from OCC.Core.BRepBndLib import brepbndlib


def check(path, expect_solids, expect_volumes):
    reader = STEPControl_Reader()
    if reader.ReadFile(path) != IFSelect_RetDone:
        return False, f"{path}: not readable as STEP"
    reader.TransferRoots()
    shape = reader.OneShape()
    volumes, problems = [], []
    explorer = TopExp_Explorer(shape, TopAbs_SOLID)
    while explorer.More():
        solid = explorer.Current()
        if not BRepCheck_Analyzer(solid).IsValid():
            problems.append(f"solid {len(volumes) + 1} is not a valid B-rep")
        props = GProp_GProps()
        brepgprop.VolumeProperties(solid, props)
        volumes.append(props.Mass() / 1e9)  # mm3 -> m3
        explorer.Next()
    box = Bnd_Box()
    brepbndlib.Add(shape, box)
    xmin, ymin, zmin, xmax, ymax, zmax = box.Get() if volumes else (0, 0, 0, 0, 0, 0)
    if any(v <= 0 for v in volumes):
        problems.append("a solid has no volume (faces turned inside out)")
    if expect_solids is not None and len(volumes) != expect_solids:
        problems.append(f"{len(volumes)} solids, {expect_solids} expected")
    remaining = sorted(volumes)
    for wanted in sorted(expect_volumes):
        match = next((v for v in remaining if abs(v - wanted) <= wanted * 1e-3), None)
        if match is None:
            problems.append(f"no solid of {wanted:g} m3 (solids: {', '.join(f'{v:.6g}' for v in volumes)})")
        else:
            remaining.remove(match)
    summary = f"{len(volumes)} solid(s), total {sum(volumes):.6g} m3, extent {(xmax - xmin) / 1000:.3f} x {(ymax - ymin) / 1000:.3f} x {(zmax - zmin) / 1000:.3f} m"
    return not problems, f"{path}: {summary}" + "".join(f"\n    {p}" for p in problems)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("files", nargs="+")
    parser.add_argument("--expect-solids", type=int)
    parser.add_argument("--expect-volume", type=float, nargs="*", default=[])
    args = parser.parse_args()
    ok = True
    for path in args.files:
        good, report = check(path, args.expect_solids, args.expect_volume)
        print(("OK   " if good else "FAIL ") + report)
        ok &= good
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
