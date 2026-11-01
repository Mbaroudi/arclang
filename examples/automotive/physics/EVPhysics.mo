// Hand-written physics of examples/automotive/ev_range_simulation.arc.
// It extends the package `arclang export -f modelica` generates
// (EVRangeSimulation), which holds the structure and is never edited:
// each model below extends a generated partial model and adds equations,
// and the two systems redeclare every part of a generated configuration.
// Every component is placed and every connection drawn, so OMEdit and Modex
// show each model as a diagram of Modelica Standard Library components.
within ;
package EVPhysics "Physics of the EV range simulation"
  model Battery "Pack with its open-circuit voltage curve, an internal resistance that grows in the cold, and its thermal mass"
    extends EVRangeSimulation.Physical.BatteryPack;
    parameter Modelica.Units.SI.Temperature ambient = 263.15 "The pack soaks at the outside temperature before the trip";
    parameter Real activation(unit="K") = 2500 "Arrhenius slope of the resistance: x3 at -10 °C, x4.4 at -20 °C, x1 at 25 °C";
    parameter Modelica.Units.SI.ElectricCharge capacity = energy / voltage "Charge of the full pack";
    Modelica.Blocks.Sources.RealExpression stateOfCharge(y=soc) annotation(Placement(transformation(origin={-60,32}, extent={{-6,-4},{6,4}})));
    Modelica.Blocks.Tables.CombiTable1Ds ocv(table=[0, 0.82; 0.05, 0.88; 0.1, 0.91; 0.2, 0.94; 0.5, 0.985; 0.8, 1.03; 1, 1.06], smoothness=Modelica.Blocks.Types.Smoothness.ContinuousDerivative) "Open-circuit voltage per nominal voltage, by state of charge" annotation(Placement(transformation(origin={-60,14}, extent={{-6,-6},{6,6}}, rotation=270)));
    Modelica.Blocks.Math.Gain nominal(k=voltage) annotation(Placement(transformation(origin={-60,-6}, extent={{-5,-5},{5,5}}, rotation=270)));
    Modelica.Electrical.Analog.Sources.SignalVoltage cells "Open-circuit voltage" annotation(Placement(transformation(origin={-35,0}, extent={{-10,-10},{10,10}}, rotation=90)));
    Modelica.Blocks.Sources.RealExpression resistanceNow(y=internal_resistance * exp(activation * (1 / packHeat.T - 1 / 298.15))) annotation(Placement(transformation(origin={-5,40}, extent={{-8,-4},{8,4}})));
    Modelica.Electrical.Analog.Basic.VariableResistor resistance(useHeatPort=true, i(start=0), v(start=0)) "Internal resistance, its losses heating the pack" annotation(Placement(transformation(origin={-5,20}, extent={{-8,-8},{8,8}})));
    Modelica.Electrical.Analog.Sensors.CurrentSensor current annotation(Placement(transformation(origin={22,20}, extent={{-6,-6},{6,6}})));
    Modelica.Blocks.Sources.RealExpression auxiliaryCurrent(y=auxiliary_load / max(hv_p.v - hv_n.v, 1)) annotation(Placement(transformation(origin={30,0}, extent={{-8,-4},{8,4}})));
    Modelica.Electrical.Analog.Sources.SignalCurrent auxiliaries(v(start=voltage)) "Lights, pumps, electronics" annotation(Placement(transformation(origin={48,0}, extent={{-7,-7},{7,7}}, rotation=270)));
    Modelica.Thermal.HeatTransfer.Components.HeatCapacitor packHeat(C=thermal_mass, T(start=ambient, fixed=true)) "Thermal mass of the cells" annotation(Placement(transformation(origin={-5,-20}, extent={{-8,-8},{8,8}})));
    Modelica.Thermal.HeatTransfer.Components.ThermalConductor toAir(G=cooling) annotation(Placement(transformation(origin={18,-34}, extent={{-6,-6},{6,6}})));
    Modelica.Thermal.HeatTransfer.Sources.FixedTemperature outside(T=ambient) annotation(Placement(transformation(origin={40,-34}, extent={{-6,-6},{6,6}}, rotation=180)));
    Modelica.Electrical.Analog.Basic.Ground ground annotation(Placement(transformation(origin={58,-36}, extent={{-5,-5},{5,5}})));
    Modelica.Units.SI.Power power = (hv_p.v - hv_n.v) * current.i "Power delivered, auxiliaries included";
    Modelica.Units.SI.Energy drawn(start=0, fixed=true) "Energy drawn, auxiliaries included";
    Modelica.Units.SI.ElectricCharge charge(start=0, fixed=true) "Charge drawn";
    Real soc(unit="1") "State of charge";
    Modelica.Units.NonSI.Temperature_degC packTemperature = Modelica.Units.Conversions.to_degC(packHeat.T);
    Modelica.Units.SI.Power losses = resistance.LossPower "Heat of the internal resistance";
  equation
    connect(stateOfCharge.y, ocv.u) annotation(Line(points={{-53.4,32}, {-50,32}, {-50,24}, {-60,24}, {-60,21.2}}, color={0,0,127}));
    connect(ocv.y[1], nominal.u) annotation(Line(points={{-60,7.4}, {-60,0}}, color={0,0,127}));
    connect(nominal.y, cells.v) annotation(Line(points={{-60,-11.5}, {-60,-16}, {-50,-16}, {-50,0}, {-47,0}}, color={0,0,127}));
    connect(cells.p, resistance.p) annotation(Line(points={{-35,10}, {-35,20}, {-13,20}}, color={0,0,255}));
    connect(resistanceNow.y, resistance.R) annotation(Line(points={{3.8,40}, {8,40}, {8,32}, {-5,32}, {-5,28}}, color={0,0,127}));
    connect(resistance.n, current.p) annotation(Line(points={{3,20}, {16,20}}, color={0,0,255}));
    connect(current.n, hv_p) annotation(Line(points={{28,20}, {61,20}}, color={0,0,255}));
    connect(auxiliaries.p, hv_p) annotation(Line(points={{48,7}, {48,20}, {61,20}}, color={0,0,255}));
    connect(auxiliaries.n, hv_n) annotation(Line(points={{48,-7}, {48,-20}, {61,-20}}, color={0,0,255}));
    connect(auxiliaryCurrent.y, auxiliaries.i) annotation(Line(points={{38.8,0}, {39.6,0}}, color={0,0,127}));
    connect(cells.n, hv_n) annotation(Line(points={{-35,-10}, {-35,-20}, {61,-20}}, color={0,0,255}));
    connect(ground.p, hv_n) annotation(Line(points={{58,-31}, {58,-20}, {61,-20}}, color={0,0,255}));
    connect(resistance.heatPort, packHeat.port) annotation(Line(points={{-5,12}, {-5,8}, {-20,8}, {-20,-30}, {-5,-30}, {-5,-28}}, color={191,0,0}));
    connect(packHeat.port, toAir.port_a) annotation(Line(points={{-5,-28}, {-5,-34}, {12,-34}}, color={191,0,0}));
    connect(toAir.port_b, outside.port) annotation(Line(points={{24,-34}, {34,-34}}, color={191,0,0}));
    der(drawn) = power;
    der(charge) = current.i;
    soc = 1 - charge / capacity;
    // An empty pack stops the vehicle: the trip ends there, short of its length.
    when soc <= 0 then
      terminate("the battery is empty");
    end when;
  end Battery;

  model CabinHeater "PTC heater keeping the cabin at its setpoint against the cold, drawing its power from the HV bus"
    extends EVRangeSimulation.Physical.CabinHeater;
    parameter Modelica.Units.SI.Temperature ambient = 263.15 "The cabin starts at the outside temperature";
    Modelica.Blocks.Sources.Constant comfort(k=setpoint) annotation(Placement(transformation(origin={-30,32}, extent={{-6,-6},{6,6}})));
    Modelica.Blocks.Continuous.LimPID thermostat(controllerType=Modelica.Blocks.Types.SimpleController.PI, k=1000, Ti=300, yMax=max_power, yMin=0) "Heating power" annotation(Placement(transformation(origin={-5,32}, extent={{-7,-7},{7,7}})));
    Modelica.Thermal.HeatTransfer.Sources.PrescribedHeatFlow heater annotation(Placement(transformation(origin={20,32}, extent={{-6,-6},{6,6}})));
    Modelica.Thermal.HeatTransfer.Components.HeatCapacitor cabin(C=cabin_capacity, T(start=ambient, fixed=true)) "Air and interior of the cabin" annotation(Placement(transformation(origin={40,10}, extent={{-8,-8},{8,8}})));
    Modelica.Thermal.HeatTransfer.Sensors.TemperatureSensor cabinTemperature annotation(Placement(transformation(origin={20,-4}, extent={{-5,-5},{5,5}}, rotation=180)));
    Modelica.Thermal.HeatTransfer.Components.ThermalConductor body(G=cabin_conductance) "Windows, doors, roof, floor" annotation(Placement(transformation(origin={40,-22}, extent={{-6,-6},{6,6}}, rotation=270)));
    Modelica.Thermal.HeatTransfer.Sources.FixedTemperature outside(T=ambient) annotation(Placement(transformation(origin={40,-40}, extent={{-5,-5},{5,5}}, rotation=90)));
    Modelica.Blocks.Sources.RealExpression drawnCurrent(y=heater.Q_flow / max(hv_p.v - hv_n.v, 1)) "A PTC heater turns all it draws into heat" annotation(Placement(transformation(origin={-48,0}, extent={{-8,-4},{8,4}})));
    Modelica.Electrical.Analog.Sources.SignalCurrent load annotation(Placement(transformation(origin={-30,0}, extent={{-7,-7},{7,7}}, rotation=270)));
    Modelica.Units.SI.Power heating = heater.Q_flow "Heating power, drawn from the bus";
  equation
    connect(comfort.y, thermostat.u_s) annotation(Line(points={{-23.4,32}, {-13.4,32}}, color={0,0,127}));
    connect(thermostat.y, heater.Q_flow) annotation(Line(points={{2.7,32}, {14,32}}, color={0,0,127}));
    connect(heater.port, cabin.port) annotation(Line(points={{26,32}, {30,32}, {30,2}, {40,2}}, color={191,0,0}));
    connect(cabin.port, cabinTemperature.port) annotation(Line(points={{40,2}, {40,-4}, {25,-4}}, color={191,0,0}));
    connect(cabinTemperature.T, thermostat.u_m) annotation(Line(points={{14.5,-4}, {-5,-4}, {-5,23.6}}, color={0,0,127}));
    connect(cabin.port, body.port_a) annotation(Line(points={{40,2}, {40,-16}}, color={191,0,0}));
    connect(body.port_b, outside.port) annotation(Line(points={{40,-28}, {40,-35}}, color={191,0,0}));
    connect(drawnCurrent.y, load.i) annotation(Line(points={{-39.2,0}, {-38.4,0}}, color={0,0,127}));
    connect(load.p, hv_p) annotation(Line(points={{-30,7}, {-30,20}, {-61,20}}, color={0,0,255}));
    connect(load.n, hv_n) annotation(Line(points={{-30,-7}, {-30,-20}, {-61,-20}}, color={0,0,255}));
  end CabinHeater;

  model Drive "Torque-controlled motor, limited to its power, and reduction gear, drawing its power from the HV bus"
    extends EVRangeSimulation.Physical.DriveUnit;
    Modelica.Blocks.Sources.RealExpression torqueLimit(y=max_power / max(abs(rotor.w), 1)) "The torque the motor's power allows at this speed" annotation(Placement(transformation(origin={-50,-6}, extent={{-8,-4},{8,4}})));
    Modelica.Blocks.Math.Gain negative(k=-1) annotation(Placement(transformation(origin={-50,-45}, extent={{-4,-4},{4,4}})));
    Modelica.Blocks.Nonlinear.VariableLimiter limiter annotation(Placement(transformation(origin={-45,-26.7}, extent={{-5,-5},{5,5}})));
    Modelica.Mechanics.Rotational.Sources.Torque motor annotation(Placement(transformation(origin={-30,-26.7}, extent={{-7,-7},{7,7}})));
    Modelica.Mechanics.Rotational.Components.Inertia rotor(J=0.05) annotation(Placement(transformation(origin={-5,-26.7}, extent={{-7,-7},{7,7}})));
    Modelica.Mechanics.Rotational.Components.IdealGear gear(ratio=gear_ratio) annotation(Placement(transformation(origin={20,-26.7}, extent={{-7,-7},{7,7}})));
    Modelica.Electrical.Analog.Sources.SignalCurrent load "Current drawn from the bus" annotation(Placement(transformation(origin={-30,13.35}, extent={{-7,-7},{7,7}}, rotation=270)));
    Modelica.Units.SI.Power mechanical "Shaft power of the motor";
    Modelica.Units.SI.Power electrical "Power drawn from the bus";
  equation
    connect(torque_cmd, limiter.u) annotation(Line(points={{-61,-26.7}, {-51,-26.7}}, color={0,0,127}));
    connect(torqueLimit.y, limiter.limit1) annotation(Line(points={{-41.2,-6}, {-38,-6}, {-38,-16}, {-54,-16}, {-54,-22.7}, {-51,-22.7}}, color={0,0,127}));
    connect(torqueLimit.y, negative.u) annotation(Line(points={{-41.2,-6}, {-38,-6}, {-38,-16}, {-58,-16}, {-58,-45}, {-54.8,-45}}, color={0,0,127}));
    connect(negative.y, limiter.limit2) annotation(Line(points={{-45.6,-45}, {-42,-45}, {-42,-36}, {-54,-36}, {-54,-30.7}, {-51,-30.7}}, color={0,0,127}));
    connect(limiter.y, motor.tau) annotation(Line(points={{-39.5,-26.7}, {-38.4,-26.7}}, color={0,0,127}));
    connect(motor.flange, rotor.flange_a) annotation(Line(points={{-23,-26.7}, {-12,-26.7}}, color={0,0,0}));
    connect(rotor.flange_b, gear.flange_a) annotation(Line(points={{2,-26.7}, {13,-26.7}}, color={0,0,0}));
    connect(gear.flange_b, shaft) annotation(Line(points={{27,-26.7}, {40,-26.7}, {40,0}, {61,0}}, color={0,0,0}));
    connect(load.p, hv_p) annotation(Line(points={{-30,20.35}, {-30,26.7}, {-61,26.7}}, color={0,0,255}));
    connect(load.n, hv_n) annotation(Line(points={{-30,6.35}, {-30,0}, {-61,0}}, color={0,0,255}));
    mechanical = motor.tau * rotor.w;
    electrical = noEvent(if mechanical >= 0 then mechanical / efficiency else mechanical * efficiency);
    load.i = electrical / max(load.v, 1);
  end Drive;

  model Body "Longitudinal dynamics: mass, aerodynamic drag and rolling resistance"
    extends EVRangeSimulation.Physical.VehicleBody;
    constant Modelica.Units.SI.Acceleration g = Modelica.Constants.g_n;
    parameter Modelica.Units.SI.Density rho = 101325 / (287.05 * ambient) "Air density at the ambient temperature";
    Modelica.Mechanics.Rotational.Components.IdealRollingWheel tyres(radius=wheel_radius) annotation(Placement(transformation(origin={-35,0}, extent={{-8,-8},{8,8}})));
    Modelica.Mechanics.Translational.Components.Mass car(m=mass, s(start=0, fixed=true), v(start=0, fixed=true)) annotation(Placement(transformation(origin={0,0}, extent={{-8,-8},{8,8}})));
    Modelica.Mechanics.Translational.Sources.Force resistance "Aerodynamic drag and rolling resistance" annotation(Placement(transformation(origin={35,0}, extent={{-8,-8},{8,8}}, rotation=180)));
    Modelica.Units.SI.Length distance = car.s "Distance driven";
  equation
    connect(wheel, tyres.flangeR) annotation(Line(points={{-61,0}, {-43,0}}, color={0,0,0}));
    connect(tyres.flangeT, car.flange_a) annotation(Line(points={{-27,0}, {-8,0}}, color={0,127,0}));
    connect(car.flange_b, resistance.flange) annotation(Line(points={{8,0}, {27,0}}, color={0,127,0}));
    resistance.f = -(0.5 * rho * drag_area * car.v * abs(car.v) + rolling_resistance * mass * g * tanh(car.v / 0.5));
    speed = car.v;
  end Body;

  model SpeedControl "Cruise control: a PI on the vehicle speed, the motor torque limited"
    extends EVRangeSimulation.Behaviour.VehicleControlUnit.SpeedController;
    parameter Modelica.Units.SI.Torque tauMax = 350 "Motor torque limit";
    Modelica.Blocks.Sources.Constant setpoint(k=target_speed) annotation(Placement(transformation(origin={-30,25}, extent={{-8,-8},{8,8}})));
    Modelica.Blocks.Continuous.LimPID pi(controllerType=Modelica.Blocks.Types.SimpleController.PI, k=200, Ti=10, yMax=tauMax, yMin=-tauMax) annotation(Placement(transformation(origin={10,0}, extent={{-10,-10},{10,10}})));
  equation
    connect(setpoint.y, pi.u_s) annotation(Line(points={{-21.2,25}, {-12,25}, {-12,0}, {-2,0}}, color={0,0,127}));
    connect(speed, pi.u_m) annotation(Line(points={{-61,0}, {-45,0}, {-45,-22}, {10,-22}, {10,-12}}, color={0,0,127}));
    connect(pi.y, torque_cmd) annotation(Line(points={{21,0}, {61,0}}, color={0,0,127}));
  end SpeedControl;

  model ControlUnit "The vehicle control unit, its speed controller implemented"
    extends EVRangeSimulation.Physical.VehicleControlUnit(redeclare EVPhysics.SpeedControl speedController);
  end ControlUnit;

  model CityRange "Configuration City (standard pack) on the winter highway"
    extends EVRangeSimulation.Systems.City(
      redeclare EVPhysics.Battery batteryPack(ambient=vehicleBody.ambient),
      redeclare EVPhysics.CabinHeater cabinHeater(ambient=vehicleBody.ambient),
      redeclare EVPhysics.Drive driveUnit,
      redeclare EVPhysics.Body vehicleBody,
      redeclare EVPhysics.ControlUnit vehicleControlUnit);
    parameter Modelica.Units.SI.Length trip = 300e3 "Length of the trip: the simulation ends there";
  equation
    when vehicleBody.distance >= trip then
      terminate("the trip is driven");
    end when;
    // The 5 h of the test case, for a run from OMEdit (arclang verify passes its own stop_time).
    annotation(experiment(StopTime=18000, Interval=10));
  end CityRange;

  model TouringRange "Configuration Touring (long range pack) on the winter highway"
    extends EVRangeSimulation.Systems.Touring(
      redeclare EVPhysics.Battery batteryPack(ambient=vehicleBody.ambient),
      redeclare EVPhysics.CabinHeater cabinHeater(ambient=vehicleBody.ambient),
      redeclare EVPhysics.Drive driveUnit,
      redeclare EVPhysics.Body vehicleBody,
      redeclare EVPhysics.ControlUnit vehicleControlUnit);
    parameter Modelica.Units.SI.Length trip = 300e3 "Length of the trip: the simulation ends there";
  equation
    when vehicleBody.distance >= trip then
      terminate("the trip is driven");
    end when;
    // The 5 h of the test case, for a run from OMEdit (arclang verify passes its own stop_time).
    annotation(experiment(StopTime=18000, Interval=10));
  end TouringRange;
  annotation(uses(Modelica(version="4.1.0")));
end EVPhysics;
