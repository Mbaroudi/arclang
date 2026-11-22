"""A lead vehicle and a follower on one route of a CARLA town; the follower keeps its gap by the
law of twin/physics/AccPhysics.mo. Each step publishes what a twin reads: the lead's speed, the
follower's speed and the gap between them (bumper to bumper, from the two vehicles' positions,
and from the follower's radar when the server gives one)."""
import argparse
import json
import math
import sys
import time

import carla
import paho.mqtt.client as mqtt

STEP = 0.05          # s, the fixed step of the simulation
GAP0 = 30.0          # m, the gap at the start
STANDSTILL, HEADWAY = 6.0, 1.5
K_GAP, K_SPEED = 0.25, 0.9
A_MAX, A_MIN = 2.0, -5.0

# The lead's speed in time: (until s, m/s). It drives, slows, stops and goes again.
PROFILE = [(5, 0), (30, 12), (50, 5), (70, 14), (85, 0), (100, 0), (125, 10), (140, 0), (150, 0)]


def target_speed(t):
    for until, speed in PROFILE:
        if t < until:
            return speed
    return 0.0


def speed(vehicle):
    v = vehicle.get_velocity()
    return math.sqrt(v.x * v.x + v.y * v.y + v.z * v.z)


def steer_to(vehicle, route, index):
    """Pure pursuit on the route: the steering towards a point a few metres ahead, and where the
    vehicle is on the route."""
    tf = vehicle.get_transform()
    here = tf.location
    while index + 1 < len(route) and route[index + 1].transform.location.distance(here) < route[index].transform.location.distance(here):
        index += 1
    ahead = route[min(index + 4, len(route) - 1)].transform.location
    yaw = math.radians(tf.rotation.yaw)
    dx, dy = ahead.x - here.x, ahead.y - here.y
    lateral = -math.sin(yaw) * dx + math.cos(yaw) * dy
    dist2 = max(dx * dx + dy * dy, 1.0)
    return max(-1.0, min(1.0, 2.0 * 2.9 * lateral / dist2 / 0.6)), index


def along(vehicle, route, index):
    """Where the vehicle is along the route, in metres from its start."""
    here, a = vehicle.get_location(), route[index].transform
    forward = a.get_forward_vector()
    return 2.0 * index + (here.x - a.location.x) * forward.x + (here.y - a.location.y) * forward.y


