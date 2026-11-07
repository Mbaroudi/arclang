# An OPC UA server for the live twin test: House.Outdoor.T and House.Room.T
# (namespace urn:arclang:house), the synthetic day of the heat pump example
# written row by row, each value with the row's instant as its timestamp.
#   python opcua_replay_server.py house-2026-01-14.csv PAUSE_SECONDS WAIT_SECONDS
# Needs asyncua (pip install asyncua); listens on opc.tcp://localhost:4840/arclang/.
import asyncio, csv, sys
from datetime import datetime
from asyncua import Server, ua


def instant(stamp):
    return datetime.fromisoformat(stamp.replace("Z", "+00:00"))


async def main(path, pause, wait):
    server = Server()
    await server.init()
    server.set_endpoint("opc.tcp://0.0.0.0:4840/arclang/")
    server.set_security_policy([ua.SecurityPolicyType.NoSecurity])
    ns = await server.register_namespace("urn:arclang:house")
    house = await server.nodes.objects.add_object(ua.NodeId("House", ns), "House")
    outdoor = await house.add_variable(ua.NodeId("House.Outdoor.T", ns), "OutdoorT", 0.0)
    room = await house.add_variable(ua.NodeId("House.Room.T", ns), "RoomT", 0.0)
    rows = [r for r in csv.reader(l for l in open(path) if not l.startswith("#"))][1:]
    # No reading yet: a value dated now would be the latest instant of the window.
    for node in (outdoor, room):
        await node.write_value(ua.DataValue(ua.Variant(float("nan"), ua.VariantType.Double), StatusCode_=ua.StatusCode(ua.StatusCodes.BadWaitingForInitialData), SourceTimestamp=instant(rows[0][0])))
    async with server:
        print("ready", flush=True)
        await asyncio.sleep(wait)
        for stamp, o, i in rows:
            for node, v in ((outdoor, float(o)), (room, float(i))):
                await node.write_value(ua.DataValue(ua.Variant(v, ua.VariantType.Double), SourceTimestamp=instant(stamp), ServerTimestamp=instant(stamp)))
            await asyncio.sleep(pause)
        print("replayed", len(rows), flush=True)
        await asyncio.sleep(3600)


asyncio.run(main(sys.argv[1], float(sys.argv[2]), float(sys.argv[3])))
