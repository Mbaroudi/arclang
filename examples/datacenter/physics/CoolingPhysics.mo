// Hand-written physics of examples/datacenter/ai_liquid_cooling.arc.
// It extends the package `arclang export -f modelica` generates
// (AILiquidCooling), which holds the structure and is never edited: each
// model below extends a generated partial model and adds equations, and
// the systems redeclare every part of a generated configuration.
within ;
package CoolingPhysics "Physics of the AI hall's liquid cooling"
  constant Modelica.Units.SI.SpecificHeatCapacity cp = 3800 "Coolant (water with 25 % propylene glycol)";

  model Racks "A training job's heat, most of it to the cold plates; the hottest GPU's cold plate"
    extends AILiquidCooling.Physical.GPURacks;
    parameter Modelica.Units.SI.Time job_start = 3600 "The training job starts at 1 h, from idle";
    parameter Modelica.Units.SI.Time ramp = 300 "From idle to full power";
    parameter Modelica.Units.SI.MassFlowRate nominal_flow = 44 "Flow the cold plates are rated at";
    Modelica.Thermal.HeatTransfer.Sources.PrescribedHeatFlow toCoolant "Heat the cold plates give the coolant";
    Real load(unit="1") = idle_load + (1 - idle_load) * min(max((time - job_start) / ramp, 0), 1) "Share of the GPUs' power the job uses";
    Real cap(unit="1") = min(max(power_cap, 0.5), 1) "Share of the power the scheduler allows";
    Modelica.Units.SI.Power it_power = rack_count * rack_power * load * cap "Power of the racks";
    Modelica.Units.SI.HeatFlowRate to_liquid = liquid_fraction * it_power "Heat to the cold plates";
    Modelica.Units.SI.Energy it_energy(start=0, fixed=true) "Energy the racks used";
    Modelica.Units.SI.MassFlowRate massFlow = max(coolant_flow, 0.05 * nominal_flow) "Coolant through the racks";
    Modelica.Units.SI.TemperatureDifference rise = to_liquid / (massFlow * CoolingPhysics.cp) "Coolant warming across the racks";
  equation
    toCoolant.Q_flow = to_liquid;
    connect(toCoolant.port, cold_plates);
    der(it_energy) = it_power;
    // The hottest GPU sits at the return end, and a cold plate conducts less at a lower flow.
    gpu_temperature = cold_plates.T + rise / 2 + gpu_power * load * cap / (cold_plate_conductance * (massFlow / nominal_flow) ^ 0.8);
  end Racks;

  model CDU "The racks' coolant, its pumps and the plate exchanger to the facility water"
    extends AILiquidCooling.Physical.CoolantDistributionUnit;
    parameter Modelica.Units.SI.Time pump_failure_at = 1e9 "A pump stops (never, by default)";
    parameter Modelica.Units.SI.Temperature coolant_start = 305.15 "Coolant at the start of the day";
    Modelica.Thermal.HeatTransfer.Components.HeatCapacitor coolant(C=coolant_capacity, T(start=coolant_start, fixed=true)) "The racks' loop, at its mean temperature";
    Modelica.Thermal.HeatTransfer.Sources.PrescribedHeatFlow fromCoolant;
    Modelica.Thermal.HeatTransfer.Sources.PrescribedHeatFlow toPrimary;
    Real running = if time < pump_failure_at then pump_count else pump_count - 1 "Pumps that can run";
    Modelica.Units.SI.MassFlowRate massFlow = pump_flow * max(min(running, needed_pumps), 0) "The spare pump only replaces a stopped one";
    Modelica.Units.SI.ThermalConductance conductance = hx_conductance * (max(massFlow, 1e-3) / (pump_flow * needed_pumps)) ^ 0.8 "The exchanger conducts less at a lower flow";
    Modelica.Units.SI.HeatFlowRate exchanged = conductance * (coolant.T - primary.T) "Heat given to the facility water";
    Modelica.Units.SI.Power electrical = pump_power * max(min(running, needed_pumps), 0) "Pumps' electricity";
    Modelica.Units.SI.Energy energy(start=0, fixed=true);
  equation
    connect(secondary, coolant.port);
    connect(fromCoolant.port, coolant.port);
    connect(toPrimary.port, primary);
    fromCoolant.Q_flow = -exchanged;
    toPrimary.Q_flow = exchanged;
    // The coolant leaves the exchanger colder than its mean by half its warming.
    supply_temperature = coolant.T - exchanged / (2 * max(massFlow, 1e-3) * CoolingPhysics.cp);
    coolant_flow = massFlow;
    der(energy) = electrical;
  end CDU;

  model FacilityWater "The facility water loop, one thermal mass"
    extends AILiquidCooling.Physical.FacilityWaterLoop;
    Modelica.Thermal.HeatTransfer.Components.HeatCapacitor water(C=water_capacity, T(start=300.15, fixed=true));
  equation
    connect(from_cdu, water.port);
    connect(to_coolers, water.port);
  end FacilityWater;

  model Reject "Dry coolers whose fans follow the command, a chiller trimming above full fan speed"
    extends AILiquidCooling.Physical.DryCoolersAndChiller;
    Modelica.Thermal.HeatTransfer.Sources.PrescribedHeatFlow fromWater;
    Modelica.Thermal.HeatTransfer.Sources.PrescribedHeatFlow toAir;
    Real fan(unit="1") = min(max(command, 0.1), 1) "Fan speed: command 0 to 1";
    Real trim(unit="1") = min(max(command - 1, 0), 1) "Chiller load: command 1 to 2";
    Modelica.Units.SI.HeatFlowRate free = cooler_conductance * fan * max(water.T - air.T, 0) "Free cooling: the air must be colder than the water";
    Modelica.Units.SI.HeatFlowRate chilled = chiller_capacity * trim;
    parameter Modelica.Units.SI.TemperatureDifference condenser_approach = 15 "Condensing above the outdoor air";
    parameter Modelica.Units.SI.TemperatureDifference evaporator_approach = 10 "Evaporating below the facility water";
    Real cop(unit="1") = chiller_efficiency * (water.T - evaporator_approach) / max(air.T + condenser_approach - (water.T - evaporator_approach), 5) "Chiller: a share of Carnot between its evaporator and condenser";
    Modelica.Units.SI.Power electrical = fan_power * fan ^ 3 + chilled / cop "Fans and compressor";
    Modelica.Units.SI.Energy energy(start=0, fixed=true);
  equation
    fromWater.Q_flow = -(free + chilled);
    toAir.Q_flow = free + chilled + chilled / cop;
    connect(fromWater.port, water);
    connect(toAir.port, air);
    der(energy) = electrical;
  end Reject;

  model Weather "The design day: warmest at 15 h, at the design temperature"
    extends AILiquidCooling.Physical.OutdoorAir;
    Modelica.Thermal.HeatTransfer.Sources.PrescribedTemperature outdoor;
    Modelica.Units.SI.Temperature T = design_temperature - daily_swing + daily_swing * cos(2 * Modelica.Constants.pi * (time - 15 * 3600) / 86400) "Outdoor air";
  equation
    outdoor.T = T;
    connect(outdoor.port, air);
  end Weather;

  model Control "A PI on the coolant supply (fans, then chiller); a PI capping the GPUs above their limit"
    extends AILiquidCooling.Behaviour.CoolingController.SupplyTemperatureControl;
    Modelica.Blocks.Sources.Constant target(k=setpoint);
    Modelica.Blocks.Continuous.LimPID cooling(controllerType=Modelica.Blocks.Types.SimpleController.PI, k=0.5, Ti=600, yMax=2, yMin=0, initType=Modelica.Blocks.Types.Init.InitialOutput, y_start=0.5) "0 to 1: fans; 1 to 2: chiller";
    Modelica.Blocks.Sources.Constant limit(k=cap_above);
    Modelica.Blocks.Continuous.LimPID capping(controllerType=Modelica.Blocks.Types.SimpleController.PI, k=0.2, Ti=120, yMax=1, yMin=0.5, initType=Modelica.Blocks.Types.Init.InitialOutput, y_start=1) "Power allowed: 1 until a GPU passes its limit";
  equation
    // Warmer than the setpoint: more cooling.
    connect(supply_temperature, cooling.u_s);
    connect(target.y, cooling.u_m);
    connect(cooling.y, command);
    connect(limit.y, capping.u_s);
    connect(gpu_temperature, capping.u_m);
    connect(capping.y, power_cap);
  end Control;

  model Controller "The cooling controller, its control implemented"
    extends AILiquidCooling.Physical.CoolingController(redeclare CoolingPhysics.Control supplyTemperatureControl);
  end Controller;

  model SummerDay "N+1 pumps on the design summer day, a training job from 1 h"
    extends AILiquidCooling.Systems.N1Pumps(
      redeclare CoolingPhysics.Racks gpuRacks,
      redeclare CoolingPhysics.CDU coolantDistributionUnit,
      redeclare CoolingPhysics.FacilityWater facilityWaterLoop,
      redeclare CoolingPhysics.Reject dryCoolersAndChiller,
      redeclare CoolingPhysics.Weather outdoorAir,
      redeclare CoolingPhysics.Controller coolingController);
    Real ppue(unit="1") = 1 + (coolantDistributionUnit.energy + dryCoolersAndChiller.energy) / max(gpuRacks.it_energy, 1) "Partial PUE of the cooling";
    annotation(experiment(StopTime=86400, Interval=60));
  end SummerDay;

  model PumpFailure "N+1 pumps: one stops at 14 h, on the hottest afternoon"
    extends AILiquidCooling.Systems.N1Pumps(
      redeclare CoolingPhysics.Racks gpuRacks,
      redeclare CoolingPhysics.CDU coolantDistributionUnit(pump_failure_at=14 * 3600),
      redeclare CoolingPhysics.FacilityWater facilityWaterLoop,
      redeclare CoolingPhysics.Reject dryCoolersAndChiller,
      redeclare CoolingPhysics.Weather outdoorAir,
      redeclare CoolingPhysics.Controller coolingController);
    Real ppue(unit="1") = 1 + (coolantDistributionUnit.energy + dryCoolersAndChiller.energy) / max(gpuRacks.it_energy, 1) "Partial PUE of the cooling";
    annotation(experiment(StopTime=86400, Interval=60));
  end PumpFailure;

  model TwoPumpsDay "Two pumps, no spare, on the design summer day"
    extends AILiquidCooling.Systems.TwoPumps(
      redeclare CoolingPhysics.Racks gpuRacks,
      redeclare CoolingPhysics.CDU coolantDistributionUnit,
      redeclare CoolingPhysics.FacilityWater facilityWaterLoop,
      redeclare CoolingPhysics.Reject dryCoolersAndChiller,
      redeclare CoolingPhysics.Weather outdoorAir,
      redeclare CoolingPhysics.Controller coolingController);
    Real ppue(unit="1") = 1 + (coolantDistributionUnit.energy + dryCoolersAndChiller.energy) / max(gpuRacks.it_energy, 1) "Partial PUE of the cooling";
    annotation(experiment(StopTime=86400, Interval=60));
  end TwoPumpsDay;

  model PumpFailureTwoPumps "Two pumps: one stops at 14 h, on the hottest afternoon"
    extends AILiquidCooling.Systems.TwoPumps(
      redeclare CoolingPhysics.Racks gpuRacks,
      redeclare CoolingPhysics.CDU coolantDistributionUnit(pump_failure_at=14 * 3600),
      redeclare CoolingPhysics.FacilityWater facilityWaterLoop,
      redeclare CoolingPhysics.Reject dryCoolersAndChiller,
      redeclare CoolingPhysics.Weather outdoorAir,
      redeclare CoolingPhysics.Controller coolingController);
    Real ppue(unit="1") = 1 + (coolantDistributionUnit.energy + dryCoolersAndChiller.energy) / max(gpuRacks.it_energy, 1) "Partial PUE of the cooling";
    annotation(experiment(StopTime=86400, Interval=60));
  end PumpFailureTwoPumps;
  annotation(uses(Modelica(version="4.1.0")));
end CoolingPhysics;
