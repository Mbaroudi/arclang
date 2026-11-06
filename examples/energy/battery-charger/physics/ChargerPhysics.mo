// The physics the charge controller is closed on (docs/embedded-c.md, "The
// plant in the loop"): a 48 V pack behind its power stage. The controller
// asks the stage for a current; the stage follows it with a short lag; the
// pack's voltage is its open-circuit voltage, rising with the charge, plus
// the drop on its internal resistance. Nothing here is specific to ArcLang:
// any Modelica tool simulates it, and arclang builds it as an FMU.
package ChargerPhysics
  model Battery "A 48 V pack behind its power stage"
    input Real i_set(unit = "A") "The current the controller asks the stage for";
    parameter Real tau(unit = "s") = 0.02 "Time constant of the power stage";
    parameter Real capacity(unit = "A.s") = 72000 "20 Ah";
    parameter Real soc0 = 0.7 "State of charge at the start";
    parameter Real r(unit = "Ohm") = 0.08 "Internal resistance";
    Real soc(start = soc0, fixed = true) "State of charge, 0 to 1";
    Real i_batt(unit = "A", start = 0, fixed = true) "The current into the pack";
    Real v_batt(unit = "V") "The voltage at the terminals";
  equation
    der(i_batt) = (i_set - i_batt) / tau;
    der(soc) = i_batt / capacity;
    v_batt = 48 + 8 * soc + r * i_batt;
  end Battery;
end ChargerPhysics;
