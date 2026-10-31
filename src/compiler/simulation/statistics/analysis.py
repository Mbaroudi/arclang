"""ArcLang: the samples of a Monte Carlo case, analysed with Python.

Usage: python3 analysis.py samples.csv statistics.json

samples.csv is written by `arclang verify`: one row per sample, the
uncertain parameters in SI units, then one "margin: <expectation>" column per
expectation (positive on the side that holds), then holds (1 or 0).

For each margin: the share of samples that fail, a normal fit of the margin
and the failure probability it extrapolates (beyond what the samples alone
can show), the partial rank correlation of each parameter (PRCC, with a
bootstrap interval), and, with SALib, Borgonovo's delta and first-order
Sobol index estimated from the same samples. The verdict of the case is not
computed here: it is the compiler's.
"""
import csv
import json
import math
import sys

import numpy as np


def ranks(values):
    order = np.argsort(values, kind="mergesort")
    ranked = np.empty(len(values))
    ranked[order] = np.arange(len(values))
    # Ties share the mean of their ranks.
    for value in np.unique(values):
        where = values == value
        if where.sum() > 1:
            ranked[where] = ranked[where].mean()
    return ranked


def prcc(x, y):
    """Partial rank correlation of each column of x with y."""
    rx = np.column_stack([ranks(x[:, j]) for j in range(x.shape[1])])
    ry = ranks(y)
    out = []
    for j in range(x.shape[1]):
        others = np.column_stack([np.ones(len(ry)), np.delete(rx, j, axis=1)])
        res_x = rx[:, j] - others @ np.linalg.lstsq(others, rx[:, j], rcond=None)[0]
        res_y = ry - others @ np.linalg.lstsq(others, ry, rcond=None)[0]
        # When the other parameters already order the margin (or x_j) entirely,
        # what is left is rounding: the correlation is undefined, not a number.
        left_x = (res_x ** 2).sum() / max(((rx[:, j] - rx[:, j].mean()) ** 2).sum(), 1e-300)
        left_y = (res_y ** 2).sum() / max(((ry - ry.mean()) ** 2).sum(), 1e-300)
        if left_x < 1e-9 or left_y < 1e-9:
            out.append(float("nan"))
            continue
        out.append(float((res_x * res_y).sum() / math.sqrt((res_x ** 2).sum() * (res_y ** 2).sum())))
    return out


def normal_tail(margin):
    mean, sd = float(np.mean(margin)), float(np.std(margin, ddof=1))
    if sd == 0:
        return mean, sd, 0.0 if mean >= 0 else 1.0
    return mean, sd, 0.5 * math.erfc(mean / sd / math.sqrt(2))


def main(samples_path, output_path):
    with open(samples_path, newline="") as handle:
        rows = list(csv.reader(handle))
    header, body = rows[0], rows[1:]
    margin_columns = [i for i, name in enumerate(header) if name.startswith("margin: ")]
    inputs = list(range(1, margin_columns[0]))
    names = [header[i] for i in inputs]
    x = np.array([[float(row[i]) for i in inputs] for row in body])
    try:
        from SALib.analyze import delta
        from importlib.metadata import version
        salib = version("SALib")
    except ImportError:
        delta, salib = None, None
    rng = np.random.default_rng(1)
    margins = []
    for column in margin_columns:
        kept = [k for k, row in enumerate(body) if row[column] != ""]
        y = np.array([float(body[k][column]) for k in kept])
        xs = x[kept]
        mean, sd, tail = normal_tail(y)
        influences = []
        # A margin that does not vary (a run ending on its bound) has no influence to explain.
        constant = float(np.ptp(y)) <= 1e-6 * max(float(np.abs(y).max()), 1.0)
        if constant:
            margins.append({"expect": header[column][len("margin: "):], "samples": int(len(y)), "failure_fraction": float((y < 0).mean()), "margin_mean": mean, "margin_sd": sd, "fitted_failure_probability": tail, "constant": True, "influences": []})
            continue
        point = prcc(xs, y)
        boot = []
        for _ in range(200):
            pick = rng.integers(0, len(y), len(y))
            boot.append(prcc(xs[pick], y[pick]))
        boot = np.array(boot)
        for j, name in enumerate(names):
            if math.isnan(point[j]):
                influences.append({"parameter": name, "method": "PRCC", "value": None, "low": None, "high": None})
                continue
            low, high = np.nanpercentile(boot[:, j], [2.5, 97.5])
            influences.append({"parameter": name, "method": "PRCC", "value": round(point[j], 4), "low": round(float(low), 4), "high": round(float(high), 4)})
        if delta is not None and len(y) >= 20:
            bounds = [[float(xs[:, j].min()), float(xs[:, j].max())] for j in range(xs.shape[1])]
            problem = {"num_vars": len(names), "names": names, "bounds": bounds}
            result = delta.analyze(problem, xs, y, print_to_console=False, seed=1)
            for j, name in enumerate(names):
                influences.append({"parameter": name, "method": "delta", "value": round(float(result["delta"][j]), 4), "low": round(float(result["delta"][j] - result["delta_conf"][j]), 4), "high": round(float(result["delta"][j] + result["delta_conf"][j]), 4)})
                influences.append({"parameter": name, "method": "S1", "value": round(float(result["S1"][j]), 4), "low": round(float(result["S1"][j] - result["S1_conf"][j]), 4), "high": round(float(result["S1"][j] + result["S1_conf"][j]), 4)})
        margins.append({
            "expect": header[column][len("margin: "):],
            "samples": int(len(y)),
            "failure_fraction": float((y < 0).mean()),
            "margin_mean": mean,
            "margin_sd": sd,
            "fitted_failure_probability": tail,
            "constant": False,
            "influences": influences,
        })
    engine = f"Python {sys.version.split()[0]}, NumPy {np.__version__}" + (f", SALib {salib}" if salib else " (SALib absent: pip install SALib for delta and S1)")
    with open(output_path, "w") as handle:
        json.dump({"engine": engine, "margins": margins}, handle, indent=2)


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