def pedals(accel):
    """Throttle and brake for an acceleration asked, in the plainest way: in proportion."""
    if accel >= 0:
        return min(1.0, accel / 3.0), 0.0
    return 0.0, min(1.0, -accel / 8.0)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--host", default="carla")
    ap.add_argument("--port", type=int, default=2000)
    ap.add_argument("--mqtt", default="mosquitto")
    ap.add_argument("--topic", default="carla/acc")
    ap.add_argument("--duration", type=float, default=150.0)
    ap.add_argument("--town", default="")
    ap.add_argument("--csv", default="")
    ap.add_argument("--pace", type=float, default=0.0, help="simulated seconds per second of the clock; 0 runs as fast as the server goes")
    args = ap.parse_args()

    client = carla.Client(args.host, args.port)
    client.set_timeout(600.0)
    world = client.load_world(args.town) if args.town else client.get_world()
    print("map", world.get_map().name, "server", client.get_server_version(), flush=True)
    settings = world.get_settings()
    settings.synchronous_mode = True
    settings.fixed_delta_seconds = STEP
    settings.no_rendering_mode = True
    world.apply_settings(settings)

    town = world.get_map()
    # The longest run without a junction, so that both vehicles drive one lane.
    best = []
    for spawn in town.get_spawn_points():
        wp, run = town.get_waypoint(spawn.location), []
        while wp and not wp.is_junction and len(run) < 700:
            run.append(wp)
            nxt = wp.next(2.0)
            wp = nxt[0] if nxt else None
        if len(run) > len(best):
            best = run
    route = list(best)
    while len(route) < 900:
        nxt = route[-1].next(2.0)
        if not nxt:
            break
        route.append(nxt[0])
    print("route", len(route) * 2, "m, of which", len(best) * 2, "m before a junction", flush=True)

    library = world.get_blueprint_library()
    car = library.find("vehicle.tesla.model3")

    def spawn_at(index):
        tf = route[index].transform
        tf.location.z += 0.5
        return world.spawn_actor(car, tf)

    ego = spawn_at(2)
    lead = spawn_at(2 + int((GAP0 + 2 * ego.bounding_box.extent.x) / 2.0))
    actors = [ego, lead]
    radar_gap = {"value": None}
    try:
        radar_bp = library.find("sensor.other.radar")
        radar_bp.set_attribute("horizontal_fov", "8")
        radar_bp.set_attribute("vertical_fov", "4")
        radar_bp.set_attribute("range", "120")
        radar = world.spawn_actor(radar_bp, carla.Transform(carla.Location(x=ego.bounding_box.extent.x + 0.1, z=0.7)), attach_to=ego)
        radar.listen(lambda data: radar_gap.__setitem__("value", min((d.depth for d in data), default=None)))
        actors.append(radar)
    except Exception as error:  # the twin then reads the gap from the positions alone
        print("no radar:", error, flush=True)

    broker = mqtt.Client(mqtt.CallbackAPIVersion.VERSION2)
    broker.connect(args.mqtt, 1883)
    broker.loop_start()
    out = open(args.csv, "w") if args.csv else None
    if out:
        out.write("time,lead_speed,ego_speed,gap,radar_gap\n")

    for _ in range(40):  # let the two vehicles settle on the road
        world.tick()
    half = ego.bounding_box.extent.x + lead.bounding_box.extent.x
    epoch = time.time()
    i_ego = i_lead = 0
    pedal_sum = 0.0
    steps = int(args.duration / STEP)
    began = time.time()
    try:
        for k in range(steps + 1):
            t = k * STEP
            v_lead, v_ego = speed(lead), speed(ego)
            gap = along(lead, route, i_lead) - along(ego, route, i_ego) - half

            steer, i_lead = steer_to(lead, route, i_lead)
            throttle, brake = pedals(1.2 * (target_speed(t) - v_lead))
            if target_speed(t) == 0 and v_lead < 0.3:
                throttle, brake = 0.0, 1.0
            lead.apply_control(carla.VehicleControl(throttle=throttle, brake=brake, steer=steer))

            accel = max(A_MIN, min(A_MAX, K_GAP * (gap - STANDSTILL - HEADWAY * v_ego) + K_SPEED * (v_lead - v_ego)))
            steer, i_ego = steer_to(ego, route, i_ego)
            # The inner loop of a cruise control: the pedals follow the acceleration asked, by the
            # acceleration measured (proportional and integral).
            acc, fwd = ego.get_acceleration(), ego.get_transform().get_forward_vector()
            error = accel - (acc.x * fwd.x + acc.y * fwd.y + acc.z * fwd.z)
            pedal_sum = max(-1.0, min(1.0, pedal_sum + 0.6 * error * STEP))
            pedal = max(-1.0, min(1.0, 0.05 * error + pedal_sum))
            throttle, brake = (pedal, 0.0) if pedal >= 0 else (0.0, -pedal)
            if v_ego < 0.3 and accel <= 0:
                throttle, brake, pedal_sum = 0.0, 1.0, 0.0
            ego.apply_control(carla.VehicleControl(throttle=throttle, brake=brake, steer=steer))

            sample = {"time": round(epoch + t, 3), "lead_speed": round(v_lead, 4), "ego_speed": round(v_ego, 4), "gap": round(gap, 4)}
            if radar_gap["value"] is not None:
                sample["radar_gap"] = round(radar_gap["value"], 4)
            if k % 4 == 0:  # 5 samples a second
                broker.publish(args.topic, json.dumps(sample))
                if out:
                    out.write("%s,%s,%s,%s,%s\n" % (sample["time"], sample["lead_speed"], sample["ego_speed"], sample["gap"], sample.get("radar_gap", "")))
            if k % 200 == 0:
                print("t=%5.1f lead=%5.2f ego=%5.2f gap=%6.2f radar=%s wall=%.0fs" % (t, v_lead, v_ego, gap, sample.get("radar_gap"), time.time() - began), flush=True)
            world.tick()
            if args.pace > 0:
                time.sleep(max(0.0, began + t / args.pace - time.time()))
    finally:
        if out:
            out.close()
        settings.synchronous_mode = False
        world.apply_settings(settings)
        for actor in actors:
            actor.destroy()
        broker.loop_stop()
        broker.disconnect()
    print("done", flush=True)


if __name__ == "__main__":
    sys.exit(main())
