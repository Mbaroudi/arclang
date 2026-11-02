// Hand-written physics of examples/building/heat_pump_heating.arc.
// It extends the package `arclang export -f modelica` generates
// (HeatPumpHeating), which holds the structure and is never edited.
within ;
package HeatingPhysics "Physics of the heat pump heating"
  model Space "One thermal mass, the envelope to the outdoor air, the internal gains"
    extends HeatPumpHeating.Physical.LivingSpace;
    Modelica.Thermal.HeatTransfer.Components.HeatCapacitor room(C=heat_capacity, T(start=initial_temperature, fixed=true));
    Modelica.Thermal.HeatTransfer.Components.ThermalConductor walls(G=envelope_conductance);
    Modelica.Thermal.HeatTransfer.Sources.FixedHeatFlow gains(Q_flow=internal_gain, T_ref=293.15, alpha=0) "People, appliances, sun";
  equation
    connect(heating, room.port);
    connect(gains.port, room.port);
    connect(room.port, walls.port_a);
    connect(walls.port_b, envelope);
    temperature = room.T;
  end Space;

  model Weather "The design day: coldest at 3 h, warmest at 15 h"
    extends HeatPumpHeating.Physical.OutdoorAir;
    Modelica.Thermal.HeatTransfer.Sources.PrescribedTemperature outdoor;
    Modelica.Units.SI.Temperature T = design_temperature + daily_swing * cos(2 * Modelica.Constants.pi * (time - 15 * 3600) / 86400) "Outdoor air";
  equation
    outdoor.T = T;
    connect(outdoor.port, air);
    connect(outdoor.port, source);
  end Weather;

  model HeatPump "Capacity and efficiency that fall with the outdoor temperature"
    extends HeatPumpHeating.Physical.HeatPump;
    parameter Modelica.Units.SI.Temperature rated_at = 280.15 "Capacity rated at 7 °C outdoor";
    parameter Real derating(unit="1/K") = 0.015 "Capacity lost per kelvin below the rating point";
    Modelica.Thermal.HeatTransfer.Sources.PrescribedHeatFlow delivered;
    Modelica.Thermal.HeatTransfer.Sources.PrescribedHeatFlow absorbed;
    Modelica.Units.SI.Temperature outdoor = evaporator.T "Air at the outdoor unit";
    Modelica.Units.SI.HeatFlowRate available "Capacity at this outdoor temperature";
    Modelica.Units.SI.HeatFlowRate heat "Heat delivered to the house";
    Real cop(unit="1") "Coefficient of performance";
    Modelica.Units.SI.Power electrical "Electricity drawn";
    Modelica.Units.SI.Energy consumed(start=0, fixed=true) "Electricity used since midnight";
  equation
    available = heating_capacity * max(0.2, 1 - derating * (rated_at - outdoor));
    heat = available * min(max(demand, 0), 1);
    cop = carnot_efficiency * supply_temperature / (supply_temperature - outdoor);
    electrical = heat / cop;
    der(consumed) = electrical;
    delivered.Q_flow = heat;
    absorbed.Q_flow = -(heat - electrical);
    connect(delivered.port, condenser);
    connect(absorbed.port, evaporator);
  end HeatPump;

  model ComfortLoop "A PI on the room temperature; the demand between off and full capacity"
    extends HeatPumpHeating.Behavior.Thermostat.ComfortController;
    Modelica.Blocks.Sources.Constant target(k=setpoint);
    Modelica.Blocks.Continuous.LimPID pi(controllerType=Modelica.Blocks.Types.SimpleController.PI, k=1.5, Ti=1800, yMax=1, yMin=0, initType=Modelica.Blocks.Types.Init.InitialOutput, y_start=0.6);
  equation
    connect(target.y, pi.u_s);
    connect(temperature, pi.u_m);
    connect(pi.y, demand);
  end ComfortLoop;

  model RoomThermostat "The thermostat, its comfort controller implemented"
    extends HeatPumpHeating.Physical.Thermostat(redeclare HeatingPhysics.ComfortLoop comfortController);
  end RoomThermostat;

  model CompactDay "Configuration Compact on the design day"
    extends HeatPumpHeating.Systems.Compact(
      redeclare HeatingPhysics.Space livingSpace,
      redeclare HeatingPhysics.Weather outdoorAir,
      redeclare HeatingPhysics.HeatPump heatPump,
      redeclare HeatingPhysics.RoomThermostat thermostat);
  end CompactDay;

  model StandardDay "Configuration Standard on the design day"
    extends HeatPumpHeating.Systems.Standard(
      redeclare HeatingPhysics.Space livingSpace,
      redeclare HeatingPhysics.Weather outdoorAir,
      redeclare HeatingPhysics.HeatPump heatPump,
      redeclare HeatingPhysics.RoomThermostat thermostat);
  end StandardDay;
  annotation(uses(Modelica(version="4.1.0")));
end HeatingPhysics;
