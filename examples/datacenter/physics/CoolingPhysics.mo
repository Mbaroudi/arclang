// Hand-written physics of examples/datacenter/ai_liquid_cooling.arc.
// It extends the package `arclang export -f modelica` generates
// (AILiquidCooling), which holds the structure and is never edited: each
// model below extends a generated partial model and adds equations, and
// the systems redeclare every part of a generated configuration.
// Every component is placed and every connection drawn, so OMEdit shows
// each model as a diagram of Modelica Standard Library components.
within ;
package CoolingPhysics "Physics of the AI hall's liquid cooling"
  constant Modelica.Units.SI.SpecificHeatCapacity cp = 3800 "Coolant (water with 25 % propylene glycol)";
  constant Modelica.Units.SI.Time never = 1e9 "An event that does not happen";

  model Racks "A training job's heat, most of it to the cold plates; the hottest GPU's cold plate; a rack isolated for a leak"
    extends AILiquidCooling.Physical.GPURacks;
    parameter Modelica.Units.SI.Time job_start = 3600 "The training job starts at 1 h, from idle";
    parameter Modelica.Units.SI.Time ramp = 300 "From idle to full power";
    parameter Modelica.Units.SI.TemperatureDifference design_rise = 10 "Coolant warming the cold plates are rated at";
    parameter Modelica.Units.SI.MassFlowRate nominal_flow = liquid_fraction * rack_count * rack_power / (CoolingPhysics.cp * design_rise) "Flow the cold plates are rated at";
    parameter Modelica.Units.SI.Time leak_at = CoolingPhysics.never "A rack leaks";
    parameter Modelica.Units.SI.Time isolation_time = 8.5 "From the alarm to the valves closed (the leak isolation chain)";
    Modelica.Blocks.Sources.RealExpression heatToCoolant(y=to_liquid) annotation(Placement(transformation(origin={8,20}, extent={{-14,-5},{14,5}})));
    Modelica.Thermal.HeatTransfer.Sources.PrescribedHeatFlow toCoolant "Heat the cold plates give the coolant" annotation(Placement(transformation(origin={40,20}, extent={{-7,-7},{7,7}})));
    Modelica.Blocks.Sources.RealExpression hottestGpu(y=cold_plates.T + rise / 2 + gpu_power * load * cap / (cold_plate_conductance * flow_ratio ^ 0.8)) "The hottest GPU sits at the return end; a cold plate conducts less at a lower flow" annotation(Placement(transformation(origin={-5,-20}, extent={{-22,-5},{22,5}})));
    Modelica.Blocks.Continuous.FirstOrder sensor(T=2, initType=Modelica.Blocks.Types.Init.InitialOutput, y_start=310) "The GPU's temperature sensor, read every couple of seconds" annotation(Placement(transformation(origin={38,-20}, extent={{-6,-6},{6,6}})));
    Real load(unit="1") = idle_load + (1 - idle_load) * min(max((time - job_start) / ramp, 0), 1) "Share of the GPUs' power the job uses";
    Real cap(unit="1") = min(max(power_cap, 0.5), 1) "Share of the power the scheduler allows";
    Real isolated(unit="1") = if time >= leak_at + isolation_time and time < leak_at + shutdown_delay then 1 else 0 "A rack isolated, still powered";
    Real powered = if time < leak_at + shutdown_delay then rack_count else rack_count - 1 "Racks with power";
    Real cooled = powered - isolated "Racks the coolant reaches";
    Modelica.Units.SI.Power it_power = powered * rack_power * load * cap "Power of the racks";
    Modelica.Units.SI.HeatFlowRate to_liquid = liquid_fraction * cooled * rack_power * load * cap "Heat to the cold plates";
    Modelica.Units.SI.Energy it_energy(start=0, fixed=true) "Energy the racks used";
    Modelica.Units.SI.MassFlowRate massFlow = max(coolant_flow, 0.05 * nominal_flow) "Coolant through the racks";
    Real flow_ratio(unit="1") = (massFlow / max(cooled, 1)) / (nominal_flow / rack_count) "Flow per rack against its rating";
    Modelica.Units.SI.TemperatureDifference rise = to_liquid / (massFlow * CoolingPhysics.cp) "Coolant warming across the racks";
    Modelica.Units.SI.Temperature isolated_gpu_temperature(start=310, fixed=true) "The hottest GPU of the leaking rack: it follows the others, then heats once its coolant stops";
  equation
    connect(heatToCoolant.y, toCoolant.Q_flow) annotation(Line(points={{23.4,20}, {33,20}}, color={0,0,127}));
    connect(toCoolant.port, cold_plates) annotation(Line(points={{47,20}, {61,20}}, color={191,0,0}));
    connect(hottestGpu.y, sensor.u) annotation(Line(points={{19.2,-20}, {30.8,-20}}, color={0,0,127}));
    connect(sensor.y, gpu_temperature) annotation(Line(points={{44.6,-20}, {61,-20}}, color={0,0,127}));
    der(it_energy) = it_power;
    der(isolated_gpu_temperature) = if time < leak_at + isolation_time then (hottestGpu.y - isolated_gpu_temperature) / 10
      elseif time < leak_at + shutdown_delay then gpu_power * load * cap / gpu_capacity
      else -(isolated_gpu_temperature - cold_plates.T) / 300;
  end Racks;

  model CDU "The racks' coolant, its pumps and the plate exchanger to the facility water"
    extends AILiquidCooling.Physical.CoolantDistributionUnit;
    parameter Modelica.Units.SI.Time pump_failure_at = CoolingPhysics.never "A pump stops";
    parameter Modelica.Units.SI.Temperature coolant_start = 305.15 "Coolant at the start of the day";
    Modelica.Thermal.HeatTransfer.Components.HeatCapacitor coolant(C=coolant_capacity, T(start=coolant_start, fixed=true)) "The racks' loop, at its mean temperature" annotation(Placement(transformation(origin={-35,14}, extent={{-9,-9},{9,9}})));
    Modelica.Blocks.Sources.RealExpression withdrawn(y=-exchanged) annotation(Placement(transformation(origin={-5,-14}, extent={{-12,-5},{12,5}}, rotation=180)));
    Modelica.Thermal.HeatTransfer.Sources.PrescribedHeatFlow fromCoolant annotation(Placement(transformation(origin={-35,-14}, extent={{-7,-7},{7,7}}, rotation=180)));
    Modelica.Blocks.Sources.RealExpression given(y=exchanged) "Plate exchanger" annotation(Placement(transformation(origin={8,26.7}, extent={{-12,-5},{12,5}})));
    Modelica.Thermal.HeatTransfer.Sources.PrescribedHeatFlow toPrimary annotation(Placement(transformation(origin={40,26.7}, extent={{-7,-7},{7,7}})));
    Modelica.Blocks.Sources.RealExpression flowOut(y=massFlow) annotation(Placement(transformation(origin={30,0}, extent={{-12,-5},{12,5}})));
    Modelica.Blocks.Sources.RealExpression supplyOut(y=coolant.T - exchanged / (2 * max(massFlow, 1e-3) * CoolingPhysics.cp)) "The coolant leaves the exchanger colder than its mean by half its warming" annotation(Placement(transformation(origin={30,-26.7}, extent={{-16,-5},{16,5}})));
    Real running = if time < pump_failure_at then pump_count else pump_count - 1 "Pumps that can run";
    Modelica.Units.SI.MassFlowRate massFlow = pump_flow * max(min(running, needed_pumps), 0) "The spare pump only replaces a stopped one";
    Modelica.Units.SI.ThermalConductance conductance = hx_conductance * (max(massFlow, 1e-3) / (pump_flow * needed_pumps)) ^ 0.8 "The exchanger conducts less at a lower flow";
    Modelica.Units.SI.HeatFlowRate exchanged = conductance * (coolant.T - primary.T) "Heat given to the facility water";
    Modelica.Units.SI.Power electrical = pump_power * max(min(running, needed_pumps), 0) "Pumps' electricity";
    Modelica.Units.SI.Energy energy(start=0, fixed=true);
  equation
    connect(secondary, coolant.port) annotation(Line(points={{-61,0}, {-35,0}, {-35,5}}, color={191,0,0}));
    connect(withdrawn.y, fromCoolant.Q_flow) annotation(Line(points={{-18.2,-14}, {-28,-14}}, color={0,0,127}));
    connect(fromCoolant.port, coolant.port) annotation(Line(points={{-42,-14}, {-50,-14}, {-50,0}, {-35,0}, {-35,5}}, color={191,0,0}));
    connect(given.y, toPrimary.Q_flow) annotation(Line(points={{21.2,26.7}, {33,26.7}}, color={0,0,127}));
    connect(toPrimary.port, primary) annotation(Line(points={{47,26.7}, {61,26.7}}, color={191,0,0}));
    connect(flowOut.y, coolant_flow) annotation(Line(points={{43.2,0}, {61,0}}, color={0,0,127}));
    connect(supplyOut.y, supply_temperature) annotation(Line(points={{47.6,-26.7}, {61,-26.7}}, color={0,0,127}));
    der(energy) = electrical;
  end CDU;

  model FacilityWater "The facility water loop, one thermal mass (a buffer tank adds to it)"
    extends AILiquidCooling.Physical.FacilityWaterLoop;
    Modelica.Thermal.HeatTransfer.Components.HeatCapacitor water(C=water_capacity, T(start=300.15, fixed=true)) annotation(Placement(transformation(origin={0,14}, extent={{-10,-10},{10,10}})));
  equation
    connect(from_cdu, water.port) annotation(Line(points={{-61,0}, {0,0}, {0,4}}, color={191,0,0}));
    connect(to_coolers, water.port) annotation(Line(points={{61,0}, {0,0}, {0,4}}, color={191,0,0}));
  end FacilityWater;

  model Reject "Dry coolers whose fans follow the command, a chiller trimming above full fan speed; both stop in a grid outage"
    extends AILiquidCooling.Physical.DryCoolersAndChiller;
    parameter Modelica.Units.SI.TemperatureDifference condenser_approach = 15 "Condensing above the outdoor air";
    parameter Modelica.Units.SI.TemperatureDifference evaporator_approach = 10 "Evaporating below the facility water";
    parameter Modelica.Units.SI.Time outage_at = CoolingPhysics.never "The grid is lost";
    Modelica.Blocks.Sources.RealExpression removed(y=-(free + chilled)) annotation(Placement(transformation(origin={-8,20}, extent={{-14,-5},{14,5}}, rotation=180)));
    Modelica.Thermal.HeatTransfer.Sources.PrescribedHeatFlow fromWater annotation(Placement(transformation(origin={-40,20}, extent={{-7,-7},{7,7}}, rotation=180)));
    Modelica.Blocks.Sources.RealExpression rejected(y=free + chilled + chilled / cop) "With the compressor's work" annotation(Placement(transformation(origin={8,0}, extent={{-16,-5},{16,5}})));
    Modelica.Thermal.HeatTransfer.Sources.PrescribedHeatFlow toAir annotation(Placement(transformation(origin={40,0}, extent={{-7,-7},{7,7}})));
    Boolean fansPowered = time < outage_at or time >= outage_at + generator_start "On the grid, or on the generator";
    Boolean chillerPowered = time < outage_at or time >= outage_at + chiller_restart "The compressor waits out its restart";
    Real fan(unit="1") = if fansPowered then min(max(command, 0.1), 1) else 0 "Fan speed: command 0 to 1";
    Real trim(unit="1") = if chillerPowered then min(max(command - 1, 0), 1) else 0 "Chiller load: command 1 to 2";
    Modelica.Units.SI.HeatFlowRate free = cooler_conductance * fan * max(water.T - air.T, 0) "Free cooling: the air must be colder than the water";
    Modelica.Units.SI.HeatFlowRate chilled = chiller_capacity * trim;
    Real cop(unit="1") = chiller_efficiency * (water.T - evaporator_approach) / max(air.T + condenser_approach - (water.T - evaporator_approach), 5) "Chiller: a share of Carnot between its evaporator and condenser";
    Modelica.Units.SI.Power electrical = fan_power * fan ^ 3 + chilled / cop "Fans and compressor";
    Modelica.Units.SI.Energy energy(start=0, fixed=true);
  equation
    connect(removed.y, fromWater.Q_flow) annotation(Line(points={{-23.4,20}, {-33,20}}, color={0,0,127}));
    connect(fromWater.port, water) annotation(Line(points={{-47,20}, {-61,20}}, color={191,0,0}));
    connect(rejected.y, toAir.Q_flow) annotation(Line(points={{25.6,0}, {33,0}}, color={0,0,127}));
    connect(toAir.port, air) annotation(Line(points={{47,0}, {61,0}}, color={191,0,0}));
    der(energy) = electrical;
  end Reject;

  model Weather "The design day: warmest at 15 h, at the design temperature"
    extends AILiquidCooling.Physical.OutdoorAir;
    Modelica.Units.SI.Temperature T = design_temperature - daily_swing + daily_swing * cos(2 * Modelica.Constants.pi * (time - 15 * 3600) / 86400) "Outdoor air";
    Modelica.Blocks.Sources.RealExpression daily(y=T) annotation(Placement(transformation(origin={10,0}, extent={{-14,-5},{14,5}}, rotation=180)));
    Modelica.Thermal.HeatTransfer.Sources.PrescribedTemperature outdoor annotation(Placement(transformation(origin={-30,0}, extent={{-8,-8},{8,8}}, rotation=180)));
  equation
    connect(daily.y, outdoor.T) annotation(Line(points={{-5.4,0}, {-20.4,0}}, color={0,0,127}));
    connect(outdoor.port, air) annotation(Line(points={{-38,0}, {-61,0}}, color={191,0,0}));
  end Weather;

  model Control "A PI on the coolant supply (fans, then chiller); a PI capping the GPUs above their limit"
    extends AILiquidCooling.Behaviour.CoolingController.SupplyTemperatureControl;
    Modelica.Blocks.Sources.Constant target(k=setpoint) annotation(Placement(transformation(origin={-30,2}, extent={{-6,-6},{6,6}})));
    Modelica.Blocks.Continuous.LimPID cooling(controllerType=Modelica.Blocks.Types.SimpleController.PI, k=0.5, Ti=600, yMax=2, yMin=0, initType=Modelica.Blocks.Types.Init.InitialOutput, y_start=0.5) "0 to 1: fans; 1 to 2: chiller" annotation(Placement(transformation(origin={10,20}, extent={{-8,-8},{8,8}})));
    Modelica.Blocks.Sources.Constant limit(k=cap_above) annotation(Placement(transformation(origin={-30,-20}, extent={{-6,-6},{6,6}})));
    Modelica.Blocks.Continuous.LimPID capping(controllerType=Modelica.Blocks.Types.SimpleController.PI, k=0.2, Ti=120, yMax=1, yMin=0.5, initType=Modelica.Blocks.Types.Init.InitialOutput, y_start=1) "Power allowed: 1 until a GPU passes its limit" annotation(Placement(transformation(origin={10,-20}, extent={{-8,-8},{8,8}})));
  equation
    // Warmer than the setpoint: more cooling.
    connect(supply_temperature, cooling.u_s) annotation(Line(points={{-61,20}, {0.4,20}}, color={0,0,127}));
    connect(target.y, cooling.u_m) annotation(Line(points={{-23.4,2}, {10,2}, {10,10.4}}, color={0,0,127}));
    connect(cooling.y, command) annotation(Line(points={{18.8,20}, {61,20}}, color={0,0,127}));
    connect(limit.y, capping.u_s) annotation(Line(points={{-23.4,-20}, {0.4,-20}}, color={0,0,127}));
    connect(gpu_temperature, capping.u_m) annotation(Line(points={{-61,-20}, {-45,-20}, {-45,-38}, {10,-38}, {10,-29.6}}, color={0,0,127}));
    connect(capping.y, power_cap) annotation(Line(points={{18.8,-20}, {61,-20}}, color={0,0,127}));
  end Control;

  model Controller "The cooling controller, its control implemented"
    extends AILiquidCooling.Physical.CoolingController(redeclare CoolingPhysics.Control supplyTemperatureControl);
  end Controller;

  // ---- The scenarios: one system model per configuration and event ----

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

  model PowerOutage "N+1 pumps: the grid is lost at 14 h; the pumps run on the UPS, the fans on the generator after 30 s, the chiller after its restart"
    extends AILiquidCooling.Systems.N1Pumps(
      redeclare CoolingPhysics.Racks gpuRacks,
      redeclare CoolingPhysics.CDU coolantDistributionUnit,
      redeclare CoolingPhysics.FacilityWater facilityWaterLoop,
      redeclare CoolingPhysics.Reject dryCoolersAndChiller(outage_at=14 * 3600),
      redeclare CoolingPhysics.Weather outdoorAir,
      redeclare CoolingPhysics.Controller coolingController);
    Real ppue(unit="1") = 1 + (coolantDistributionUnit.energy + dryCoolersAndChiller.energy) / max(gpuRacks.it_energy, 1) "Partial PUE of the cooling";
    annotation(experiment(StopTime=86400, Interval=10));
  end PowerOutage;

  model LeakInARack "N+1 pumps: a rack leaks at 14 h; its valves close, the scheduler powers it off, the others train on"
    extends AILiquidCooling.Systems.N1Pumps(
      redeclare CoolingPhysics.Racks gpuRacks(leak_at=14 * 3600),
      redeclare CoolingPhysics.CDU coolantDistributionUnit,
      redeclare CoolingPhysics.FacilityWater facilityWaterLoop,
      redeclare CoolingPhysics.Reject dryCoolersAndChiller,
      redeclare CoolingPhysics.Weather outdoorAir,
      redeclare CoolingPhysics.Controller coolingController);
    Real ppue(unit="1") = 1 + (coolantDistributionUnit.energy + dryCoolersAndChiller.energy) / max(gpuRacks.it_energy, 1) "Partial PUE of the cooling";
    annotation(experiment(StopTime=86400, Interval=1));
  end LeakInARack;

  model NextGenerationToday "Racks of 200 kW on the current plant, on the design summer day"
    extends AILiquidCooling.Systems.NextGenerationRacksOnTheCurrentPlant(
      redeclare CoolingPhysics.Racks gpuRacks,
      redeclare CoolingPhysics.CDU coolantDistributionUnit,
      redeclare CoolingPhysics.FacilityWater facilityWaterLoop,
      redeclare CoolingPhysics.Reject dryCoolersAndChiller,
      redeclare CoolingPhysics.Weather outdoorAir,
      redeclare CoolingPhysics.Controller coolingController);
    Real ppue(unit="1") = 1 + (coolantDistributionUnit.energy + dryCoolersAndChiller.energy) / max(gpuRacks.it_energy, 1) "Partial PUE of the cooling";
    annotation(experiment(StopTime=86400, Interval=60));
  end NextGenerationToday;

  model NextGenerationHall "Racks of 200 kW with a high flow CDU, larger dry coolers and a 1.6 MW chiller, on the design summer day"
    extends AILiquidCooling.Systems.NextGenerationHall(
      redeclare CoolingPhysics.Racks gpuRacks,
      redeclare CoolingPhysics.CDU coolantDistributionUnit,
      redeclare CoolingPhysics.FacilityWater facilityWaterLoop,
      redeclare CoolingPhysics.Reject dryCoolersAndChiller,
      redeclare CoolingPhysics.Weather outdoorAir,
      redeclare CoolingPhysics.Controller coolingController);
    Real ppue(unit="1") = 1 + (coolantDistributionUnit.energy + dryCoolersAndChiller.energy) / max(gpuRacks.it_energy, 1) "Partial PUE of the cooling";
    annotation(experiment(StopTime=86400, Interval=60));
  end NextGenerationHall;
  annotation(uses(Modelica(version="4.1.0")));
end CoolingPhysics;
