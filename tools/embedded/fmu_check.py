#!/usr/bin/env python3
"""Run an FMU exported by `arclang export -f fmu` with fmpy (its FMI 2.0
co-simulation interface, FMU2Slave) on the inputs of an executed case and
compare its outputs, step by step, with the model-level reference written
beside it.

    python3 tools/embedded/fmu_check.py out/battery_charger.fmu out/cases/TC-CHG-001

At each communication point k * step the case's inputs of step k are set,
fmi2DoStep(k * step, step) is called, and the outputs read at (k + 1) * step
are compared with the reference row there. Exits 1 when a value differs
(tolerance 0 by default)."""
import argparse
import csv
import shutil
import sys

from fmpy import extract, read_model_description
from fmpy.fmi2 import FMU2Slave


def read_csv(path):
    with open(path, newline="") as f:
        rows = list(csv.reader(f))
    return rows[0], [[float(x) for x in r] for r in rows[1:] if r]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("fmu")
    parser.add_argument("case", help="cases/<id>: reads <id>.inputs.csv and <id>.reference.csv")
    parser.add_argument("--tolerance", type=float, default=0.0)
    args = parser.parse_args()
    description = read_model_description(args.fmu)
    variables = {v.name: v for v in description.modelVariables}
    step = float(description.defaultExperiment.stepSize)
    in_names, in_rows = read_csv(args.case + ".inputs.csv")
    ref_names, ref_rows = read_csv(args.case + ".reference.csv")
    outputs = ref_names[1:]
    unzipped = extract(args.fmu)
    fmu = FMU2Slave(guid=description.guid, unzipDirectory=unzipped, modelIdentifier=description.coSimulation.modelIdentifier, instanceName="check")
    fmu.instantiate()
    fmu.setupExperiment(startTime=0.0)
    fmu.enterInitializationMode()
    fmu.exitInitializationMode()
    setters = {"Real": fmu.setReal, "Integer": fmu.setInteger, "Boolean": fmu.setBoolean}
    getters = {"Real": fmu.getReal, "Integer": fmu.getInteger, "Boolean": fmu.getBoolean}
    worst = 0.0
    first = None
    for k, (inputs, expected) in enumerate(zip(in_rows, ref_rows)):
        for name, value in zip(in_names[1:], inputs[1:]):
            v = variables[name]
            cast = {"Real": float, "Integer": int, "Boolean": lambda x: x != 0.0}[v.type]
            setters[v.type]([v.valueReference], [cast(value)])
        fmu.doStep(currentCommunicationPoint=k * step, communicationStepSize=step)
        for name, want in zip(outputs, expected[1:]):
            v = variables[name]
            got = float(getters[v.type]([v.valueReference])[0])
            diff = abs(got - want)
            worst = max(worst, diff)
            if diff > args.tolerance and first is None:
                first = f"at {expected[0]} s, {name}: model {want}, FMU {got}"
    fmu.terminate()
    fmu.freeInstance()
    shutil.rmtree(unzipped, ignore_errors=True)
    print(f"{args.case}: fmpy FMI {description.fmiVersion} co-simulation, {len(ref_rows)} steps x {len(outputs)} outputs, largest difference {worst} (tolerance {args.tolerance})")
    if first:
        print(f"  MISMATCH {first}")
        return 1
    print("  the FMU reproduces the model")
    return 0


if __name__ == "__main__":
    sys.exit(main())
