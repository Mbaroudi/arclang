// The last minute of a heavy vehicle returning to a tower's arms, straight down (one dimension).
// Belly flop: the vehicle falls flat, drag grows until it balances the weight (terminal velocity).
// Flip: it turns upright in a few seconds, its area to the wind shrinks.
// Landing burn: engines relight when the braking they must give reaches their lowest thrust (they
// cannot throttle below it), follow the thrust that brings the speed to zero at the arms' height,
// and are cut when the vehicle is nearly at rest.
// Catch: the arms are a spring and a damper under the two catch fittings.
// All numbers are illustrative, round, and chosen for the example; they are not those of any real vehicle.
within ;
package CatchPhysics "Descent and catch of a heavy vehicle"
  model Descent "From the belly flop to the weight on the arms"
    parameter Real m_dry(unit = "kg") = 100000 "Vehicle without propellant";
    parameter Real m_prop0(unit = "kg") = 8000 "Propellant kept for the landing burn";
    parameter Real h0(unit = "m") = 1800 "Altitude where the run starts, already at terminal velocity";
    parameter Real v0(unit = "m/s") = 60 "Speed at the start (downwards)";
    parameter Real rho0(unit = "kg/m3") = 1.2 "Air density at the ground";
    parameter Real H(unit = "m") = 8500 "Scale height of the atmosphere";
    parameter Real Cd = 1 "Drag coefficient";
    parameter Real A_belly(unit = "m2") = 450 "Area to the wind, falling flat";
    parameter Real A_upright(unit = "m2") = 60 "Area to the wind, upright";
    parameter Real h_flip(unit = "m") = 1100 "Altitude where the flip starts";
    parameter Real t_flip(unit = "s") = 6 "Duration of the flip";
    parameter Real T_max(unit = "N") = 2.6e6 "Thrust of the engines, at most";
    parameter Real T_min(unit = "N") = 1.5e6 "Thrust of the engines, at least (they cannot throttle below it)";
    parameter Real Isp(unit = "s") = 330 "Specific impulse";
    parameter Real h_arm(unit = "m") = 146 "Height of the catch fittings' rest position on the tower";
    parameter Real v_cut(unit = "m/s") = 1 "Engines are cut below this speed";
    parameter Real k(unit = "N/m") = 2e6 "Stiffness of the two arms together";
    parameter Real c(unit = "N.s/m") = 2e5 "Damping of the two arms together";
    constant Real g = 9.80665;
    Real h(unit = "m", start = h0, fixed = true) "Altitude of the catch fittings";
    Real v(unit = "m/s", start = v0, fixed = true) "Speed, downwards";
    Real prop(unit = "kg", start = m_prop0, fixed = true) "Propellant left";
    Real m(unit = "kg") "Mass";
    Real A(unit = "m2") "Area to the wind";
    Real drag(unit = "N") "Drag";
    Real T_cmd(unit = "N") "Thrust that brings the speed to zero at the arms";
    Real T(unit = "N") "Thrust of the engines";
    Real F_arms(unit = "N") "What the arms push up with, together";
    Real F_fitting(unit = "N") "What each catch fitting carries";
    Real v_contact(unit = "m/s", start = 0) "Speed when the fittings reach the arms";
    Real propellant_used(unit = "kg") "Propellant the landing burn used";
    Real t_flip_start(start = 1e9) "Time the flip started";
    Boolean flipped(start = false) "Flip done";
    Boolean ignited(start = false) "Engines relit";
    Boolean cut(start = false) "Engines cut";
    Boolean touched(start = false) "Fittings have reached the arms";
  equation
    m = m_dry + prop;
    when h <= h_flip then
      t_flip_start = time;
    end when;
    when time >= t_flip_start + t_flip then
      flipped = true;
    end when;
    A = if h > h_flip then A_belly elseif flipped then A_upright else A_belly + (A_upright - A_belly) * (time - t_flip_start) / t_flip;
    drag = 0.5 * rho0 * exp(-max(h, 0) / H) * Cd * A * v ^ 2;
    T_cmd = m * (g + v ^ 2 / (2 * max(h - h_arm, 1)));
    when flipped and T_cmd >= T_min then
      ignited = true;
    end when;
    when ignited and v < v_cut then
      cut = true;
    end when;
    when h <= h_arm and not pre(touched) then
      touched = true;
      v_contact = v;
    end when;
    T = if ignited and not cut and not touched then min(max(T_cmd, T_min), T_max) else 0;
    F_arms = if h < h_arm then k * (h_arm - h) + c * max(v, 0) else 0;
    F_fitting = F_arms / 2;
    der(h) = -v;
    m * der(v) = m * g - drag - T - F_arms;
    der(prop) = -T / (Isp * g);
    propellant_used = m_prop0 - prop;
  end Descent;

  model HeavyVehicle "A vehicle 15 % heavier than the design: the engines brake it from higher up, with more propellant"
    extends Descent(m_dry = 115000, m_prop0 = 9500);
  end HeavyVehicle;

  model FlipTooLow "The flip starts too low: the vehicle is still turning when it reaches the arms"
    extends Descent(h_flip = 400);
  end FlipTooLow;
end CatchPhysics;
