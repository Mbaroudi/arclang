// Hand-written physics of examples/automotive/ev_range_simulation.arc.
// It extends the package `arclang export -f modelica` generates
// (EVRangeSimulation), which holds the structure and is never edited:
// each model below extends a generated partial model and adds equations,
// and the two systems redeclare every part of a generated configuration.
// Every component is placed and every connection drawn, so OMEdit and Modex
// show each model as a diagram of Modelica Standard Library components.
within ;
package EVPhysics "Physics of the EV range simulation"
  model Battery "Ideal pack at constant voltage; state of charge from the energy drawn"
    extends EVRangeSimulation.Physical.BatteryPack;
    Modelica.Electrical.Analog.Sources.ConstantVoltage cells(V=voltage) "Open-circuit voltage" annotation(Placement(transformation(origin={0,0}, extent={{-10,-10},{10,10}}, rotation=270)));
    Modelica.Electrical.Analog.Basic.Ground ground annotation(Placement(transformation(origin={30,-36}, extent={{-6,-6},{6,6}})));
    Modelica.Units.SI.Power power "Power delivered at the terminals";
    Modelica.Units.SI.Energy drawn(start=0, fixed=true) "Energy drawn, auxiliaries included";
    Real soc(unit="1") "State of charge";
  equation
    connect(cells.p, hv_p) annotation(Line(points={{0,10}, {0,20}, {61,20}}, color={0,0,255}));
    connect(cells.n, hv_n) annotation(Line(points={{0,-10}, {0,-20}, {61,-20}}, color={0,0,255}));
    connect(ground.p, hv_n) annotation(Line(points={{30,-30}, {30,-20}, {61,-20}}, color={0,0,255}));
    power = -cells.v * cells.i;
    der(drawn) = power + auxiliary_load;
    soc = 1 - drawn / energy;
    // An empty pack stops the vehicle: the trip ends there, short of its length.
    when soc <= 0 then
      terminate("the battery is empty");
    end when;
  end Battery;

  model Drive "Torque-controlled motor and reduction gear, drawing its power from the HV bus"
    extends EVRangeSimulation.Physical.DriveUnit;
    Modelica.Mechanics.Rotational.Sources.Torque motor annotation(Placement(transformation(origin={-30,-26.7}, extent={{-7,-7},{7,7}})));
    Modelica.Mechanics.Rotational.Components.Inertia rotor(J=0.05) annotation(Placement(transformation(origin={-5,-26.7}, extent={{-7,-7},{7,7}})));
    Modelica.Mechanics.Rotational.Components.IdealGear gear(ratio=gear_ratio) annotation(Placement(transformation(origin={20,-26.7}, extent={{-7,-7},{7,7}})));
    Modelica.Electrical.Analog.Sources.SignalCurrent load "Current drawn from the bus" annotation(Placement(transformation(origin={-30,13.35}, extent={{-7,-7},{7,7}}, rotation=270)));
    Modelica.Units.SI.Power mechanical "Shaft power of the motor";
    Modelica.Units.SI.Power electrical "Power drawn from the bus";
  equation
    connect(torque_cmd, motor.tau) annotation(Line(points={{-61,-26.7}, {-38.4,-26.7}}, color={0,0,127}));
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
      redeclare EVPhysics.Battery batteryPack,
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
      redeclare EVPhysics.Battery batteryPack,
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
