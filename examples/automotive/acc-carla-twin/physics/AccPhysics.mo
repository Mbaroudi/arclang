package AccPhysics
  "A follower behind a lead vehicle: the gap law of an adaptive cruise control and the vehicle that answers it"

  model Follow
    // The lead vehicle's speed is measured; the twin drives it.
    Real v_lead(unit = "m/s") = 0 "speed of the lead vehicle";
    parameter Real gap0(unit = "m") = 30 "gap at the start, bumper to bumper";
    parameter Real standstill(unit = "m") = 6 "gap kept at rest";
    parameter Real headway(unit = "s") = 1.5 "time gap kept in motion";
    parameter Real k_gap(unit = "1/s2") = 0.25 "acceleration per metre of gap error";
    parameter Real k_speed(unit = "1/s") = 0.9 "acceleration per m/s of closing speed";
    parameter Real a_max(unit = "m/s2") = 2.0 "strongest acceleration asked";
    parameter Real a_min(unit = "m/s2") = -5.0 "strongest braking asked";
    parameter Real lag(unit = "s") = 0.5 "time the vehicle takes to answer";
    Real x_lead(unit = "m", start = 0, fixed = true);
    Real x_ego(unit = "m", start = 0, fixed = true);
    Real v_ego(unit = "m/s", start = 0, fixed = true) "speed of the follower";
    Real a(unit = "m/s2", start = 0, fixed = true) "acceleration of the follower";
    Real a_cmd(unit = "m/s2") "acceleration the gap law asks";
    Real gap(unit = "m") "gap to the lead vehicle";
  equation
    der(x_lead) = v_lead;
    der(x_ego) = v_ego;
    gap = gap0 + x_lead - x_ego;
    a_cmd = max(a_min, min(a_max, k_gap * (gap - standstill - headway * v_ego) + k_speed * (v_lead - v_ego)));
    lag * der(a) + a = a_cmd;
    der(v_ego) = if v_ego <= 0 and a < 0 then 0 else a;
  end Follow;
end AccPhysics;
