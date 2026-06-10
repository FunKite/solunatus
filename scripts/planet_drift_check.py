#!/usr/bin/env python3
"""Validate solunatus planet positions against JPL Horizons.

Runs the solunatus binary in JSON mode for one or more dates, queries the
JPL Horizons API (the gold-standard ephemeris) for the same instants and
observer location, and gates each planet's altitude/azimuth/magnitude/distance
deltas against thresholds.

Used by the scheduled planet-validation CI workflow; also runnable locally:

    cargo build --release
    python3 scripts/planet_drift_check.py --binary ./target/release/solunatus

Exit status is non-zero if any delta exceeds its threshold.
"""

import argparse
import json
import re
import subprocess
import sys
import time
import urllib.parse
import urllib.request

HORIZONS_API = "https://ssd.jpl.nasa.gov/api/horizons.api"

PLANET_IDS = {
    "Mercury": "199",
    "Venus": "299",
    "Mars": "499",
    "Jupiter": "599",
    "Saturn": "699",
    "Uranus": "799",
    "Neptune": "899",
}

# Default test epochs (local dates; solunatus resolves --date to local noon).
DEFAULT_DATES = ["2026-06-15", "2030-03-01", "2040-09-10"]

# Default observer: Boston, USA.
DEFAULT_LAT = 42.3601
DEFAULT_LON = -71.0589
DEFAULT_TZ = "America/New_York"

# Thresholds. The planet model uses Keplerian mean elements with major
# Jupiter/Saturn/Uranus perturbation terms, so a few hundredths of a degree
# of drift vs the JPL integrated ephemeris is expected and fine for
# rise/set-level accuracy (0.25 deg of sky motion is ~1 minute of time).
# Observed 1990-2049: positions within 0.06 deg, distance within 0.41%.
MAX_ANGLE_DEG = 0.25
# Magnitudes use simple phase-angle formulas that degrade for Venus as a
# thin crescent near inferior conjunction (observed 0.64 mag off in Jan
# 1990); everything else stays within ~0.25 mag.
MAX_MAG = 0.8
MAX_DIST_FRAC = 0.005  # relative error in distance


def run_solunatus(binary, date, lat, lon, tz):
    cmd = [
        binary,
        "--lat", str(lat),
        f"--lon={lon}",
        "--tz", tz,
        "--date", date,
        "--json",
    ]
    out = subprocess.run(cmd, capture_output=True, text=True, check=True)
    return json.loads(out.stdout)


def query_horizons(planet_id, utc_str, lat, lon, retries=3):
    # utc_str: "YYYY-MM-DD HH:MM:SS UTC" from solunatus JSON output
    start = utc_str.replace(" UTC", "")
    params = {
        "format": "text",
        "COMMAND": f"'{planet_id}'",
        "OBJ_DATA": "'NO'",
        "MAKE_EPHEM": "'YES'",
        "EPHEM_TYPE": "'OBSERVER'",
        "CENTER": "'coord@399'",
        "COORD_TYPE": "'GEODETIC'",
        "SITE_COORD": f"'{lon},{lat},0'",
        "START_TIME": f"'{start}'",
        "STOP_TIME": f"'{start[:-2]}59'",
        "STEP_SIZE": "'2'",
        "QUANTITIES": "'4,9,20'",  # apparent az/el, vis. magnitude, range
        "ANG_FORMAT": "'DEG'",
        "APPARENT": "'AIRLESS'",
    }
    url = HORIZONS_API + "?" + urllib.parse.urlencode(params)
    last_err = None
    for attempt in range(retries):
        try:
            with urllib.request.urlopen(url, timeout=60) as resp:
                text = resp.read().decode()
            break
        except Exception as err:  # noqa: BLE001 - retry any transport error
            last_err = err
            time.sleep(5 * (attempt + 1))
    else:
        raise RuntimeError(f"Horizons query failed for {planet_id}: {last_err}")

    if "$$SOE" not in text:
        raise RuntimeError(f"No ephemeris data in Horizons response for {planet_id}:\n{text[:500]}")
    line = text.split("$$SOE")[1].strip().splitlines()[0]
    # Format: date time [flags] az el mag surf_brt range range_rate
    nums = [p for p in line.split() if re.match(r"^-?\d+\.?\d*$", p)]
    return {
        "azimuth": float(nums[0]),
        "altitude": float(nums[1]),
        "magnitude": float(nums[2]),
        "distance_au": float(nums[4]),
    }


def angle_delta(a, b):
    d = abs(a - b) % 360.0
    return min(d, 360.0 - d)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", default="./target/release/solunatus")
    parser.add_argument("--dates", nargs="*", default=DEFAULT_DATES)
    parser.add_argument("--lat", type=float, default=DEFAULT_LAT)
    parser.add_argument("--lon", type=float, default=DEFAULT_LON)
    parser.add_argument("--tz", default=DEFAULT_TZ)
    parser.add_argument("--max-angle", type=float, default=MAX_ANGLE_DEG)
    parser.add_argument("--max-mag", type=float, default=MAX_MAG)
    parser.add_argument("--max-dist-frac", type=float, default=MAX_DIST_FRAC)
    args = parser.parse_args()

    failures = 0
    for date in args.dates:
        data = run_solunatus(args.binary, date, args.lat, args.lon, args.tz)
        utc = data["datetime"]["utc"]
        planets = {p["name"]: p for p in data["planets"]}
        print(f"\n=== {date} (UTC {utc}) lat={args.lat} lon={args.lon} ===")
        print(f"{'Planet':9} {'dAz(deg)':>9} {'dAlt(deg)':>10} {'dMag':>6} {'dDist(%)':>9}  result")
        for name, pid in PLANET_IDS.items():
            ref = query_horizons(pid, utc, args.lat, args.lon)
            got = planets[name]
            d_az = angle_delta(got["azimuth"], ref["azimuth"])
            d_alt = abs(got["altitude"] - ref["altitude"])
            d_mag = abs(got["magnitude"] - ref["magnitude"])
            d_dist = abs(got["distance_au"] - ref["distance_au"]) / ref["distance_au"]
            ok = (
                d_az <= args.max_angle
                and d_alt <= args.max_angle
                and d_mag <= args.max_mag
                and d_dist <= args.max_dist_frac
            )
            if not ok:
                failures += 1
            print(
                f"{name:9} {d_az:9.4f} {d_alt:10.4f} {d_mag:6.2f} {d_dist * 100:9.4f}  "
                + ("PASS" if ok else "FAIL")
            )
            # Horizons asks API users to keep request rates modest.
            time.sleep(1)

    if failures:
        print(f"\n{failures} planet check(s) exceeded thresholds", file=sys.stderr)
        return 1
    print("\nAll planet checks within thresholds.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
