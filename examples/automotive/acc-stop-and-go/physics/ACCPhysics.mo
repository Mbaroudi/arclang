package ACCPhysics
  "Longitudinal dynamics of the ego vehicle and the ACC control law (constant time gap policy, ISO 15622), with the traffic scenarios the requirements are verified on"

  model Ego
    "Ego vehicle: the acceleration commanded by the ACC reaches the wheels through the powertrain or the brakes with a first-order lag; it stops and stays at standstill rather than reversing"
    parameter Real tauActuator(unit = "s") = 0.35 "Lag of the brake and powertrain response";
    parameter Real v0(unit = "m/s") = 27.78 "Initial speed";
    input Real aCmd "Acceleration commanded by the ACC (m/s2)";
    Real a(start = 0, fixed = true) "Acceleration reached at the wheels (m/s2)";
    Real v(unit = "m/s", start = v0, fixed = true) "Speed";
    Real x(unit = "m", start = 0, fixed = true) "Distance travelled";
  equation
    der(a) = (aCmd - a) / tauActuator;
    der(v) = if v <= 0 and a < 0 then 0 else a;
    der(x) = max(v, 0);
  end Ego;

  model Lead
    "Vehicle ahead: a speed profile made of constant-acceleration phases"
    parameter Real x0(unit = "m") = 50 "Initial position ahead of the ego vehicle's front";
    parameter Real v0(unit = "m/s") = 27.78 "Initial speed";
    parameter Real tBrake(unit = "s") = 1e6 "When it starts braking";
    parameter Real aBrake = 3 "Its deceleration (m/s2)";
    parameter Real vLow(unit = "m/s") = 13.9 "Speed it brakes down to";
    parameter Real tGo(unit = "s") = 1e6 "When it accelerates again";
    parameter Real aGo = 1 "Its acceleration (m/s2)";
    parameter Real vHigh(unit = "m/s") = 25 "Speed it accelerates up to";
    Real v(unit = "m/s", start = v0, fixed = true) "Speed";
    Real x(unit = "m", start = x0, fixed = true) "Position";
    Real a "Acceleration, measured by the radar (m/s2)";
  equation
    a = der(v);
    der(v) = if time >= tGo and v < vHigh then aGo
             elseif time >= tBrake and time < tGo and v > vLow then -aBrake
             else 0;
    der(x) = v;
  end Lead;

  model Controller
    "ACC longitudinal control: speed control without a target, distance control behind one (constant time gap), the lower of the two, bounded in acceleration and jerk; a takeover request when the deceleration needed exceeds what the ACC may command"
    parameter Real vSet(unit = "m/s") = 36.1 "Set speed (130 km/h)";
    parameter Real tau(unit = "s") = 1.8 "Time gap chosen by the driver";
    parameter Real d0(unit = "m") = 4 "Distance kept at standstill";
    parameter Real kv = 0.3 "Speed control gain (1/s)";
    parameter Real kd = 0.12 "Distance error gain (1/s2)";
    parameter Real kr = 0.55 "Relative speed gain (1/s)";
    parameter Real aMax = 2.0 "Highest acceleration commanded (m/s2), ISO 15622";
    parameter Real dMax = 3.5 "Highest deceleration commanded (m/s2), ISO 15622";
    parameter Real jMax = 2.5 "Highest jerk commanded (m/s3), ISO 15622";
    parameter Real tf(unit = "s") = 0.2 "Time constant of the command filter";
    input Real v(unit = "m/s") "Ego speed";
    input Real gap(unit = "m") "Distance to the target";
    input Real vTarget(unit = "m/s") "Speed of the target";
    input Real aTarget "Acceleration of the target (m/s2)";
    Real dRef(unit = "m") "Distance to keep";
    Real aSpeed "Acceleration of speed control (m/s2)";
    Real aGap "Acceleration of distance control (m/s2)";
    Real aWanted "Acceleration wanted before the limits (m/s2)";
    Real aCmd(start = 0, fixed = true) "Acceleration commanded (m/s2)";
    Real decel "Deceleration commanded, positive when braking (m/s2)";
    Real jerk "Jerk of the command (m/s3)";
    Real timeGap(unit = "s") "Time gap to the target";
    Real need "Deceleration needed to stop closing in at the standstill distance (m/s2)";
    Boolean takeover(start = false, fixed = true) "Takeover request to the driver";
    Boolean hold(start = false, fixed = true) "Holding the car at standstill behind a stopped target";
    parameter Real aHold = 1.5 "Deceleration requested to hold the car stopped (m/s2)";
    Real takeoverRequest "1 while the takeover request is shown";
  equation
    dRef = d0 + tau * v;
    aSpeed = kv * (vSet - v);
    aGap = kd * (gap - dRef) + kr * (vTarget - v);
    // Behind a stopped target, the car is brought to a full stop and held there;
    // it drives off when the target moves away (the driver's confirmation is the
    // mode machine's, verified by executing it).
    when v < 0.5 and vTarget < 0.2 and gap < dRef + 2 then
      hold = true;
    elsewhen vTarget > 1.0 then
      hold = false;
    end when;
    aWanted = if hold then -aHold else min(aSpeed, aGap);
    der(aCmd) = max(-jMax, min(jMax, (max(-dMax, min(aMax, aWanted)) - aCmd) / tf));
    decel = -aCmd;
    jerk = der(aCmd);
    timeGap = gap / max(v, 0.5);
    // The deceleration that stops the ego vehicle d0 behind the target: from the relative
    // speed alone, or, when the target brakes, from the two stopping distances.
    need = max(if v > vTarget then (v - vTarget) ^ 2 / (2 * max(gap - d0, 0.5)) else 0,
               if aTarget < -0.5 then v ^ 2 / (2 * max(gap - d0 + vTarget ^ 2 / (2 * (-aTarget)), 0.5)) else 0);
    when need > dMax then
      takeover = true;
    elsewhen need < 0.5 * dMax and gap > dRef then
      takeover = false;
    end when;
    takeoverRequest = if takeover then 1 else 0;
  end Controller;

  model FollowBraking
    "Following at 100 km/h, 1.8 s; the lead vehicle brakes at 3 m/s2 down to 50 km/h, then accelerates back to 90 km/h"
    Ego ego(v0 = 27.78);
    Lead lead(x0 = 55, v0 = 27.78, tBrake = 20, aBrake = 3, vLow = 13.9, tGo = 60, aGo = 1, vHigh = 25);
    Controller acc(tau = 1.8);
    Real gap(unit = "m") "Distance between the two vehicles";
  equation
    gap = lead.x - ego.x;
    acc.v = ego.v;
    acc.gap = gap;
    acc.vTarget = lead.v;
    acc.aTarget = lead.a;
    ego.aCmd = acc.aCmd;
  end FollowBraking;

  model CutIn
    "Cruising at 100 km/h behind a vehicle at 1.8 s; a slower vehicle (80 km/h) cuts in 22 m ahead"
    parameter Real tCut(unit = "s") = 10 "When the vehicle cuts in";
    Ego ego(v0 = 27.78);
    Lead lead(x0 = 55, v0 = 27.78);
    Lead cutter(x0 = 22 + (27.78 - 22.22) * 10, v0 = 22.22) "Cut-in vehicle, 22 m ahead of the ego vehicle when it enters the lane";
    Controller acc(tau = 1.8);
    Real gap(unit = "m") "Distance to the vehicle followed";
    Real target "1 once the cut-in vehicle is the target";
  equation
    target = if time >= tCut then 1 else 0;
    gap = if time >= tCut then cutter.x - ego.x else lead.x - ego.x;
    acc.v = ego.v;
    acc.gap = gap;
    acc.vTarget = if time >= tCut then cutter.v else lead.v;
    acc.aTarget = if time >= tCut then cutter.a else lead.a;
    ego.aCmd = acc.aCmd;
  end CutIn;

  model HardBraking
    "Following at 100 km/h, 1.8 s; the lead vehicle brakes at 7 m/s2, beyond what the ACC may command: the ACC raises the takeover request at once, the driver brakes 1 s later and stops behind it"
    parameter Real tReaction(unit = "s") = 1.0 "Driver reaction time to the takeover request";
    parameter Real aDriver = 8 "Driver braking (m/s2)";
    Ego ego(v0 = 27.78);
    Lead lead(x0 = 55, v0 = 27.78, tBrake = 5, aBrake = 7, vLow = 0);
    Controller acc(tau = 1.8);
    Real gap(unit = "m") "Distance between the two vehicles";
    Real tRequest(unit = "s", start = 1e6, fixed = true) "When the takeover request was raised";
    Real driverBraking "1 once the driver brakes";
  equation
    gap = lead.x - ego.x;
    acc.v = ego.v;
    acc.gap = gap;
    acc.vTarget = lead.v;
    acc.aTarget = lead.a;
    when acc.takeover then
      tRequest = time;
    end when;
    driverBraking = if time >= tRequest + tReaction then 1 else 0;
    ego.aCmd = if driverBraking > 0.5 then -aDriver else acc.aCmd;
  end HardBraking;

  model StopAndGo
    "Congestion: following at 50 km/h, the lead vehicle stops at 1.5 m/s2, stands for 8 s and drives off at 1 m/s2"
    Ego ego(v0 = 13.9);
    Lead lead(x0 = 30, v0 = 13.9, tBrake = 5, aBrake = 1.5, vLow = 0, tGo = 22, aGo = 1, vHigh = 13.9);
    Controller acc(tau = 1.8);
    Real gap(unit = "m") "Distance between the two vehicles";
  equation
    gap = lead.x - ego.x;
    acc.v = ego.v;
    acc.gap = gap;
    acc.vTarget = lead.v;
    acc.aTarget = lead.a;
    ego.aCmd = acc.aCmd;
  end StopAndGo;

  annotation(uses(Modelica(version = "4.1.0")));
end ACCPhysics;
