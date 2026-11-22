# Adaptive cruise control, followed in CARLA

A digital twin on a driving simulator: the model of a gap law, compared sample by sample with the vehicle that [CARLA](https://carla.org/) drives.

**Level:** Advanced · **Shows:** a digital twin on recorded data and on live data (MQTT), a Modelica model, a simulator as the source of measurements

## What it models

A follower keeps its gap behind a lead vehicle that drives, slows to a crawl, stops and goes again, for 150 s. The follower's cruise control asks an acceleration from the gap and the closing speed (a constant time gap of 1.5 s above 6 m at rest), and the vehicle answers it with a delay.

`physics/AccPhysics.mo` is that law and that vehicle in a dozen lines of Modelica. The twin gives it the lead vehicle's measured speed (`drives:`), compares its speed and its gap with those measured (`compares:`), and checks on the run that the follower never comes closer than 4 m (`REQ-GAP`).

## What to look at

- `acc_carla_twin.arc`: the two twins, one on a recorded run and one on a live server.
- `physics/AccPhysics.mo`: the model the measurements drive.
- `data/carla-town04.csv`: a run recorded from CARLA 0.9.15 on the map Town04. **It is a simulator's data, not a road vehicle's**: its header says where it comes from.
- `carla/bridge.py`: the client that drives both vehicles in CARLA and publishes what the twin reads.

## Try it

The recorded run needs OpenModelica only (in VS Code, *ArcLang: Set up OpenModelica*; on arclang.fr, sign in). From the command line, at the root of the repository:

```bash
arclang twin examples/automotive/acc-carla-twin/acc_carla_twin.arc --twin TW-CARLA-RUN
```

```
twin 'Follower in CARLA, a recorded run' (TW-CARLA-RUN): PASS
  data: data/carla-town04.csv, 751 samples over 150 s from 2026-11-22T11:26:24.078Z
  ok   v_ego ~ ego_speed: max deviation 2.7652 m/s, RMS 0.2899 m/s, 99.7 % of the samples within 1.5 m/s (95.0 % required), first beyond at 72 s
  ok   gap ~ gap: max deviation 2.5077 m, RMS 0.6775 m, 100.0 % of the samples within 3 m (95.0 % required)
  ok   min(gap) >= 4 m: min(gap) = 6.005 m
```

In VS Code: *ArcLang: Run the digital twins* shows the measurement and the model on one chart for each comparison.

## Live, on a CARLA server

The live twin reads a local MQTT broker that `carla/bridge.py` feeds. Three containers, from this folder:

```bash
docker network create carla-net
docker run -d --name carla-mqtt --network carla-net --network-alias mosquitto \
  -p 127.0.0.1:1883:1883 -v "$PWD/carla/mosquitto.conf:/mosquitto/config/mosquitto.conf:ro" eclipse-mosquitto:2
docker run -d --name carla-server --platform linux/amd64 --network carla-net --network-alias carla --memory 9g \
  carlasim/carla:0.9.15 /bin/bash ./CarlaUE4.sh -nullrhi -nosound -carla-rpc-port=2000
docker build --platform linux/amd64 -t arclang-carla-client carla
```

Then the twin in one terminal and the client in another:

```bash
arclang twin acc_carla_twin.arc --twin TW-CARLA-LIVE --live
docker run --rm --platform linux/amd64 --network carla-net arclang-carla-client --town Town04 --pace 4
```

```
connected: mqtt://localhost:1883 (carla/acc)
12:34:51 twin 'Follower in CARLA, live': PASS over 287 samples; v_ego within 100.0 %, gap within 100.0 %
12:35:08 twin 'Follower in CARLA, live': PASS over 677 samples; v_ego within 99.7 %, gap within 100.0 %
12:35:27 twin 'Follower in CARLA, live': PASS over 751 samples; v_ego within 99.7 %, gap within 100.0 %
```

What to know about the server:

- **No graphics card is needed.** `-nullrhi` runs CARLA without rendering: the physics, the vehicles and the ray-cast radar work; the cameras do not. This is how the run above was made, on a laptop with no GPU for Docker, the x86-64 image emulated on an ARM processor.
- **The image is large**: 8.9 GB to download, about 30 GB on disk.
- **It can crash at start under emulation.** It did once here, without `--memory` and the port; started again with the line above, it ran.
- **Time is the simulation's.** The server steps 0.05 s at a time, about twenty times faster than the clock without rendering. `--pace 4` slows the client to four simulated seconds a second so that the twin's `refresh` sees the run go by; the twin counts its window in the data's own time.
- Town04 is in the base image; a map that is not (Town06) is refused by the server.

Remove it all with `docker rm -f carla-server carla-mqtt`, `docker network rm carla-net` and `docker rmi carlasim/carla:0.9.15 arclang-carla-client`.

## What the client does

- It lays one route along a lane of the town and puts the follower on it, the lead vehicle 30 m ahead. Both steer along the route (pure pursuit).
- The lead vehicle follows a speed in time: 12 m/s, 5 m/s, 14 m/s, a stop, 10 m/s, a stop.
- The follower applies the gap law of the Modelica model to the gap and the speeds it measures, then an inner loop turns the acceleration asked into pedals by the acceleration measured (proportional and integral).
- Five times a simulated second it publishes `{"time", "lead_speed", "ego_speed", "gap", "radar_gap"}` on the topic `carla/acc`. `gap` is the distance along the lane between the two bumpers, from the vehicles' positions. `radar_gap` is the nearest echo of a radar on the follower's nose: it agrees with `gap` at rest and reads shorter at times in motion, so the twin compares `gap`.

## To understand by changing it

- **Take the inner loop out** (in `carla/bridge.py`, set the pedals in proportion to the acceleration asked): the follower then settles farther back than the law asks, because the pedals only balance the drag. The twin fails, with 42 % of the gaps within 3 m: the model no longer describes the vehicle.
- **Shorten the time gap** to 0.5 s in both the client and the model and read `min(gap)` against `REQ-GAP`.
- **Change `lag`** in the model: the comparison of the speeds says how fast the vehicle really answers.

## Limits

- The gap law is written twice, in Modelica and in the Python client. Nothing checks that the two stay the same.
- The measurements have no noise and no latency: a simulator's positions, read exactly.
- One scenario, one map, one vehicle (CARLA's `vehicle.tesla.model3`), a flat straight lane for most of the run.
- CARLA's Traffic Manager, its ROS bridge and OpenSCENARIO are not used.

See [docs/digital-twin.md](../../../docs/digital-twin.md).
