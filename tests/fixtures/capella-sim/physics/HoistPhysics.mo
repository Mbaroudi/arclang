// The executable model of each element of the hoist: each extends the class
// `arclang export -f modelica` generates for its element; ArcLang rolls them
// up along the architecture (CargoHoist.Executable).
within ;
package HoistPhysics "Physics of the cargo hoist"
  constant Modelica.Units.SI.Acceleration g = Modelica.Constants.g_n;

  model Pack "Ideal pack at constant voltage"
    extends CargoHoist.Physical.Battery;
    Modelica.Electrical.Analog.Sources.ConstantVoltage cells(V=voltage);
    Modelica.Electrical.Analog.Basic.Ground ground;
    Modelica.Units.SI.Energy drawn(start=0, fixed=true) "Energy drawn";
    Real soc(unit="1") "State of charge";
  equation
    connect(cells.p, p);
    connect(cells.n, n);
    connect(ground.p, n);
    der(drawn) = -cells.v * cells.i;
    soc = 1 - drawn / energy;
    when soc <= 0 then
      terminate("the battery is empty");
    end when;
  end Pack;

  model Winch "Pulls the cable as commanded; draws the mechanical power over its efficiency, and its standby"
    extends CargoHoist.Physical.Winch;
    Modelica.Mechanics.Translational.Sources.Force pull;
    Modelica.Electrical.Analog.Sources.SignalCurrent load "Current drawn from the pack";
    Modelica.Mechanics.Translational.Sensors.SpeedSensor speed;
    Modelica.Units.SI.Force force "Pull on the cable";
    Modelica.Units.SI.Power electrical "Power drawn from the pack";
  equation
    force = min(max(winchDrive.force_cmd, 0), max_force);
    pull.f = force;
    connect(pull.flange, cable);
    connect(speed.flange, cable);
    electrical = force * abs(speed.v) / efficiency + standby;
    connect(load.p, p);
    connect(load.n, n);
    load.i = electrical / max(load.v, 1);
  end Winch;

  model Cabin "The cabin and its load: their weight, their height"
    extends CargoHoist.Physical.Cabin;
    Modelica.Mechanics.Translational.Components.Mass body(m=mass + load, s(start=0, fixed=true), v(start=0, fixed=true));
    Modelica.Mechanics.Translational.Sources.Force weight;
    Modelica.Units.SI.Length height = body.s "Height above the ground floor";
  equation
    connect(cable, body.flange_a);
    connect(weight.flange, body.flange_b);
    weight.f = -(mass + load) * g;
    heightSensor.height = body.s "The sensor reads the cabin's height";
  end Cabin;

  model HeightSensor "The sensor: the cabin it is on gives its reading"
    extends CargoHoist.Behavior.Cabin.HeightSensor;
  end HeightSensor;

  model Drive "The winch drive: the winch reads its force command"
    extends CargoHoist.Behavior.Winch.WinchDrive;
  end Drive;

  model HeightHold "A PID on the height, the pull between zero and the limit"
    extends CargoHoist.Behavior.Controller.HeightHold;
    Modelica.Blocks.Sources.Constant setpoint(k=target_height);
    Modelica.Blocks.Continuous.LimPID pid(controllerType=Modelica.Blocks.Types.SimpleController.PID, k=400, Ti=8, Td=1.5, yMax=6000, yMin=0, initType=Modelica.Blocks.Types.Init.InitialOutput, y_start=0);
  equation
    connect(setpoint.y, pid.u_s);
    connect(height, pid.u_m);
    connect(pid.y, force_cmd);
  end HeightHold;

  annotation(uses(Modelica(version="4.1.0")));
end HoistPhysics;
