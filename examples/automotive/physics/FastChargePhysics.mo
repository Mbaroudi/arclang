// Hand-written physics of examples/automotive/battery_fast_charge.arc.
// It extends the package `arclang export -f modelica` generates
// (BatteryFastCharge), which holds the structure and is never edited.
within ;
package FastChargePhysics "Physics of the battery fast charge"
  model Cells "Open-circuit voltage rising with the charge; an internal resistance that heats the cells"
    extends BatteryFastCharge.Physical.CellStack;
    parameter Modelica.Units.SI.SpecificHeatCapacity cp = 1000 "Cells, busbars and housing";
    parameter Modelica.Units.SI.ThermalConductance toAir = 15 "Housing to the ambient air";
    Modelica.Electrical.Analog.Sources.SignalVoltage ocv "Open-circuit voltage";
    Modelica.Electrical.Analog.Basic.Resistor resistance(R=internal_resistance, useHeatPort=true, T_ref=298.15, alpha=0);
    Modelica.Electrical.Analog.Basic.Ground ground;
    Modelica.Thermal.HeatTransfer.Components.HeatCapacitor core(C=mass * cp, T(start=ambient + warm_up, fixed=true)) "Thermal mass of the cells";
    Modelica.Thermal.HeatTransfer.Components.ThermalConductor housing(G=toAir);
    Modelica.Thermal.HeatTransfer.Sources.FixedTemperature air(T=ambient);
    Real soc(unit="1", start=initial_soc, fixed=true) "State of charge";
    Modelica.Units.SI.Power heating = resistance.LossPower "Heat of the internal resistance";
  equation
    ocv.v = voltage * (0.85 + 0.25 * soc);
    connect(plus, resistance.p);
    connect(resistance.n, ocv.p);
    connect(ocv.n, minus);
    connect(ground.p, minus);
    der(soc) = ocv.v * resistance.i / energy;
    connect(resistance.heatPort, core.port);
    connect(core.port, housing.port_a);
    connect(housing.port_b, air.port);
    connect(core.port, heat);
    temperature = core.T;
  end Cells;

  model Charger "Constant power at the limit the battery asks for"
    extends BatteryFastCharge.Physical.FastCharger;
    Modelica.Electrical.Analog.Sources.SignalCurrent source "Current pushed into the pack";
    Modelica.Units.SI.Power power "Power delivered";
    Modelica.Units.SI.Time elapsed = time "Time since the charge started";
  equation
    connect(source.p, dc_n);
    connect(source.n, dc_p);
    power = max_power * limit;
    source.i = power / max(dc_p.v - dc_n.v, 1);
  end Charger;

  model Loop "Cold plate: convection to the coolant, stronger as the pump runs faster"
    extends BatteryFastCharge.Physical.CoolingLoop;
    Modelica.Thermal.HeatTransfer.Components.Convection plate;
    Modelica.Thermal.HeatTransfer.Sources.FixedTemperature coolant(T=coolant_inlet);
    Modelica.Units.SI.HeatFlowRate removed = plate.Q_flow "Heat taken from the cells";
  equation
    plate.Gc = cooling_conductance * max(pump_cmd, 0.1);
    connect(heat, plate.solid);
    connect(plate.fluid, coolant.port);
  end Loop;

  model Regulator "Pump speed up with the temperature; charge power down past the derating threshold"
    extends BatteryFastCharge.Behaviour.BatteryManagementUnit.ThermalRegulator;
  equation
    pump_cmd = min(1, max(0, (temperature - pump_on) / 5));
    limit = min(1, max(0.2, 1 - 0.8 * (temperature - derate_start) / (derate_full - derate_start)));
  end Regulator;

  model ManagementUnit "The battery management unit, its regulator implemented"
    extends BatteryFastCharge.Physical.BatteryManagementUnit(redeclare FastChargePhysics.Regulator thermalRegulator);
  end ManagementUnit;

  model AirCooledCharge "Configuration Air cooled: a charge from 10 % to 80 %"
    extends BatteryFastCharge.Systems.AirCooled(
      redeclare FastChargePhysics.Cells cellStack,
      redeclare FastChargePhysics.Charger fastCharger,
      redeclare FastChargePhysics.Loop coolingLoop,
      redeclare FastChargePhysics.ManagementUnit batteryManagementUnit);
    parameter Real target(unit="1") = 0.8 "The charge ends at this state of charge";
  equation
    when cellStack.soc >= target then
      terminate("the pack is charged");
    end when;
  end AirCooledCharge;

  model ChilledCharge "Configuration Chilled: a charge from 10 % to 80 %"
    extends BatteryFastCharge.Systems.Chilled(
      redeclare FastChargePhysics.Cells cellStack,
      redeclare FastChargePhysics.Charger fastCharger,
      redeclare FastChargePhysics.Loop coolingLoop,
      redeclare FastChargePhysics.ManagementUnit batteryManagementUnit);
    parameter Real target(unit="1") = 0.8 "The charge ends at this state of charge";
  equation
    when cellStack.soc >= target then
      terminate("the pack is charged");
    end when;
  end ChilledCharge;
  annotation(uses(Modelica(version="4.1.0")));
end FastChargePhysics;
